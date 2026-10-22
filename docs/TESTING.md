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

## Evidence requirements

For each camera test, record:

- build/version/package;
- exact command or UI action;
- logcat lines (`OpOpenCamNative`);
- output path, file size, dimensions/format;
- for DNG: TIFF/DNG tags (width, height, bits, DNGVersion, black/white);
- for video/Motion Photo: `ffprobe` streams/durations;
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
