# Stock Camera Ground-Truth Plan

## 1. What this plan must answer

The goal is not "take a few screenshots of the stock app". The goal is a
behavioural model of the stock OnePlus 13 camera that is good enough to:

1. Reproduce the important user-visible behaviour with public Android APIs.
2. Explain every long delay: camera open, AF, shutter, JPEG/HEVC encode, gallery
   enhancement, zoom hand-off.
3. Identify what runs on CPU, GPU, NPU/HTP, DSP and hardware codecs.
4. Separate cleanly portable features from vendor-only features.
5. Give the photo-quality baseline a controlled, repeatable scene and metric set.
6. Define what to measure on the specialist 8K video camera later.

## 2. Package and process map

| Role | Package / process | Notes |
|---|---|---|
| Stock camera | `com.oplus.camera` 6.020.171 | system app under `/product/app/OplusCamera` |
| Camera provider | `android.hardware.camera.provider...vendor_qti` | QTI provider, Camera2 HAL |
| Camera service | `cameraserver` / `media.camera` | stream config, buffers, metadata |
| Stock gallery | `com.coloros.gallery3d` | viewer, editor, possible post-processing |
| AI service | `com.oplus.aiunit` | likely entry point for on-device models |
| Our app | `com.oneplus.camera` | Camera2 NDK engine, debug build |
| MediaStore | `com.android.providers.media` | publication and metadata |

Do not assume that every visual change comes from the camera package. The
gallery, AI service and MediaProvider are separate processes and must be traced
separately.

## 3. What can be measured without root

| Layer | Observable | Tool | Confidence |
|---|---|---|---|
| UI | view tree, resource IDs, labels, bounds | `uiautomator dump` | high |
| UI timing | frame stats, jank, layer latency | `dumpsys gfxinfo`, `SurfaceFlinger --latency` | high |
| UI motion | frame-by-frame animation | `screenrecord`, ffmpeg | medium |
| Input | touch/key timestamps | `getevent -lt`, Perfetto `input` | high |
| Camera service | active clients, stream configs, static metadata, vendor tags | `dumpsys media.camera` | high |
| HAL / ISP slices | atrace `camera`, `hal`; vendor log tags | Perfetto, logcat | medium-high |
| Codec sessions | codec name, mime, size, fps, profile, latency | `dumpsys media.metrics` | high |
| Audio | tracks, sample rate, channels, latency | `dumpsys media.audio_flinger`, `media.metrics` | high |
| Memory | process RSS, system memory | `dumpsys meminfo`, Perfetto `sys_stats` | medium |
| Thermal | zone temperatures, throttling state | `dumpsys thermalservice`, sysfs | high |
| Storage | write throughput, file timing | Perfetto `disk`, file mtime/size | medium |
| Files | JPEG/DNG/MP4 structure and metadata | ffprobe, ExifTool, custom parsers | high |
| Gallery | file mutation, screenshots, process logs | hashes, MediaStore, logcat | high |
| GPU | frequency / busy counters | sysfs, Perfetto | low (permission denied on daily device) |
| NPU / DSP | inference timing, utilization | logcat, threads, Perfetto `nnapi` | low-medium (inferred) |
| Stock callstacks | function-level profile | simpleperf / perfetto callstacks | unavailable rootless |

### Rootless limits and fallbacks

- **GPU counters:** `/sys/class/kgsl/kgsl-3d0/gpubusy` and `gpuclk` are
  permission-denied. Use frame timing, GPU frequency tracks if Perfetto exposes
  them, and compare CPU idle while the preview is smooth. Do not invent a GPU
  utilization number.
- **NPU/HTP:** no public utilization node. Infer from `QNN`/`HTP`/`adsprpc`
  logs, thread activity, inference-shaped latency, and the presence of models.
  A rooted or engineering device is the only way to get real counters.
- **DSP:** same limitation; `adsprpc` logs and remoteproc state are the
  fallback.
- **Stock app callstacks:** the app is not `profileable`; `simpleperf` cannot
  attach to its userspace. Perfetto gives kernel scheduling, binder, frequency
  and vendor atrace slices, not Java/native stacks.
- **HAL request parameters:** only what the HAL logs or `dumpsys` prints. We can
  infer the rest from stream configs, vendor tags and output files.

If the user later wants true GPU/NPU/DSP counters, the correct next step is a
separate test device with root or engineering firmware, not the daily phone.

## 4. Lab setup

### 4.1 Scene

Use one repeatable scene for all photo comparisons:

