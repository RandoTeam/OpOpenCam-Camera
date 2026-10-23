# Stock Camera Reconnaissance - 2026-10-23

This is the first lab report. It is reconnaissance, not a full capture matrix.
All artifacts are local under `C:\Temp\opencode\stock-lab`; no vendor file or raw
tag dump is committed.

## Device

| Item | Value |
|---|---|
| Model | OnePlus 13 `PJZ110` |
| OS | Android 16 / ColorOS 16 |
| Stock camera | `com.oplus.camera` 6.020.171 (system) |
| Our app | `com.oneplus.camera` 0.4.0-alpha.1 (debug) |
| Stock gallery | `com.coloros.gallery3d` |
| AI service | `com.oplus.aiunit` |
| Perfetto | v49.0 on device |
| Root | no |

## Method

- `uiautomator dump` and screenshots for the photo UI.
- `dumpsys media.camera` while the stock preview was active.
- `dumpsys media.metrics`, `gfxinfo`, `thermalservice`, `meminfo`, thread list.
- Read-only inventory of `/odm/etc/camera` names and config text.
- A 5 s Perfetto trace during preview to validate the tracing pipeline.
- No photo was captured in this run.

## Active stock session (preview)

Operation mode: `CUSTOM (32769 / 0x8001)`.

| Stream | Size | HAL format | Consumer / physical | State | Buffers |
|---|---|---|---|---|---|
| 0 | 1280x720 | `0x20` | none | idle | 8 |
| 1 | 1920x1440 | `0x23` YUV | `ImageReader-1920x1440` preview | active, 66 frames | 16 |
| 2 | 4096x3072 | `0x25` | none | idle | 8 |
| 3 | 4096x3072 | `0x23` YUV | physical camera 2 (ultra-wide) | idle | 8 |
| 4 | 4096x3072 | `0x23` YUV | physical camera 3 (tele) | idle | 8 |
| 5 | 4096x3072 | `0x23` YUV | physical camera 4 (tele2) | idle | 8 |
| 6 | 320x240 | `0x23` YUV | `ImageReader-320x240` | idle | 16 |
| 7 | 4096x3072 | `0x25` | vendor use case | idle | 8 |
| 8 | 8192x6144 | `0x25` | none | idle | 8 |

Interpretation (to verify, not final):

- Stock configures **nine output streams while previewing**, including 12 MP
  RAW, 50 MP RAW, and physical streams for ultra-wide/tele/tele2.
- Only the 1920x1440 YUV preview stream was producing frames. The still/RAW
  streams were preconfigured but idle, ready for a capture request without a
  full session rebuild.
- The preview is **1920x1440 (4:3)**, not 1440x1080. Our current preview is
  1440x1080; this is a visible sharpness/FOV difference to investigate.
- The 50 MP RAW stream is **8192x6144**, `0x25` (HAL RAW10), configured at
  preview time.
- Physical camera streams for the ultra-wide, tele and tele2 are present in the
  same logical session. This is the likely basis of the stock multi-camera
  zoom/hand-off and SAT behavior.

Latest request metadata (preview):

- `android.control.afMode = CONTINUOUS_PICTURE`;
- `android.control.afState = PASSIVE_FOCUSED`;
- `android.control.aeTargetFpsRange = [15 30]`;
- `android.control.aeState = CONVERGED`;
- `android.control.awbState = CONVERGED`;
- `android.request.frameCount = 69`, `request.id = 11`;
- vendor focus hints: `com.oplus.focus.is.depth.based = 1`,
  `com.oplus.is.pdaf.done = 1`.

Preview request latency histogram: 71 samples, 85.9% in 80-120 ms, 5.6% in
40-80 ms, 5.6% in 120-160 ms, 1.4% in 160-200 ms, 1.4% above 280 ms. This is
the HAL request interval, not shutter latency.

## UI reconnaissance

Stable IDs observed on the photo screen:

| ID | Label (ru) | Role |
|---|---|---|
| `flash_mode` | `ВспышкаВыкл.` | flash |
| `global_ev` | `Global EVВыкл` | global EV |
| `live_photo` | `Живое фотоoff` | Live Photo |
| `motion_capture` | `Спортивный режимoff` | sports/action |
| `macro_close_up` | `Макросъемкаoff` | macro |
| `submenu` | `SubSet` | submenu |
| `...soft_light` | `soft lightoff` | soft light |
| `...filter` | `filteroff` | filters |
| `...button` | `face beautyoff` | beauty |
| `zoom_seek_bar` | - | zoom |
| `headline_view` | `ФОТОmode` | mode label + gesture |
| `thumbnail` | `Latest Photos` | gallery |
| `shutter_button` | `"Shutter" button` | capture |
| `switch_camera_button` | `Front Camera` | front/rear |

The More grid was captured; labels were clipped by the first screenshot. The
visible entries include Night, Panorama, Movies, Slow motion, Time-lapse, Long
exposure, Dual-view video, Sticker, XPAN and Tilt-shift; scan/AI entries are
unconfirmed. A clean, settled capture is required before the feature list is
final.

## Config-derived capabilities (read-only names/values)

The vendor configuration tree indicates, subject to a real capture test:

- 8K **30** fps recording, not 8K60;
- 4K60 and 1080p120;
- 8192x6144 stills (50 MP class);
- offline reprocessing for 4K60 EIS and video SAT;
- TurboRAW and bracket modes;
- a Live Photo cover enhancer using `RRDBNet_sm8750` and QNN HTP libraries;
- per-sensor DNG color matrices/tone curves and GAN/detail models.

These are capability names from `/odm/etc/camera`; no raw config, model or
calibration data is committed. They must be confirmed with files and traces.

## Tooling findings

- Perfetto works, but its config and output must live under
  `/data/misc/perfetto-configs` and `/data/misc/perfetto-traces`; it cannot write
  to `/data/local/tmp` as the `perfetto` user.
- `STAT_FREQ_TIMES` is rejected by this Perfetto build; use atrace `freq` and
  `counter_track` instead.
- The Python `perfetto` package (trace_processor) works on the pulled trace.
- The 5 s preview trace contains UI/scheduler/binder/frequency data but no
  useful camera HAL slices; a launch/capture trace is required.
- Stock debug/log activities are signature-protected; `am start` is rejected.
- GPU busy/clock sysfs is permission-denied.
- `simpleperf` cannot attach a usable event to the stock app in this
  configuration.

## What this changes in our plan

1. The next stock run must trace **cold launch + preview + shutter + file
   write**, not only idle preview.
2. Compare our current session shape with stock's nine-stream session. A
   preconfigured still/RAW/50MP/physical session may explain stock speed and
   smooth lens hand-off.
3. Raise our preview stream from 1440x1080 to at least 1920x1440 if the display
   and HAL allow it.
4. Test whether Camera2 NDK can open a similar multi-stream session without
   vendor session parameters; if it can, adopt it for still/RAW/zoom.
5. Confirm 8K30 (not 8K60) on the stock device before writing any 8K claim.
6. The specialist 8K60 camera remains the reference for 8K60 and seamless lens
   switching.

## Next experiments

- Cold launch trace + `am start -W` + first preview frame.
- 12 MP JPEG capture trace with scene clock; measure shutter-to-file.
- 50 MP JPEG/RAW capture trace; confirm the 8192x6144 stream is used.
- Zoom sweep trace; see when physical streams become active.
- Live Photo run with `dumpsys media.metrics` before/after shutter.
- Gallery open with a known file; hash before/after and screenshot timeline.
