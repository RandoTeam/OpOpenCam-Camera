# OpOpenCam — Device Test Plan & Captured Baseline

Snapshot: `test_reports/device_capture_20261022/` + `test_reports/live_20261022_022/`
Device: OnePlus 13 `PJZ110`, Android 16, user 0.
Active app: `com.oneplus.camera` (applicationId; JNI class package `com.opopencam.camera`).

---

## 1. What is already verified

- Hybrid build works: Rust `camera_engine` + Compose shell.
- Live Camera2 NDK preview to `SurfaceView` works.
- 12 MP JPEG: **4096×3072**, EXIF orientation, ~1.3 MB.
- RAW 12: DNG **4096×3072**, ~25 MB, DNG 1.4.
- RAW 50: DNG **8192×6144**, 96 MB, 16-bit, requires `com.oneplus.camera`.
- HDR 3×: three YUV frames → fused JPEG **3072×4096** on logical, ultra-wide and tele.
- Lens switching: cameras 3 (ultra-wide), 0 (logical), 4 (tele) verified.
- Preview/mode restart latency: ~50 ms after removing the 2 s `onReady` wait.
- 50 MP JPEG: **unavailable**; the HAL returns 1920×1440 and the engine rejects it
  via the JPEG SOF check. Use RAW 50 instead.
- The old patched clone `com.oneplus.camera` must be uninstalled before installing
  OpOpenCam; the stock camera `com.oplus.camera` remains enabled.

## 2. Reference Motion Photo measurements

Analyzed files (full report: `motion_analysis.txt`):

| Source | JPEG | Motion MP4 | Video | Audio | Duration |
|---|---|---|---|---|---|
| Stock `IMG20261021224738.jpg` | 3072×4096 | 6.22 MB | HEVC 1728×1296, ~30 fps, ~19.8 Mbps | AAC 48 kHz **mono**, 191 kbps | 2.32 s |
| Stock `IMG20260922153310.jpg` | 3072×4096 | 8.20 MB | HEVC 1728×1296, ~24.7 fps, ~18.5 Mbps | AAC 48 kHz **mono**, 191 kbps | 3.35 s |
| LMC 8.4 `GCam_…MP.jpg` | 3072×4096 | 2.57 MB | H.264 1024×768 + 2048×1536, 1.43 s | **no audio track** | 1.43 s |
| LMC 8.3 `PXL_…jpg` | 3072×4096 | none | — | — | no Motion Photo |

Stock also stores a **GainMap** JPEG item (HDR gain map, 244–435 KB).

Conclusion: stock does 4:3 video, ~2.3–3.35 s, mono audio. LMC is worse: no audio, dual low-res video tracks, or no Motion Photo at all. Our target can exceed both.

## 3. Camera capability matrix (from `camera_capabilities_summary.txt`)

| Camera | Role | Active array | RAW sizes | Zoom | Orientation |
|---|---|---|---|---|---|
| 0 | Logical SAT | 4096×3072 | 8192×6144, 4096×3072 | 0.6–20 | 90 |
| 1 | Front | 3280×2464 | 6560×4928, 3280×2464 | 1–10 | 270 |
| 2 | Main | 4096×3072 | 8192×6144, 4096×3072 | 1–10 | 90 |
| 3 | Ultra-wide | 4096×3072 | 8192×6144, 4096×3072 | 1–10 | 90 |
| 4 | Tele 3x | 4096×3072 | 8192×6144, 4096×3072 | 1–10 | 90 |

Capabilities (rear): `RAW`, `YUV_REPROCESSING`, `PRIVATE_REPROCESSING`, `BURST_CAPTURE`,
`MANUAL_SENSOR`, `DYNAMIC_RANGE_TEN_BIT`, `LOGICAL_MULTI_CAMERA`, `STREAM_USE_CASE`.

Streams / fps (rear cameras):

- **4096×3072 (4:3)** output at **30 fps** for YUV (33), PRIVATE (34), P010 10-bit (54).
- **3840×2160 (16:9)** output at **60 fps** for 33/34/54.
- **2560×1920 (4:3)** at **60 fps**; **1920×1440** at 60 fps.
- **RAW10 4096×3072** at 30 fps.
- No 60 fps at 4096×3072. 4K 4:3 is a 30 fps mode.
- P010 (10-bit) is available at 4096×3072 30 fps and 3840×2160 60 fps.

Implications:

- **4K 4:3 30 fps video is feasible** (4096×3072, slightly above UHD 4K).
- **4K 16:9 60 fps** is feasible.
- **10-bit HDR video** is feasible via P010 / dynamic-range profiles.
- **50 MP**: 8192×6144 is advertised in RAW sizes; 50 MP JPEG is not in the standard stream list and must be tested via vendor tags (`ultra.resolution.mode`, `EnableIdealRAW`).

## 4. Tests to run when the phone is back

### T1 — Orientation and clean still
1. Point at a scene with horizontal text.
2. Capture via app shutter.
3. Pull JPEG; check EXIF orientation and upright rendering.
Acceptance: 4096×3072, EXIF orientation 6, text upright in gallery, no crop.

