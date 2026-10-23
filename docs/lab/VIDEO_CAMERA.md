# Specialist Video Camera Checklist

This document is for the second camera the user will provide: a video-first
device with seamless lens switching and 8K60 recording. The goal is to learn
how it achieves those results, compare it with the OnePlus 13, and decide what
is portable to OpOpenCam.

The OnePlus 13 stock configuration indicates 8K **30** fps, not 8K60. Do not
assume the OnePlus can do 8K60; the specialist camera is the 8K60 reference.

## 1. Device intake

Record before the first run:

| Item | Value |
|---|---|
| Model / codename | |
| Firmware / OS | |
| SoC | |
| GPU | |
| NPU / DSP | |
| Camera app package/version | |
| Camera HAL/provider | |
| Logical camera IDs | |
| Sensors and lenses | |
| Max still resolution | |
| Max video resolution/fps | |
| Codecs advertised | |
| HDR formats | |
| Audio inputs | |
| Storage / external SSD | |
| Battery / cooling | |
| Root / debug state | |

Commands:

```bash
adb shell getprop
adb shell dumpsys media.camera
adb shell dumpsys media.metrics
adb shell dumpsys package <camera-package>
adb shell pm list packages | grep -i camera
adb shell dumpsys media.audio_flinger
```

## 2. Capability questions

### 8K60

- Is 8K60 a real sensor readout, a binning/upscale, or a multi-frame merge?
- Which encoder/component handles it: hardware HEVC, vendor encoder, external?
- Profile/level, bit depth, chroma, bitrate, GOP?
- Is it 8K60 in all lenses or only the main lens?
- What is the crop factor and stabilization crop?
- How long can it run before thermal throttling?
- Does it drop frames under load? Measure, do not trust the spec.

### Seamless lens switching

- Is it one logical camera with multiple physical streams?
- Does it use a vendor multi-camera session or separate sessions?
- How are exposure, white balance and focus handed over?
- Is the switch hidden by a crop/zoom blend, a freeze frame, or a dual-stream
  overlap?
- What is the switch latency during recording?
- Are there frame drops or a resolution change at the switch?
- Does audio stay continuous?
- Does the preview stay live?

### HDR / Dolby

- Which HDR format is real: HDR10, HLG, Dolby Vision, vendor log?
- Is the Dolby Vision RPU present in the file, or only a display flag?
- Which profile/level and pipeline?
- Is 10-bit used end-to-end?
- Does the app monitor HDR correctly on the internal display?

### Stabilization

- OIS, EIS, hybrid, horizon lock?
- Crop factors per mode?
- Does stabilization continue through a lens switch?
- Any wobble/rolling-shutter artifacts?

### Audio

- Mic array, stereo width, wind noise reduction?
- Sample rate/bit depth/codec/bitrate?
- Audio zoom / focus?
- A/V sync at start, during a lens switch, and at stop?
- Monitoring (meters, headphones)?

### Storage and power

- Sustained write speed required for 8K60.
- Internal vs external SSD; file system; thermal behavior.
- Battery consumption per minute; charging while recording.
- File splitting at 4 GB / 30 minutes?

## 3. Experiment matrix

| ID | Scenario | Duration | Evidence |
|---|---|---|---|
| S01 | 8K60 main lens, static scene | 30 s | file, ffprobe, thermal, power |
| S02 | 8K60 main lens, pan | 30 s | frame drops, stabilization |
| S03 | 8K60 zoom 0.6 -> 1 -> 3 -> 6 | 30 s | switch timing, continuity |
| S04 | 8K60 low light | 30 s | noise, bitrate, fps |
| S05 | 8K60 10 min | 10 min | thermal curve, throttling, split |
| S06 | 4K120 / 4K60 | 30 s | codec, fps, crop |
| S07 | All lenses at fixed zoom | 15 s each | FOV, colour match |
| S08 | HDR / Dolby | 30 s | metadata, RPU, display |
| S09 | Audio test | 30 s | channels, sync, wind |
| S10 | External SSD | 60 s | sustained write, file split |
| S11 | Cold start -> record | - | latency |
| S12 | Stop -> play -> record again | - | state machine, file integrity |