- a monitor or print with our own test chart: Siemens star, slanted edges,
  grayscale ramp, colour patches, fine text, high-contrast edges;
- a textured physical object (fabric, plant, brushed metal) for detail/noise;
- a gray card for exposure/WB;
- a bright small light and a dark box for dynamic range;
- a millisecond clock for latency and Live Photo pre/post measurement;
- a speaker playing a known beep pattern for audio sync.

Do not use personal photos, documents, faces or rooms as test content.

### 4.2 Light and temperature

- Fixed indoor light; no auto-brightness changes during a run.
- Record lux if a sensor is available; otherwise record the lamp/setting.
- Start each long run at a recorded battery temperature.
- Cool down between thermal runs; never compare a cold run with a hot run.
- Record `dumpsys thermalservice` before and after.

### 4.3 Storage and state

- At least 20 GB free.
- Disable notifications/DND and battery saver.
- Fix display brightness and refresh rate for the run; record both.
- Note Wi-Fi/airplane state; gallery AI may depend on network.
- Snapshot `DCIM/Camera` and MediaStore before the run; identify new files by
  timestamp and `_id`, never by deleting a range.

### 4.4 Time base

Use three clocks and record their relationship:

1. host time (`adb` command timestamp);
2. device `elapsedRealtimeNanos` from logcat/Perfetto;
3. scene clock visible in the captured image.

The scene clock is the only end-to-end ground truth for shutter latency and
Live Photo pre/post. Host time alone includes USB/adb scheduling.

## 5. Run protocol

### Before

1. `adb shell am force-stop com.oplus.camera`.
2. Clear all log buffers: `adb logcat -b all -c`. A plain `-c` can leave old
   buffers in place; filter the resulting file by the camera PID/start time.
3. Reset gfxinfo: `adb shell dumpsys gfxinfo com.oplus.camera reset`.
4. Snapshot: `dumpsys media.camera`, `media.metrics`, `thermalservice`,
   `battery`, `meminfo`, `SurfaceFlinger --list`.
5. Start Perfetto with the lab config.
6. Start logcat capture in the background.
7. Start screen recording if the run is about UI/animation.
8. Launch the stock camera and wait for a stable preview.
9. Record a marker: `adb shell log -t OPC_LAB "START <run-id>"`.

### During

- Drive the UI by resource ID from `uiautomator`, not by remembered coordinates.
- After each action, append a marker with the action name and host time.
- Take `dumpsys media.camera` snapshots at useful moments (preview, AF,
  shutter, encode, gallery).
- For zoom, move in small steps and mark each stop.
- For video, record at least 15 s and include a lens change.

### After

1. Stop screen recording, logcat and Perfetto.
2. Pull the trace, logs, dumpsys files and new media.
3. Hash every pulled file; never edit originals.
4. Run the file forensics from [MEDIA_FORENSICS.md](MEDIA_FORENSICS.md).
5. Write `report.jsonl` and a short `report.md`.
6. Record temperature and battery delta.

### 5.1 Stock app standby and recovery

The stock camera can enter `com.oplus.camera/.setting.ScreenOffActivity`
("Коснитесь для предварительного просмотра") after a period without
interaction. The runner must:

- detect the activity before every action;
- tap the preview centre once and re-read the foreground activity;
- abort the case if a different package is in front;
- never press system `Back` twice (that exits to the launcher); use the UI
  close/back controls and verify the package;
- keep the screen awake and, for long runs, send a harmless keepalive tap.

MediaStore `is_pending=0` does not guarantee the file is complete. The runner
must also wait until the on-disk size is stable twice before pulling, and should
identify new files with a marker plus `find -newer`, not a full MediaStore query
(which can be huge and truncated).

## 6. Scenario matrix

### 6.1 Photo

