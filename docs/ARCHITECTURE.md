# Architecture

## Split of responsibilities

| Layer | Responsibility |
|---|---|
| Compose / Kotlin | Window, permissions, `SurfaceView`, gestures, animations, settings, file pickers |
| JNI bridge | Small, stable API between Kotlin and Rust |
| Rust `camera_engine` | `ACameraManager` / `ACameraDevice` / `ACaptureSession`, preview and still outputs |
| Rust media modules | Motion Photo XMP, DNG, ring buffer, HDR fusion |

The camera session is **owned by Rust**. Kotlin passes the `Surface` from a
`SurfaceView`; `ANativeWindow_fromSurface` turns it into a camera output. Preview
buffers are written by the HAL directly into that surface — no CPU YUV→RGBA pass,
no per-frame JNI copies.

## Session layout

```
ACameraDevice (camera 0, logical SAT)
  ├── ACaptureSessionOutput (preview ANativeWindow from SurfaceView)
  └── ACaptureSessionOutput (AImageReader, JPEG 4096×3072, 3 buffers)
        └── onImageAvailable → copy JPEG → write path → notify capture condvar
```

- The preview request uses `TEMPLATE_PREVIEW` with AF continuous-picture, AE on,
  AWB auto.
- The still request uses `TEMPLATE_STILL_CAPTURE`, JPEG quality 95, and
  `JPEG_ORIENTATION = 90` for portrait rear shots.
- The session is created once; preview repeats continuously. A still capture is a
  one-shot `ACameraCaptureSession_capture` to the JPEG reader.
- Session readiness is signalled by `onReady`; the engine waits up to 2 s before
  submitting the repeating request.

## JNI API

Class: `com.opopencam.camera.NativeBridge` (Kotlin `object`).

| Function | Blocking | Notes |
|---|---|---|
| `nativeStartPreview(surface)` | yes (~0.3–2 s) | opens camera and session; takes ownership of the native window |
| `nativeStopPreview()` | no | drops the session and releases the window |
| `nativeCapturePhoto(path)` | yes (up to 6 s) | JPEG capture; call from `Dispatchers.IO` |
| `nativeSetZoom(ratio)` | no | `CONTROL_ZOOM_RATIO` on the repeating request |
| `nativeIsRunning()` | no | session state |

All entry points serialise through a process-global `Mutex<Option<CameraEngine>>`.
Only one camera client may own the preview surface at a time.

## Module map

| Module | State | Purpose |
|---|---|---|
| `bridge.rs` | active | JNI surface |
| `camera_engine.rs` | active | Camera2 NDK session, preview, JPEG capture |
| `sensors.rs` | active | OnePlus 13 topology and vendor-tag constants |
| `motion_photo_xmp.rs` | reusable | JPEG + XMP + MP4 container |
| `dng_writer.rs` | reusable | Adobe DNG 1.4 writer |
| `circular_ring_buffer.rs` | reusable | frame ring for ZSL / Live Photo |
| `capture_modes.rs` | active | 12/50 MP JPEG, RAW, HDR modes |
| `hdr.rs` | active | YUV conversion, NCC alignment, Mertens fusion, JPEG encoding |

The old NativeActivity prototype (`pure_rust_app.rs`, `pipeline.rs`, etc.) is
preserved only in `_archive/rust_track` and is not part of this crate.

## Build portability

`cargo` needs an Android linker. The repository does **not** commit machine-specific
`.cargo/config.toml`. Use `rust_camera/scripts/build.ps1` / `build.sh`, which set:

```
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=<NDK>/.../aarch64-linux-android35-clang
CARGO_TARGET_AARCH64_LINUX_ANDROID_AR=<NDK>/.../llvm-ar
```

`build.rs` links `camera2ndk` only when `CARGO_CFG_TARGET_OS=android`, so host
`cargo test` runs the pure media/math modules without the NDK.

## Package identity

The app is installed as `com.oneplus.camera` (the OnePlus Chi HAL whitelist name).
The Kotlin/JNI class package remains `com.opopencam.camera`, so JNI symbol names
are unchanged. This identity unlocked **RAW 50** (8192×6144 DNG) together with the
vendor session parameters. The old patched clone with the same package must be
uninstalled first; the stock camera `com.oplus.camera` remains enabled and is not
affected.

## Threading rules

- UI thread: Compose only. Never call blocking JNI from it.
- `Dispatchers.IO`: `nativeCapturePhoto`.
- The `onImageAvailable` still callback runs on a HAL thread and only touches
  `Mutex`/`Condvar`-protected state. The engine no longer waits for `onReady`;
  it retries `setRepeatingRequest` for up to ~1 s.
