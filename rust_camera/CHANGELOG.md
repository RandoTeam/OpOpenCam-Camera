# Changelog

> **Public build note:** vendor-specific LUT/tone modules and calibration assets
> are not included in this repository. The public build has 29 host tests.

All notable changes to the OpOpenCam Rust camera are documented here.

> **Public release history:** only `v0.4.0-alpha.1` is currently published. Earlier
> `0.1`?`0.3` entries predate the clean public repository recreation and their tags
> are intentionally not public.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0-alpha.1] ? 2026-10-22

### Added

- **4K 4:3 video**: 4096×3072 HEVC Main 30 fps, ~79 Mbps, AAC-LC stereo 48 kHz
  192 kbps, MP4 with orientation metadata.
- **4K 16:9 video**: 3840×2160 HEVC ~60 fps, ~82 Mbps, same stereo audio.
- VIDEO chip and record/stop control; preview is restored after stop.
- Rust video session: preview + MediaCodec input surface in one Camera2 session.
- Verified on device: 16.9 s / 162 MB 4K4:3 file, rotation −90°, repeated
  start/stop; 4K16:9 60 test at 2.8 s / 27 MB.
- `v0.4.0-alpha.1` release APK installed cleanly and recorded 4K 4:3 video.

### Known limitations

- Audio tail can be ~0.15?0.2 s longer than video; start PTS is aligned.
- 1080p 120, 720p 60, front video, 10-bit/HDR, EIS and MediaStore publication
  are next.
- 25 MP JPEG is closed by evidence: session accepts 5888×4416, capture returns
  1920×1440; no file is written.

## [0.3.0-alpha.1] — 2026-10-22

### Added

- **50 MP RAW → DNG** at 8192×6144 (96 MB, DNG 1.4, 16-bit) on the main sensor,
  using the `com.oneplus.camera` package identity and OPlus/Qualcomm vendor
  session parameters (`ultra.resolution.mode`, `EnableIdealRAW`, package name).
- Vendor session-parameter path via
  `ACameraDevice_createCaptureSessionWithSessionParameters`.
- Package identity switched to `com.oneplus.camera` to unlock the Chi HAL path.
  The JNI class package remains `com.opopencam.camera`.
- Live verification on the OnePlus 13: 12 MP JPEG, RAW 12 DNG, RAW 50 DNG,
  HDR 3× on logical/ultra-wide/tele, and physical lens switching (3/0/4).

### Fixed

- **Session startup latency:** the engine no longer waits 2 s for an `onReady`
  callback that this HAL never invokes; it retries `setRepeatingRequest` every
  50 ms. Preview start dropped from ~2 s to ~50 ms.
- **JPEG dimension validation:** the HAL can return a 1920×1440 JPEG even when the
  reader is configured for 8192×6144. The engine now parses the JPEG SOF and
  rejects mismatches instead of saving a fake 50 MP file.

### Known limitations

- **50 MP JPEG is unavailable** on this device: the HAL returns 1920×1440 for a
  8192×6144 JPEG reader, even with the whitelisted package and vendor parameters.
  Use **RAW 50** for maximum resolution.
- 50 MP RAW requires `com.oneplus.camera`; the old patched clone with that package
  must be uninstalled first.
- 50 MP DNG colour calibration and HDR quality on textured scenes still need tuning.

## [0.2.2-alpha.1] — 2026-10-22

### Fixed

- Link `libmediandk.so` in `build.rs`. The 0.2.1 APK failed at startup with
  `dlopen failed: cannot locate symbol "AImageReader_new"` because the media NDK
  was no longer in `DT_NEEDED` after the project cleanup.
- Added a build-script comment explaining the `camera2ndk` vs `mediandk` split.

## [0.2.1-alpha.1] — 2026-10-22

### Added

- **HDR frame alignment:** normalised cross-correlation on luma (invariant to
  exposure), coarse-to-fine search and a small large-shift penalty. Frames are
  aligned to the middle bracket before Mertens fusion.