| ID | Scenario | Variables | Evidence |
|---|---|---|---|
| P01 | cold start -> first photo | app cold, 1x, auto | trace, logcat, screenrecord, latency |
| P02 | warm start -> first photo | app warm, 1x, auto | trace, latency |
| P03 | 12 MP JPEG | 1x, HDR off, AI off | JPEG, EXIF, timing |
| P04 | 12 MP HDR | HDR on, 1x | JPEG, trace, timing |
| P05 | 50 MP / high resolution | 1x, high-res mode | JPEG, dimensions, timing |
| P06 | RAW / Pro RAW | Pro mode, RAW on | DNG, tags, timing |
| P07 | Night | Night mode, 1x/3x | JPEG, multi-frame timing |
| P08 | Portrait | person/target, 1x/3x | JPEG, depth metadata, timing |
| P09 | Macro | close target | JPEG, focus distance, timing |
| P10 | Zoom stops | 0.6/1/2/3/6/10/20/30/60 | JPEG per stop, hand-off trace |
| P11 | Continuous zoom | pinch 0.6 -> 6 -> 0.6 | screenrecord, frame timing |
| P12 | Front | 1x, portrait, night | JPEG, orientation, timing |
| P13 | Flash | off/auto/on, torch | JPEG, flash logs |
| P14 | Timer | 3 s / 10 s | screenrecord, timing |
| P15 | Aspect/resolution | 4:3, 16:9, 1:1, full | dimensions, framing |
| P16 | Burst | long-press shutter | count, interval, thermal |
| P17 | Focus/exposure | tap AF, drag EV, lock | trace, screenrecord |
| P18 | Filters/watermark | each look, watermark on/off | JPEG, metadata |
| P19 | Repeated load | 10x 12 MP, 10x 50 MP, 5x HDR | temperature, latency curve |
| P20 | Error states | storage, thermal, camera busy | screenshots, logs |

### 6.2 Live Photo and gallery

Use the dedicated protocols:

- [LIVE_PHOTO.md](LIVE_PHOTO.md)
- [GALLERY_PIPELINE.md](GALLERY_PIPELINE.md)

### 6.3 Video (observation only until photo is accepted)

| ID | Scenario | Variables |
|---|---|---|
| V01 | 4K 30 | rear main, EIS on/off, 15 s |
| V02 | 4K 60 | rear main, EIS on/off, 15 s |
| V03 | 1080p 120 | slow motion, analysis |
| V04 | 8K 30 | stock capability, crop, EIS, thermal |
| V05 | All lenses | main/UW/tele/tele2 at fixed zoom |
| V06 | Zoom while recording | 0.6 -> 1 -> 3 -> 6 -> 10 |
| V07 | Audio | stereo, wind, audio zoom, sync |
| V08 | HDR / Movie / LOG | mode comparison |
| V09 | Dual-view / time-lapse | feature inventory |
| V10 | Thermal ramp | 10 min 4K/8K, temperature and throttling |

Video implementation stays frozen; these runs only produce ground truth.

## 7. Layer-by-layer investigation

### L0 - App/UI

Questions:

- Which controls exist, in which order, with which states?
- What happens on tap, long-press, swipe, pinch and double-tap?
- Which animations and haptics accompany each action?
- What is shown while the photo is being processed?

Tools: `uiautomator`, screenshots, screen recording, `gfxinfo`, `getevent`.
Output: [UI_INVENTORY.md](UI_INVENTORY.md).

### L1 - Camera API and session

Questions:

- Which logical/physical camera is used at each zoom?
- Is the app using Camera2, CameraX, a vendor SDK or a private native API?
- Does it use reprocessing, multi-resolution streams, 10-bit, or vendor
  session parameters?
- How many outputs are configured for still, Live Photo and video?

Tools: `dumpsys media.camera` before/during/after, HAL logs, Perfetto `camera`,
stream configuration tables. Vendor tags are read locally and summarised; raw
tag dumps are not committed.

### L2 - Camera service and buffers

Questions:

- How many buffers per stream, and what format?
- Is there a ring buffer for Live Photo? How large?
- Are stills captured from a preview stream, a snapshot stream, or a
  reprocessing path?
- Where do frame drops appear?

Tools: `dumpsys media.camera`, Perfetto `camera`/`hal`/`binder`, `meminfo`,
SurfaceFlinger buffer info.

### L3 - HAL, CHI and ISP

Questions:

- Which pipeline stages run per shot: sensor, IFE, BPS, IPE, JPEG, offline
  reprocess?
- Is HDR done in sensor (SHDR/DOL), in ISP, or by multi-frame fusion?
- Which stages are skipped for 50 MP/RAW?
- What does "TurboRAW" and the bracket modes actually do?

Tools: logcat tags (`CAM`, `CHI`, `OPE`, `EVA`, `IFE`, `BPS`, `IPE`, `JPEG`),
Perfetto `camera`/`hal` slices, stream configs, output files. The vendor config
files in `/odm/etc/camera` are read-only evidence for capability names; do not
copy them into the repo.

### L4 - Compute engines

Questions:

- Which work is CPU (app, HAL, algorithms), GPU (preview, LUT, composition),
  NPU/HTP (segmentation, super-resolution, face/body, cover enhancement), DSP
  (audio, sensors, CV) or a hardware codec?
