# Status

**Version:** `0.4.0-alpha.1` (versionCode 6)
**Date:** 2026-10-22
**Device:** OnePlus 13 `PJZ110`, Android 16, user 0
**Package:** `com.oneplus.camera` (JNI class package `com.opopencam.camera`)
**Public build:** original code only; no vendor binaries, LUTs or calibration assets.

## Verified on device

| Feature | Evidence |
|---|---|
| Camera2 NDK preview | Camera 0 opens; preview active |
| Preview restart latency | ~50 ms after the `onReady` fix |
| 12 MP JPEG | 4096×3072, ~1.3 MB, EXIF orientation |
| RAW 12 → DNG | 4096×3072, ~25 MB, DNG 1.4, 16-bit |
| RAW 50 → DNG | 8192×6144, 96 MB, DNG 1.4, 16-bit |
| HDR 3× | 3 YUV frames → fused JPEG 3072×4096 |
| HDR on ultra-wide / tele | works |
| 4K 4:3 video | 4096×3072 HEVC Main 30 fps, ~79 Mbps; AAC stereo 48 kHz 192 kbps; 16.9 s / 162 MB; rotation −90° |
| 4K 16:9 video | 3840×2160 HEVC ~60 fps, ~82 Mbps; AAC stereo; 2.8 s test; preview restored |
| Video start/stop | VIDEO chip, record/stop, preview restored; repeat cycle works |
| Lens 0.6× / 1× / 3× | camera IDs 3 / 0 / 4 |
| Mode/lens restart | same Surface, preview resumes |
| HAL fallback | unsupported mode falls back to 12 MP JPEG |
| CI | public GitHub Actions green |
| Release workflow | `v0.4.0-alpha.1` APK + SHA-256 published |

## Research completed

| Item | Result |
|---|---|
| Stock/Xposed feature audit | `com.tlsu.opluscamerapro` README + settings + source mirror reviewed; features mapped in [FEATURE_PARITY.md](FEATURE_PARITY.md) |
| Device capability probe | Main/UW/tele: 4096×3072, 4K4:3@30, 4K16:9@60, P010, 1080p120; front max 3280×2464 |
| 25 MP | **Closed by evidence:** session accepts 5888×4416, capture returns 1920×1440 JPEG. Use 12 MP or RAW 50. |
| 4K120 / front 4K | Not advertised; module reports 4K120 freezes; not planned |
| 10-bit P010 | Supported by HAL; video implementation planned |
| Dolby Vision | Not portable without licence/vendor encoder; HDR10/HLG alternative |
| AI gallery | Vendor models; clean/open alternatives only in research |

## Implemented but not fully validated

- HDR quality on textured static scenes and moving subjects.
- DNG color: public build uses a generic sRGB D65 matrix; per-sensor calibration
  is not bundled.
- 50 MP DNG color/black-level accuracy in Lightroom/Photoshop.
- Thermal behavior for repeated HDR / RAW 50 captures.

## Known limitations

| Issue | Detail |
|---|---|
| 50 MP JPEG unavailable | HAL returns 1920×1440 even for an 8192×6144 reader; engine rejects it via JPEG SOF check. Use RAW 50. |
| DNG color uncalibrated | Generic sRGB matrix; user-supplied ColorMatrix/black/white levels needed. |
| HDR alignment | Translation-only NCC; rotation/parallax/moving subjects can ghost. |
| 25 MP unavailable | Session accepts a 5888×4416 reader, but the HAL returns 1920×1440 JPEG; rejected by SOF guard. |
| No Live Photo yet | Motion Photo container exists; video/audio ring buffer not implemented. |
| 4K 4:3 video verified | 4096×3072 HEVC 30 fps + AAC stereo; A/V tail can be ~0.2–0.5 s longer than video, sync trimming is next. |
| Video modes not yet | 1080p 120, 720p 60, front video, 10-bit/HDR, EIS, MediaStore. |
| No Pro/portrait/macro yet | Manual controls, overlays, bokeh and focus stacking are backlogged. |
| 4K120 / front 4K unavailable | HAL stream limits; vendor-only upscale/freezes. |
| Dolby Vision unavailable | Requires Dolby licence and vendor encoder; HDR10/HLG planned. |
| Branded looks unavailable | Hasselblad/GR/LUMO/XPAN marks and branded LUTs are not redistributable. |
| Alpha signing | APKs are debug-signed. |
| Package identity | Requires uninstalling the old patched clone `com.oneplus.camera`. |
| No gallery integration | Photos are in app-specific external storage. |

## Current focus

1. DNG calibration workflow (user-supplied profiles).
2. HDR quality comparison and tuning.
3. A/V tail trimming; 1080p 120; 10-minute thermal test.
4. Live Photo 6 s / stereo audio.
5. Pro mode and gallery foundation.
6. 1080p 120 / 720p 60 / front video.

Full feature surface and priorities: [FEATURE_PARITY.md](FEATURE_PARITY.md).

## Evidence

- [reports/LIVE_2026-10-22.md](reports/LIVE_2026-10-22.md)
- [DEVICE_TEST_PLAN.md](DEVICE_TEST_PLAN.md)
- [CHANGELOG](../rust_camera/CHANGELOG.md)
- Public releases: https://github.com/RandoTeam/OpOpenCam-Camera/releases

## Notes

- The stock camera `com.oplus.camera` remains enabled.
- The old private research repo is not part of the public build.
- Host test count in the public build: 29 (vendor-specific LUT/tone modules are not
  included).
