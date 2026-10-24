# Camera Ground-Truth Lab

This directory defines how we study cameras whose source code we do not own.
The first subject is the stock OnePlus 13 camera. The second will be the
specialized 8K video camera.

The lab exists to answer questions before we write product code:

- what the stock app actually does, in what order, and how fast;
- which parts run on CPU, GPU, NPU/HTP, DSP or a hardware encoder;
- how Live Photo pre-roll, still capture, muxing and cover enhancement work;
- why the stock gallery appears to improve a photo seconds after capture;
- which stock features are cleanly portable and which are vendor-only;
- what a specialist 8K camera does differently, so we can decide what is
  realistically implementable in a rootless app.

## Non-goals

- No decompilation or copying of vendor code, resources, LUTs, calibration
  blobs, models or binaries into this repository.
- No claims based on a single screenshot or a single log line.
- No product changes in the same change set as stock-camera research.
- No "root the daily phone" step. If deeper counters are required, that is a
  separate decision on a separate test device.

## Current lab device

| Item | Value |
|---|---|
| Model | OnePlus 13 `PJZ110` |
| OS | Android 16 (API 36), ColorOS/OPLUS 16 |
| SoC | Qualcomm `sun` (SM8750 class) |
| Stock camera | `com.oplus.camera` 6.020.171 (system, `/product/app/OplusCamera`) |
| Our debug app | `com.oneplus.camera` 0.4.0-alpha.1 |
| Stock gallery | `com.coloros.gallery3d` |
| AI service | `com.oplus.aiunit` |
| Cameras | logical 0 (main), 1 (front), 2 (ultra-wide), 3 (tele), 4 (tele2) |
| Root | no |
| Perfetto | v49.0 |
| USB | `a36d1946` |

Stock and our app coexist, so no package switching is needed for observation.

## Rules

1. **Observe behaviour, not copied code.** Traces, logs, `dumpsys`, public
   metadata, file formats and UI states are evidence. Vendor source, smali,
   decompiled libraries and extracted assets are not inputs to the public
   implementation.
2. **Artifacts stay local.** The working evidence root is
   `C:\Temp\opencode\stock-lab`. It is never committed. Screenshots must not
   contain personal photos, messages, account names or serial numbers.
3. **One variable per run.** Same scene, light, zoom, focus, battery/temperature
   and app version; change one control at a time.
4. **Every claim needs an artifact.** A timing claim needs a trace or a
   timestamped log; a quality claim needs the original file and a measured
   crop; a format claim needs the file itself.
5. **Do not change stock debug settings or hidden HAL switches.** Read them if
   exposed; do not toggle them on the daily phone.
6. **Do not delete user media.** New test files are identified by a pre/post
   snapshot and copied to the lab; removal is a separate, explicit action.
7. **Separate research from product.** Research findings go to `docs/lab/`;
   product code changes go to a feature branch after the finding is accepted.

## Evidence root

```text
C:\Temp\opencode\stock-lab\
  runs\<run-id>\
    meta.json                 device, app versions, scene, light, temperature
    perfetto.pftrace          Perfetto trace
    logcat.txt                all buffers, threadtime
    dumpsys\                  media.camera, media.metrics, gfxinfo, thermal, ...
    ui\                       uiautomator XML, screenshots, screenrecord
    media\                    originals pulled from the device
    report.jsonl              one object per scenario
    report.md                 human-readable synthesis
  config\                     read-only vendor config snapshots (local only)
  ui\                         UI reconnaissance snapshots (local only)
```

`run-id` format: `YYYYMMDD-HHMM-<scenario>-<sequence>`.

## Documents

