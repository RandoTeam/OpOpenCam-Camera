# Testing

## Test pyramid

| Level | What | Where |
|---|---|---|
| Static | `cargo fmt --check` | CI + local |
| Unit | pure Rust modules: capture modes, HDR fusion, DNG, Motion Photo | CI + local |
| Lint | Clippy `-D warnings` for `aarch64-linux-android` | CI + local |
| Build | Rust `.so` + Compose debug APK | CI + local |
| Device | real camera behavior, files, thermals | OnePlus 13 |

## Commands

```powershell
.\rust_camera\scripts\check.ps1
```

Equivalent steps:

```bash
cd rust_camera/rust_core
cargo fmt --check
cargo test
cargo clippy --target aarch64-linux-android -- -D warnings
cargo build --release --target aarch64-linux-android
```

Then copy the `.so`, run `./gradlew :app:assembleDebug` and install with
`adb install -r -g`.

## CI

Public GitHub Actions runs the same suite on every push/PR and uploads the debug
APK. The release workflow builds `app-release.apk` and its SHA-256. See [CI.md](CI.md).

## Device matrix

| Device | Android | Status |
|---|---|---|
| OnePlus 13 `PJZ110` | 16 | primary, verified |
| OnePlus 13 work profile user 10 | 16 | not supported by shell access |
| Other devices | — | not supported in the first release |

## Device test tiers

### T0 — smoke

- Launch, permission, preview.
- 12 MP JPEG: 4096×3072, EXIF orientation, opens.
- No crash/ANR.

### T1 — modes

- 50 MP JPEG: expected failure (HAL 1920×1440); no fake file.
- RAW 12: DNG 4096×3072, 16-bit, opens.
- RAW 50: DNG 8192×6144, 16-bit, opens; requires `com.oneplus.camera`.
- HDR 3×: 3 YUV frames, fused JPEG 3072×4096.

### T2 — lenses and zoom

- 0.6× / 1× / 3× → cameras 3 / 0 / 4.
- Restart latency, preview alive, no crash.
- Zoom slider behavior and `activePhysicalId`.

### T3 — Live Photo / video (when implemented)

- 6.0 s clip, one HEVC video + one stereo AAC track.
- `ffprobe` durations, A/V drift, Motion Photo XMP.
- 4K 4:3 30 fps / 4K 16:9 60 fps.

### T4 — stress

- 10 HDR captures, 10 RAW 50 captures.
- 10-minute video; temperature, dropped frames, storage.
- Background/foreground, rotation, permission denial.

### T5 — video parity

- 4K 4:3 30: ✅ 4096×3072 HEVC, ~79 Mbps, stereo AAC; A/V start aligned, tail trim pending.
- 4K 16:9 60, 1080p 120, 720p 60, front 1080p: dimensions, fps, duration, A/V sync.
- EIS/OIS: FOV/crop, handheld stability.
- Lock lens/WB, mic status, bitrate, stop sound.
- 10-bit P010: bit depth, profile, HDR10/HLG metadata.
- 4K120/front 4K/Dolby: must remain documented as unavailable, not tested as
  “working” with upscaled output.

### T6 — still and Pro parity

- 25 MP: session may accept 5888×4416, but capture must be rejected if JPEG is
  1920×1440; closed by evidence, no file.
- 10-bit HEIF / Ultra HDR: bit depth/gainmap verified by a reader.
- Night/long exposure/burst/motion/panorama/document.
- Pro: manual ISO/shutter/WB/focus, histogram/level/peaking, JPEG+DNG.
- Portrait: depth/segmentation edges; macro: focus stack sharpness.

### T7 — colour, watermark, gallery

- LUT import: 33³ `.cube`, before/after screenshots (sanitized).
- Original filters: no branded assets; per-mode ordering.
- Watermark editor: text/image/layout export.
- Gallery: MediaStore visibility, share/delete/favourite, EXIF.
- Editor: non-destructive crop/rotate/filter/watermark export.

## Evidence requirements

For each camera test, record:

- build/version/package;
- exact command or UI action;
- logcat lines (`OpOpenCamNative`);
- output path, file size, dimensions/format;
- for DNG: TIFF/DNG tags (width, height, bits, DNGVersion, black/white);
- for 25 MP: actual JPEG SOF dimensions vs requested 5888×4416;
- for 10-bit/HDR: bit depth, colour profile, HDR10/HLG/gainmap metadata;
- for video/Motion Photo: `ffprobe` streams/durations/audio channels;
- for portrait/macro: depth map presence and edge/focus-stack quality;
- for LUT/filters/watermark: before/after, no branded assets;
- latency and thermal observations.

**Do not commit personal photos or raw device dumps.** Keep them local under
`test_reports/` and attach only sanitized summaries.

## Pass/fail rules

- A crash, ANR, missing file, wrong dimensions or missing audio is a **failure**.
- A HAL limitation is documented, not “passed”.
- “Implemented” without device evidence is “validation pending”.
- If no device is connected, live tests are skipped; static tests must still pass.

## Detailed procedure

See [DEVICE_TEST_PLAN.md](DEVICE_TEST_PLAN.md) for the step-by-step live plan and
[reports/LIVE_2026-10-22.md](reports/LIVE_2026-10-22.md) for the last run.
