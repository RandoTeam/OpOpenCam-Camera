# Changelog

> **Public build note:** vendor-specific LUT/tone modules and calibration assets
> are not included in this repository. The public build has 36 host tests.

All notable changes to the OpOpenCam Rust camera are documented here.

> **Public release history:** only `v0.4.0-alpha.1` is currently published. Earlier
> `0.1`?`0.3` entries predate the clean public repository recreation and their tags
> are intentionally not public.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0-beta.1] — 2026-10-27

### Added

- Single rotating portrait canvas (`360x792dp`, `rotationZ = display rotation`)
  with `requiredSize`, so the landscape window can no longer clamp and compress
  the layout. Controls keep their stock position relative to the phone.
- `LocalGlyphRotation` and `Modifier.upright()`: readable glyphs counter-rotate
  in place; the header, status, zoom stops, PHOTO/VIDEO labels and the format
  badge use it.
- Settings entry with a persisted standby toggle (`standby_enabled`).
- Preview re-open when returning from the stock camera, instead of keeping a
  disconnected device that failed on the next capture.

### Fixed

- Preview was horizontally stretched by 1.78x (feature matching against stock:
  `scaleX 0.570 / scaleY 1.009`). Removed the anisotropic matrix step; measured
  after the fix: aspect ratio `1.0000`, residual `1.4px`.
- Preview no longer rotates the frame 90 degrees in landscape; the stream is
  already delivered in screen orientation.
- Gallery thumbnails and the detail view ignored EXIF orientation and showed
  correctly saved stills sideways.
- Portrait preview now starts at the stock anchor `y=300px` instead of being
  centred, so the control deck sits below the image as in the stock camera.

## [Unreleased]

### Added

- Physical orientation detection (`OrientationEventListener`) for the
  landscape-first camera. The standby screen and preview follow the way the
  phone is held even though the activity layout remains fixed; control layout
  reflow is deliberately not changed in this step.
- First landscape UI pass (`sensorLandscape`) with a rotated `TextureView` preview
  and left/right control rails. This is not the stock single-canvas layout; UI-302
  replaces it.
- Orientation-aware JPEG/DNG capture: sensor/display rotation is applied via
  `nativeSetJpegOrientation`; landscape stills use orientation 3 (180 degrees)
  and RAW 50 DNGs carry orientation 3.
- Device-validated physical lens presets: 3x tele (camera 4), 6x tele crop, 0.6x
  ultra-wide (camera 3), 1x logical; separate JPEG/RAW captures are upright.
- HDR 3x output follows the display orientation; landscape HDR is 4096x3072.
- Background still queue: a bounded three-shot channel and one serial worker;
  JPEG/RAW/HDR capture and saving do not block the shutter. Each job carries its
  tap-time orientation, mode/lens/format/PHOTO-VIDEO are blocked while the queue
  drains, and the status shows the pending count. Device-validated with three
  rapid JPEG and three rapid HDR captures.
- Photo UI v2: stock-like PHOTO/VIDEO segmented switch, orange selection accent,
  viewfinder grid, safe-area insets, and a capture row with gallery thumb,
  centered shutter and flip-camera button.
- Top-bar format badge opens a bottom sheet with working formats only; 50 MP
  JPEG is explained as unavailable instead of being a dead chip.
- Pinch-to-zoom on the preview (within the active lens), a live REC timer in
  Video mode, and a centered saving/progress card for RAW 50/HDR.
- Top bar carries the current zoom; the redundant photo status line is gone.
- Format selection preserves the current lens/zoom when supported.
- Zoom presets now select real lenses: 0.6x = physical ultra-wide (camera 3),
  3x = physical telephoto (camera 4), 6x = 2x crop on the telephoto. The app does
  not copy the stock low-light behavior of keeping 3x/6x on the main sensor.
- Physical-lens start failure falls back to the logical camera instead of
  leaving a black preview; status text makes the fallback explicit.
- Build-verified on the host; device visual/interaction validation is pending.

- MediaStore publication for captures (`DCIM/OpOpenCam`, `Movies/OpOpenCam`);
  app-specific copy is kept; JPEG URI verified with `is_pending=0`.
- In-app latest thumbnail and gallery overlay (JPEG preview; DNG/MP4 placeholders).
- In-app gallery DNG detail now offers **Render JPEG**: the neutral Rust
  `raw_render` pipeline runs on `Dispatchers.IO` without taking the camera engine
  lock, writes to app-specific Pictures, publishes to MediaStore and refreshes
  the grid. Build-verified; device render-time validation pending.
- Still captures now apply the current zoom ratio, not only preview zoom.
- Front camera JPEG fixed to its native 3280×2464; orientation 8.
- Debug `SelfTestActivity` matrix runner (JSONL), covering JPEG zooms, UW/tele/front,
  RAW12/RAW25/RAW50, HDR, MediaStore and video/codec probes.
- RAW25 probe: 5888×4416 session rejected (`-10009`) on logical and physical main;
  no DNG written. Alternate vendor path remains research.
- Photo/Video separated at the top of the capture deck; 50 MP JPEG chip removed.
- Preview switched to 4:3 (1440×1080) so framing matches the 4:3 still.
- DNG `ColorMatrix1` fixed to XYZ→camera with SRATIONAL; TIFF orientation 6/8.
- Blocking preview restart/zoom moved off the main thread; zoom coalesced at ~50 Hz.
- Capture timing log (`Camera2 NDK: capture ... finished in ...`).
- Photo-first roadmap: saving/gallery, speed, smooth zoom, 25 MP RAW probe and UI
  redesign are P0; video is frozen.

### Fixed

- Preview counter-rotation now matches the saved still in landscape. The previous
  formula put the preview 90° sideways while the JPEG was upright.
- A gallery-refresh failure no longer strands the capture queue: the worker
  releases its slot in `finally`, so the shutter and mode controls recover.

### Validation pending

- Photo quality baseline and latency numbers.
- Portrait layout: the app still uses `sensorLandscape`; when the phone is held
  portrait the preview and stills are sideways until UI-302.
- CAP-301 queue-full feedback: the shutter disables at three shots; there is no
  explicit "full" status yet.

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

- Photo UI: 30 s idle standby screen ("Tap for preview"), UI-only, session kept alive.