### T2 — 50 MP and RAW/DNG
1. Tap `50` and capture. If the status says `HAL fallback: 12 MP`, record it.
2. Tap `RAW` and `R50`, capture both, pull the `.dng` files.
3. Measure capture latency, file size, memory, thermal.
Acceptance: 8192×6144 JPEG or DNG opens in Lightroom/Photoshop; no crash; latency recorded.

### T3 — Lenses and zoom
1. Tap `0.6`, `1x`, `3x`; the session restarts on the same surface.
2. Set `CONTROL_ZOOM_RATIO` 0.6, 1.0, 3.0, 10.0 on logical camera 0.
3. Measure switch latency and visual jump; record `activePhysicalId`.
Acceptance: preview resumes < 300 ms, no crash; the physical ID matches the UI
(or the status explains the HAL restriction).

### T4 — Burst and HDR bracketing
1. Tap `HDR`, capture a static scene; pull the JPEG.
2. Compare shadows/highlights with a single 12 MP frame, LMC and stock.
3. Test a static scene (alignment should remove hand jitter) and a scene with a
   moving subject (translation-only alignment may ghost).
Acceptance: HDR retains highlight and shadow detail; no crash; capture + fusion
latency recorded. Ghosting level on moving subjects documented.

### T5 — Motion Photo 6 s (3 s before + 3 s after)
Target: HEVC, **4096×3072 4:3 30 fps**, **AAC 48 kHz stereo 256 kbps**, JPEG 4096×3072, XMP v1,
6.0 s clip, pre-roll 3.0 s.
1. Encode continuously into a sample ring buffer (encoded, not raw).
2. On shutter, mux 3 s before + 3 s after + still.
3. Verify with `ffprobe` and Google Photos: duration, audio track, stereo, resolution, XMP offset.
Acceptance: 6.0 s clip, stereo audio, 4:3, no A/V drift, file opens as Motion Photo.
Memory estimate: 6 s at 80–120 Mbps ≈ 60–90 MB ring; acceptable, but test thermals.

### T6 — Video and audio
1. ✅ Record 4K 4:3 30 fps, HEVC, stereo AAC 192 kbps; verified 4096×3072,
   ~79 Mbps, 16.9 s / 162 MB, rotation −90°, preview restored.
2. 🚧 Record 4K 16:9 60 fps; test 10-bit P010/HDR profile if the encoder accepts it.
3. 🚧 Record 10 minutes; measure thermals, dropped frames, file size.
Acceptance: stable 30/60 fps, no overheating shutdown, A/V tail <50 ms.

### T7 — UI/UX
- Smooth zoom wheel, lens chips, shutter animation, capture progress, gallery preview.
- Test one-handed use, rotation, background/foreground, permission denial.

### T8 — Comparison
- Same scene: our HDR vs LMC 8.3 vs stock.
- Metrics: highlight clipping, shadow noise, detail retention, color, file size, capture time.

### T9 — Full parity features
- 25 MP: closed — session accepted 5888×4416, capture returned 1920×1440; no file.
- 10-bit HEIF / Ultra HDR: bit depth/gainmap verified by a reader.
- Pro mode: manual ISO/shutter/WB/focus, histogram/level/peaking, JPEG+DNG.
- Portrait/macro: depth map or segmentation edges; focus stack sharpness.
- LUT import, original filters, watermark editor; no branded assets.
- Gallery/editor: MediaStore, share/delete/favourite, non-destructive export.
- Video: 1080p 120, 720p 60, front 1080p, lock lens/WB, mic status.
- Negative tests: 4K120, front 4K, Dolby Vision must remain unavailable/documented,
  not “fixed” by upscaling.

## 5. Offline work while the phone is disconnected

Possible now:

- Refactor/build Rust core and Compose UI.
- Implement 50 MP/RAW/DNG code paths.
- Implement lens switching and zoom.
- Implement exposure bracketing + alignment/merge.
- Implement encoded video/audio ring buffer + Motion Photo XMP.
- Write unit tests and build scripts.
- Analyze all captured data (`media_camera.txt`, configs, Motion Photo samples).

Not possible without the phone:

- Live camera verification, capture, thermal tests, UI interaction, installing APKs.

## 6. Build commands

```powershell
cd rust_camera\rust_core
cargo build --release --target aarch64-linux-android
Copy-Item target/aarch64-linux-android/release/libopopencam.so `
  ..\android_app\app\src\main\jniLibs\arm64-v8a\libopopencam.so -Force

cd ..\android_app
$env:JAVA_HOME = "C:\Program Files\Eclipse Adoptium\jdk-21.0.11.10-hotspot"
.\gradlew.bat :app:assembleDebug
adb install -r -g app\build\outputs\apk\debug\app-debug.apk
```

## 7. Key captured artifacts

- `media_camera.txt` — full CameraService/HAL dump (UTF-16LE).
- `camera_capabilities.json` / `camera_capabilities_summary.txt` — parsed camera matrix.
- `CameraHWConfiguration.config`, `fwk_config.json`, `camera_engmode.xml`.
- `odm_config/` — per-sensor configs, APS config, preview decision config.
- `dng/` — color matrices, tone curves, hue/sat maps.
- `livephoto_coverenhancer/` — stock Live Photo enhancer models.
- `media_codecs_sun_vendor.xml`, `media_profiles_sun.xml`, performance XML.
- Motion Photo JP/MP4 samples and `motion_analysis.txt`.