For every run: Perfetto trace, logcat, `dumpsys media.camera` during recording,
`dumpsys media.metrics`, thermal/power snapshots, screen recording, and the
original file.

## 4. Pipeline questions

Build the same layer model as [STOCK_CAMERA.md](STOCK_CAMERA.md):

- **L0 UI:** video controls, monitoring tools, animations, haptics.
- **L1 API/session:** logical/physical cameras, streams, reprocessing, vendor
  tags.
- **L2 service/buffers:** buffer count, format, backpressure, frame drops.
- **L3 HAL/ISP:** sensor mode, HDR, EIS, lens switch, offline reprocess.
- **L4 compute:** CPU/GPU/NPU/DSP/encoder split.
- **L5 mux/publish:** codec, muxer, storage, metadata.

Specific questions:

1. Does 8K60 use one encoder or multiple slices/instances?
2. Is lens switching done before the encoder or by switching encoder inputs?
3. Is there a frame buffer large enough to hide the switch?
4. Does the ISP run multiple physical streams at once?
5. Is the 8K60 file encoded offline after a burst of frames, or in real time?
6. How does the device avoid thermal throttling: lower bitrate, lower fps,
   crop, or a bigger heat sink?

## 5. UI/UX inventory

Video-first cameras often expose professional controls that the stock OnePlus
app hides. Record:

- record button behavior, pause/resume, lock;
- resolution/fps/codec menu;
- lens/zoom control and switch feedback;
- focus peaking, zebras, waveform, histogram, false colour;
- LUT preview and log/RAW recording;
- audio meters, gain, wind filter;
- stabilization modes;
- battery/storage/time remaining;
- safe-area guides, grid, level;
- playback, scrub, trim, share;
- thermal warnings and auto-stop behavior;
- haptics and animations.

Capture with `uiautomator`, screenshots and 60 fps screen recording. Keep
branded assets local.

## 6. File forensics

Use [MEDIA_FORENSICS.md](MEDIA_FORENSICS.md), plus:

- 8K60 profile/level/tier and decoder compatibility;
- HDR/Dolby metadata and RPU;
- timecode, reel name, camera metadata;
- lens/focal length per clip;
- variable vs constant frame rate;
- audio channel layout and sync;
- file split boundaries and timestamp continuity;
- external SSD file system and write speed.

## 7. Differential analysis vs OnePlus 13

Produce a table:

| Capability | OnePlus 13 stock | Specialist camera | Difference | Portable? |
|---|---|---|---|---|
| 8K60 | no (8K30 observed in config) | target | | |
| Seamless lens switch | vendor SAT, to measure | target | | |
| 4K120 | advertised? measure | | | |
| HDR/Dolby | vendor, unproven | | | |
| Stabilization | EIS/OIS | | | |
| Audio | stereo AAC | | | |
| Storage | internal | | | |
| Monitoring UI | limited | | | |

For each difference, classify:

- **hardware limit** (sensor, ISP, encoder, thermal, storage);
- **vendor software limit** (session params, offline reprocess, proprietary
  pipeline);
- **public-API portable** (Camera2/MediaCodec/GPU shader);
- **unknown** (needs more evidence).

Only the last two can enter the product roadmap without a vendor dependency.

## 8. Acceptance

- 8K60 is verified with a real file, not a menu label.
- Seamless lens switching is measured during recording, with frame-level
  continuity evidence.
- HDR/Dolby claims are backed by file metadata and a real playback test.
- Thermal and storage limits are measured for the advertised duration.
- The differential table is complete enough to decide what OpOpenCam can
  realistically implement.
- No vendor binary, LUT or proprietary asset is committed.

## 9. Open questions for the user

1. Exact model/firmware of the specialist camera.
2. Whether it supports external SSD or only internal storage.
3. Whether we may install a debug/profiling build or use a vendor log mode.
4. Whether a second phone/PC monitor is available for the latency/clock rig.
5. Which 8K60 mode is the priority: main lens only, or all lenses.