| Document | Purpose |
|---|---|
| [STOCK_CAMERA.md](STOCK_CAMERA.md) | Full stock-camera collection plan: layers, tools, scenarios, phases |
| [LIVE_PHOTO.md](LIVE_PHOTO.md) | Live Photo forensics: pre-roll, buffers, muxing, audio, cover |
| [GALLERY_PIPELINE.md](GALLERY_PIPELINE.md) | Why the stock gallery improves a photo, and integration options |
| [UI_INVENTORY.md](UI_INVENTORY.md) | Complete photo-UI inventory and animation/haptic capture |
| [MEDIA_FORENSICS.md](MEDIA_FORENSICS.md) | JPEG/DNG/MP4/XMP extraction and quality metrics |
| [VIDEO_CAMERA.md](VIDEO_CAMERA.md) | Intake and experiment checklist for the specialist 8K camera |
| [reports/2026-10-23-stock-recon.md](reports/2026-10-23-stock-recon.md) | First reconnaissance: nine-stream stock session, UI IDs, tool limits |
| [reports/2026-10-24-livephoto-smoke.md](reports/2026-10-24-livephoto-smoke.md) | First stock Live Photo: 3.18 s clip, 1.67 s pre-roll, mono AAC, Ultra HDR gain map |
| [reports/2026-10-24-livephoto-finalization.md](reports/2026-10-24-livephoto-finalization.md) | Resolved: JPEG first, gain map/MP4 appended when the preview pauses; no reboot needed |
| [reports/2026-10-24-still-batch.md](reports/2026-10-24-still-batch.md) | 50 MP 8192x6144 JPEG, corrected HDR off/auto, zoom stops, Master RAW/JPEG |
| [reports/2026-10-24-gallery-gainmap.md](reports/2026-10-24-gallery-gainmap.md) | Gallery did not rewrite the file; real ~1.95-stop gain map |
| [reports/2026-10-24-50mp-neutral-render.md](reports/2026-10-24-50mp-neutral-render.md) | Same-scene stock 50 MP vs our RAW50 neutral render; stock has ~2x local gradient energy |

## Status (2026-10-23)

Reconnaissance is complete:

- stock package, version, camera IDs and sensor names identified;
- the stock UI is accessible through `uiautomator` and exposes stable resource
  IDs, so the matrix can be driven without blind coordinate taps;
- the photo top bar and the "More" mode grid were captured;
- `/odm/etc/camera` was inventoried read-only; raw configs and models stay local;
- `dumpsys media.camera` exposes static capabilities and vendor tags;
- `dumpsys media.metrics` exposes codec sessions, resolution, frame rate and
  encoder latency;
- the stock debug/log activities are signature-protected and cannot be opened
  from `adb shell`;
- GPU sysfs counters and userspace callstacks of the stock app are not
  available rootless; see [STOCK_CAMERA.md](STOCK_CAMERA.md) for the fallback.

The first report is [reports/2026-10-23-stock-recon.md](reports/2026-10-23-stock-recon.md).
It records the active nine-stream stock session, a 1920x1440 preview stream,
preconfigured 12 MP/50 MP RAW streams and physical ultra-wide/tele streams.
The first Live Photo report is
[reports/2026-10-24-livephoto-smoke.md](reports/2026-10-24-livephoto-smoke.md):
the first automated stock capture, a 3.18 s clip with a 1.67 s pre-roll,
1728x1296 HEVC, mono AAC and an Ultra HDR gain map. The finalization follow-up
([reports/2026-10-24-livephoto-finalization.md](reports/2026-10-24-livephoto-finalization.md))
resolves the apparent "muxing stopped" regression: the JPEG is written first,
then the gain map and MP4 are appended when the preview pauses (Home or the
120 s `ScreenOffActivity`). All triggers reproduce; no reboot is needed.

The still batch
([reports/2026-10-24-still-batch.md](reports/2026-10-24-still-batch.md)) confirms
a real 8192x6144 stock JPEG, corrects the HDR off/auto measurement (both have
gain maps, ~1.74 stops), and records the low-light zoom stops and Master RAW.
The gallery test
([reports/2026-10-24-gallery-gainmap.md](reports/2026-10-24-gallery-gainmap.md))
shows no file rewrite and a real ~1.95-stop gain map.
