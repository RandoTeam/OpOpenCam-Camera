# Status

**Version:** `0.3.0-alpha.1` (versionCode 5)
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
| Lens 0.6× / 1× / 3× | camera IDs 3 / 0 / 4 |
| Mode/lens restart | same Surface, preview resumes |
| HAL fallback | unsupported mode falls back to 12 MP JPEG |
| CI | public GitHub Actions green |
| Release workflow | `v0.3.0-alpha.1` APK + SHA-256 published |

## Research completed

| Item | Result |
|---|---|
| Stock/Xposed feature audit | `com.tlsu.opluscamerapro` README + settings + source mirror reviewed; features mapped in [FEATURE_PARITY.md](FEATURE_PARITY.md) |
| Device capability probe | Main/UW/tele: 4096×3072, 4K4:3@30, 4K16:9@60, P010, 1080p120; front max 3280×2464 |
| 25 MP | Not advertised by Camera2; one vendor-mode experiment is in `STILL-101` |
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
| 25 MP not available | Not in Camera2 stream configs; vendor Turbo RAW path may not be reachable. One test only. |
| No Live Photo yet | Motion Photo container exists; video/audio ring buffer not implemented. |
| No video recording yet | 4K 4:3 / 16:9, 1080p120, stereo audio and stabilization are next. |
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
3. 25 MP capability probe; close if vendor-only.
4. Video core: 4K 4:3 30 / 4K 16:9 60, stereo audio.
5. Live Photo 6 s / stereo audio.
6. Pro mode and gallery foundation.

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