- How long does each stage take, and what is the critical path?
- Does the gallery use the NPU after capture? Does it need network?

Tools: thread names and thread CPU time, `media.metrics`, logcat `QNN`/`HTP`/
`adsprpc`/`fastrpc`, Perfetto `nnapi`/`sched`/`freq`, model inventory in
`/odm/etc/camera` (names only), thermal/power. Do not claim utilization without
a counter.

### L5 - Encode, mux and publish

Questions:

- JPEG encoder: hardware or software, quality, subsampling, orientation, EXIF,
  XMP, MPF, gain map.
- HEVC/AVC encoder: component, profile, level, bitrate, GOP, color metadata.
- Audio: codec, sample rate, channels, bitrate, offset.
- Muxer: when does the file appear? Is it written once or rewritten?
- MediaStore: `is_pending`, `relative_path`, `_size`, `generation_modified`.
- Gallery: does it rewrite the file, add a sidecar, or only change rendering?

Tools: `media.metrics`, ffprobe, ExifTool, MediaStore queries, file hashes,
Perfetto `disk`, gallery logs.

## 8. Metrics and report schema

One JSON object per scenario:

```json
{
  "run_id": "20261024-1015-photo-p03-01",
  "device": "PJZ110",
  "app": "com.oplus.camera",
  "version": "6.020.171",
  "scenario": "P03",
  "scene": "lab_chart_v1",
  "light": "led_5600k",
  "temperature_c": 31.2,
  "started_at": "2026-10-24T10:15:00+03:00",
  "duration_ms": 1830,
  "result": "ok",
  "files": ["media/IMG_...jpg"],
  "metrics": {
    "camera_open_ms": 420,
    "first_preview_ms": 610,
    "af_lock_ms": 180,
    "shutter_to_file_ms": 690,
    "shutter_to_thumbnail_ms": 740,
    "width": 4096,
    "height": 3072,
    "bytes": 1320000,
    "codec": null
  },
  "trace": "perfetto.pftrace",
  "notes": ""
}
```

Core metrics:

- **Latency:** cold start, camera open, first preview, AF lock, shutter-to-file,
  shutter-to-thumbnail, mode switch, lens switch, gallery open, enhancement end.
- **Throughput:** preview fps, capture fps, encode fps, frame drops, write MB/s.
- **Resources:** CPU time per thread/cluster, GPU frequency (if available),
  NPU/DSP inference events (inferred), RSS, dmabuf, power, temperature.
- **Quality:** resolution, texture/MTF, noise, dynamic range, clipping, colour
  error, artifacts, file size.
- **UX:** control response, animation duration, haptic pattern, zoom continuity.

## 9. Automation

Target tools (to be added under `tools/lab/`):

1. `adb_driver.py` - parse `uiautomator` XML, find elements by resource ID or
   text, tap/swipe by bounds; fall back to template matching only when needed.
2. `run_scenario.py` - state machine for launch, mode, zoom, shutter, gallery;
   starts/stops Perfetto and logcat; writes markers.
3. `harvest_media.py` - pre/post MediaStore and filesystem snapshot; pull only
   new test files; hash them.
4. `analyze_media.py` - ffprobe, JPEG marker/XMP/MPF parser, DNG tag reader,
   quality metrics.
5. `report.py` - JSONL to Markdown with tables and links to local artifacts.
6. `perfetto/stock_camera.pbtxt` - atrace `camera,hal,gfx,view,video,audio,
   nnapi,thermal,freq,idle,sched,binder_driver,memory,power,disk,am,wm,input`,
   plus `android.surfaceflinger.frametimeline` and `linux.process_stats`.
   On this device the config must be placed in `/data/misc/perfetto-configs`
   and the trace written to `/data/misc/perfetto-traces`; Perfetto cannot write
   to `/data/local/tmp` as its own user. Do not use `STAT_FREQ_TIMES` - the
   installed Perfetto build rejects it. Use atrace `freq` plus the `cpufreq`
   counter tracks for frequency data.
7. `scene/index.html` - our own test chart with a millisecond clock.

The driver must never rely on a hard-coded coordinate for a labelled control.
Coordinates are allowed only for gestures on the preview (pinch, drag).

## 10. Phases and acceptance