- Three new host tests: shift recovery, exposure-invariant matching and aligned
  fusion (29 total in the public build).

### Notes

- Alignment is translation-only. Rotation, parallax and moving subjects can still
  ghost; feature/optical-flow alignment is the next quality step.
- Device validation for the HDR path is still pending.

## [0.2.0-alpha.1] — 2026-10-22

### Added

- **Capture modes:** 12 MP JPEG, 50 MP JPEG, RAW 12, RAW 50, HDR 3×.
- **RAW → DNG** path using the existing DNG 1.4 writer (16-bit RAW_SENSOR).
- **HDR core:** three YUV frames at −2/0/+2 EV, Mertens-style exposure fusion and
  JPEG encoding in Rust. Portrait rotation is applied to the fused output.
- **Lens selection:** logical (0), main (2), ultra-wide (3), tele (4) through the
  JNI start API; the Compose UI exposes 0.6× / 1× / 3×.
- **Mode/lens restart:** changing either restarts the Camera2 session on the same
  `SurfaceView` surface, keeping the stream configuration valid.
- **HAL fallback:** if 50 MP/RAW/HDR session configuration fails, the engine
  automatically retries 12 MP JPEG and reports the active mode to the UI.
- **Motion Photo assembly JNI:** `nativeAssembleMotionPhoto` wraps the existing
  JPEG + XMP + MP4 muxer for the Live Photo milestone.
- `docs/CAPTURE_MODES.md` with mode/fallback/HDR details.

### Verified locally

- 49 host unit tests pass (10 new tests for capture modes and HDR fusion).
- `cargo fmt --check` and Clippy `-D warnings` pass for `aarch64-linux-android`.
- Android debug APK assembles.

### Device validation pending

- 50 MP JPEG/RAW, RAW DNG colour, HDR exposure sequence, physical lens IDs and the
  lens/mode restart path have **not** been run on the phone yet. The device is
  currently away; the verification steps are in `DEVICE_TEST_PLAN.md`.
- HDR currently fuses without motion alignment; a short bracket keeps hand motion
  small, but alignment is the next quality step.

## [0.1.0-alpha.1] — 2026-10-22

### Added

- Hybrid architecture: Rust Camera2 NDK engine + Jetpack Compose shell.
- Direct preview into a `SurfaceView` surface (zero-copy, no CPU YUV conversion).
- Real full-resolution JPEG capture: 4096×3072, EXIF from the OnePlus HAL.
- EXIF orientation (`JPEG_ORIENTATION = 90`) for portrait rear shots.
- JNI bridge for start/stop preview, capture and logical zoom.
- Reusable Rust modules: Motion Photo XMP muxer, DNG 1.4 writer and frame ring
  buffer. Vendor-specific LUT/tone modules are not part of the public build.
- Device baseline: camera capability matrix and reference Motion Photo analysis
  (`DEVICE_TEST_PLAN.md`).
- Versioning, changelog, CI and beta-release scaffolding.

### Verified on device

- OnePlus 13 `PJZ110`, Android 16.
- Camera 0 opens, preview streams, and a 2.8 MB JPEG is written to app storage.
- 39 host-side unit tests pass for the reusable media/math modules.

### Known limitations

- No 50 MP JPEG, RAW or DNG capture yet.
- Lens buttons currently drive `CONTROL_ZOOM_RATIO`; physical camera switching
  is not implemented in the app layer yet.
- No exposure bracketing / HDR fusion yet.
- No Motion Photo, video or audio recording yet.
- The alpha APK is debug-signed and uses the `com.opopencam.camera` package.
- CameraService still logs a `createStream: 0 x …` diagnostic for one output;
  preview and JPEG capture are unaffected.

[0.4.0-alpha.1]: https://github.com/RandoTeam/OpOpenCam-Camera/releases/tag/v0.4.0-alpha.1
