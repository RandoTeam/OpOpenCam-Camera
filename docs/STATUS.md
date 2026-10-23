# Status

**Version:** `0.4.0-alpha.1` (versionCode 6)
**Date:** 2026-10-22
**Device:** OnePlus 13 `PJZ110`, Android 16, user 0
**Package:** `com.oneplus.camera` (JNI class package `com.opopencam.camera`)
**Public build:** original code only; no vendor binaries, LUTs or calibration assets.

**Current priority:** photo core first — capture speed, quality, saving/gallery, smooth
zoom and lens hand-off. Video remains an experimental 4K feature until the photo
core is accepted; 5K/6K/8K and HDR/Dolby are later workstreams.

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
| Photo preview framing | preview 1440?1080 4:3 active; stills remain 4096?3072 |
| DNG fallback | verified tags: 4096?3072/8192?6144, orientation 6, ColorMatrix1 SRATIONAL |
| Front JPEG | 3280?2464, orientation 8 |
| MediaStore | `content://media/external/images/media/303572`, `DCIM/OpOpenCam/`, `is_pending=0` |
| In-app gallery | thumb + full-screen grid; JPEG preview; DNG/MP4 placeholders |
| Logical zoom | 0.6/1/2/3/6 JPEG captures; 0.6 ~1.1 s, 1? ~0.6 s, 3? ~1.2 s, 6? ~1.1 s |
| RAW speed | RAW12 0.35?0.51 s; RAW50 0.66 s; HDR 3.4 s |
| Video start/stop | VIDEO chip, record/stop, preview restored; repeat cycle works |
| Lens 0.6× / 1× / 3× | camera IDs 3 / 0 / 4 |
| Mode/lens restart | same Surface, preview resumes |
| HAL fallback | unsupported mode falls back to 12 MP JPEG |
| CI | public GitHub Actions green |
| Release workflow | `v0.4.0-alpha.1` APK + SHA-256 published; release APK installed and 4K 4:3 video verified |

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

## Stock camera lab (reconnaissance, 2026-10-23)

The stock camera is a separate system app (`com.oplus.camera` 6.020.171), so it
coexists with our debug app. No package switching is needed for observation.

| Item | Finding |
|---|---|
| Cameras | logical 0 main, 1 front, 2 ultra-wide, 3 tele, 4 tele2; sensors `dodgemain`, `dodgefront`, `dodgeultrawide`, `dodgetele`, `dodgetele2` |
| UI automation | stock UI exposes stable resource IDs; `uiautomator` works without blind taps |
| Photo top bar | flash, global EV, Live Photo, sports, macro, submenu |
| Right tools | soft light, filter, face beauty |
| Bottom deck | latest thumbnail, shutter, front/rear; mode tabs Photo/Portrait/More |
| More grid | Night, Panorama, Movies, Slow motion, Time-lapse, Long exposure, Dual-view, Sticker, XPAN, Tilt-shift, scan/AI entries; some labels clipped in the first capture |
| Vendor config evidence | 8K **30** fps, 4K60, 1080p120, 8192x6144 stills, offline reprocess for 4K60 EIS and video SAT, TurboRAW/bracket modes |
| Live Photo path | vendor `livephoto_coverenhancer` with RRDBNet + QNN HTP libraries; cover enhancement timing unproven |
| Gallery | `com.coloros.gallery3d`; `PhotoEditorActivity` exposes `ACTION_EDIT`; post-processing still unexplained |
| Camera metadata | `dumpsys media.camera` exposes static stream configs and vendor tags; `dumpsys media.metrics` exposes codec sessions |
| Preview session | 9 output streams active: 1920x1440 preview, 4096x3072 RAW/YUV stills, physical UW/tele/tele2, 8192x6144 RAW; still streams idle until capture |
| Debug menus | stock debug/log activities are signature-protected; `adb shell` cannot open them |
| Rootless limits | GPU busy/clock sysfs denied; stock userspace callstacks unavailable; NPU/DSP use must be inferred |

Capture runs, Perfetto traces, Live Photo timing, gallery mutation tests and file
forensics are the next phase. Raw vendor configs, model files and tag dumps stay
local and are not committed. Protocol: [lab/](lab/).

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
| 25 MP JPEG closed | Session accepts a 5888×4416 JPEG reader, but the HAL returns 1920×1440; rejected by SOF guard. |
| RAW 25 probe rejected | 5888×4416 RAW_SENSOR session fails with `createCaptureSession -10009` on logical and physical main; no file. Alternate vendor/pixel-mode paths still pending. |
| 8K/6K video not exposed | Normal Camera2 sessions for 7680×4320, 6016×4512 and their 60 fps variants were rejected; normal stream configs cap at 4096×3072. Stock vendor config indicates 8K**30** via an offline/vendor path, not 8K60; that path is not yet attempted. Dolby encoder exists (`c2.qti.dv.encoder`); HDR/Dolby path still unproven. |
| Photo quality baseline pending | Same-scene comparison against stock/LMC and HDR tone/colour tuning not yet done. |
| No Live Photo yet | Motion Photo container exists; video/audio ring buffer not implemented. |
| 4K 4:3 video verified | 4096×3072 HEVC 30 fps + AAC stereo; A/V tail can be ~0.2–0.5 s longer than video, sync trimming is next. |
| Video modes not yet | 1080p 120, 720p 60, 10-bit/HDR, EIS; 8K30 vendor/offline path unproven; 8K60 is not a stock OnePlus capability. |
| No Pro/portrait/macro yet | Manual controls, overlays, bokeh and focus stacking are backlogged. |
| 4K120 / front 4K unavailable | HAL stream limits; vendor-only upscale/freezes. |
| Dolby Vision unavailable | Requires Dolby licence and vendor encoder; HDR10/HLG planned. |
| Branded looks unavailable | Hasselblad/GR/LUMO/XPAN marks and branded LUTs are not redistributable. |
| Alpha signing | APKs are debug-signed; CI's generated debug key differs from a local debug build, so uninstall before installing a release APK. |
| Package identity | Requires uninstalling the old patched clone `com.oneplus.camera`. |
| No gallery integration | Photos are in app-specific external storage. |

## Current focus

0. Ground-truth lab: stock-camera trace matrix, Live Photo/gallery forensics and
   UI inventory; then the specialist 8K camera. See [lab/](lab/).
1. Photo speed: measure 12 MP/RAW/HDR latency, add 3A/focus readiness and clear
   progress/saving feedback.
2. Photo saving/gallery: MediaStore publication for JPEG/DNG/MP4 plus in-app
   latest thumbnail/gallery.
3. Photo quality baseline: same-scene 12 MP vs HDR vs RAW vs stock/LMC; texture,
   noise, DR and colour.
4. Smooth zoom/lens hand-off: logical camera 0.6–6×, zoom applied to still
   requests, tests at every lens stop.
5. 25 MP RAW probe (RAW_SENSOR 5888×4416 via vendor mode); JPEG 25 MP is closed.
6. Photo-first UI: separate Photo and Video segments; remove still controls from
   Video.
7. Live Photo 3+3 after the photo core.
8. Video matrix (4K/5K/6K/8K, 30/60, HDR/Dolby) only after photo acceptance.

Full feature surface and priorities: [FEATURE_PARITY.md](FEATURE_PARITY.md).
Canonical execution sequence: [ROADMAP.md](ROADMAP.md#execution-order-2026-10-23).

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