| Phase | Work | Done when |
|---|---|---|
| L0 | Recon and package/UI map | stock app, gallery, AI, camera IDs, UI IDs documented |
| L1 | Photo trace matrix | 12 MP/HDR/50 MP/RAW/Night/Portrait + zoom stops traced |
| L2 | Live Photo forensics | pre/post duration, buffer, codec, audio, XMP, cover measured |
| L3 | Gallery forensics | file rewrite vs display-time vs cloud decided by evidence |
| L4 | UI inventory | every photo control, state, animation and haptic mapped |
| L5 | Video observation | 4K/8K/zoom/audio thermal baseline captured (no product work) |
| L6 | Specialist camera | same protocol on the 8K60 camera; differential report |
| L7 | Synthesis | portable vs vendor-only matrix; implementation backlog updated |

Acceptance for the whole lab:

- every scenario has an original file, a trace/log and a JSONL row;
- every latency number has a trace or a scene-clock measurement;
- every "stock does X" statement is backed by an artifact;
- no vendor binary, asset or raw tag dump is in the public repo;
- the photo-quality baseline can be repeated on the same scene by another
  person.

## 11. Initial reconnaissance (2026-10-23)

Verified read-only:

- stock camera `com.oplus.camera` 6.020.171, main activity `.Camera`;
- five logical cameras with sensors `dodgemain`, `dodgefront`,
  `dodgeultrawide`, `dodgetele`, `dodgetele2`;
- stock UI exposes resource IDs such as `flash_mode`, `global_ev`,
  `live_photo`, `motion_capture`, `macro_close_up`, `submenu`, `zoom_seek_bar`,
  `headline_view`, `thumbnail`, `shutter_button`, `switch_camera_button`;
- the "More" panel contains Night, Panorama, Movies, Slow motion, Time-lapse,
  Long exposure, Dual-view video, Sticker, XPAN, Tilt-shift and scan/AI entries
  (some labels were clipped in the first capture; clean capture is pending);
- vendor configuration names indicate 8K **30** fps, 4K60, 1080p120, 8192x6144
  stills, offline reprocess for 4K60 EIS and video SAT, TurboRAW and bracket
  modes;
- `/odm/etc/camera` contains per-sensor DNG profiles, GAN/detail models, face
  and body models, and a Live Photo cover enhancer (`RRDBNet_sm8750` plus QNN
  HTP libraries); these are observations, not assets to copy;
- `dumpsys media.camera` includes static stream configurations and a large
  vendor-tag set; `dumpsys media.metrics` includes codec session details;
- the stock debug/log activities are signature-protected; `adb shell am start`
  is rejected;
- GPU busy/clock sysfs nodes are permission-denied;
- `simpleperf` can list hardware events but cannot attach a usable event to the
  stock app in this configuration.

The first active-session dump found a **nine-stream preview session** with a
1920x1440 YUV preview, 4096x3072 RAW/YUV still streams, physical ultra-wide/
tele/tele2 streams and an 8192x6144 RAW stream preconfigured but idle. See
[reports/2026-10-23-stock-recon.md](reports/2026-10-23-stock-recon.md) for the
table and implications.

Not yet measured: every capture latency, Live Photo duration, gallery
enhancement, actual 8K30 output, and the full UI animation set.

## 12. Risks

| Risk | Mitigation |
|---|---|
| Stock app updates change behaviour | record version and re-run the affected matrix |
| Thermal drift corrupts comparisons | cool-down, temperature log, interleave runs |
| Gallery processing depends on network/account | run airplane-mode and signed-out variants |
| Vendor configs are mistaken for portable API | mark every finding portable/vendor-only/unknown |
| Screenshots contain personal data | controlled scene, local-only artifacts, review before sharing |
| Too many variables at once | one control per run, matrix ids, report schema |
| Chasing vendor internals instead of shipping | time-box each phase; synthesis updates the product backlog |

## 13. Open questions

1. Is the stock 50 MP path a true 8192x6144 capture or an upscale? Compare RAW
   50 and JPEG 50 on the same scene.
2. Does stock HDR use sensor DOL, ISP SHDR, or multi-frame fusion? Check
   exposure metadata and frame timing.
3. Which zoom stops trigger physical lens changes, and how long is the
   hand-off?
4. Does the stock gallery ever rewrite the original JPEG, or is the improvement
   display-time? See [GALLERY_PIPELINE.md](GALLERY_PIPELINE.md).
5. Is the Live Photo video encoded continuously during preview? See
   [LIVE_PHOTO.md](LIVE_PHOTO.md).
6. Which parts of the stock pipeline are reachable through public Camera2 or
   vendor session parameters, and which require a proprietary SDK?
