# Feature Parity: Stock Camera and OPCameraPro

This document maps **everything the stock OPlus camera does** and **everything the
`com.tlsu.opluscamerapro` Xposed module unlocks** to a clean implementation in
OpOpenCam. It is a behaviour-level, clean-room plan: feature names and observable
behaviour are facts; no module code, vendor binaries, LUTs or assets are copied.

## Sources and evidence

- Stock camera UI and feature set on OnePlus 13 `PJZ110`, ColorOS 16.
- [com.tlsu.opluscamerapro](https://github.com/Xposed-Modules-Repo/com.tlsu.opluscamerapro)
  README, settings and public source mirror
  ([Weverses/OPCameraPro](https://github.com/Weverses/OPCameraPro), GPL-3.0) — used
  for feature discovery only.
- Device Camera2 capabilities from `dumpsys media.camera` (2026-10-22).
- Our live results in [STATUS.md](STATUS.md) and
  [reports/LIVE_2026-10-22.md](reports/LIVE_2026-10-22.md).

> The module is GPL-3.0 and depends on root/LSPosed hooks inside the stock app.
> Our app is MIT and rootless. We reproduce **capabilities**, not code.

## Device capability baseline

| Camera | App ID | Still stream | Video | ISO | Notes |
|---|---|---|---|---|---|
| Main | 0 (logical) / physical 2 | 4096×3072 binned; RAW 50 8192×6144 via vendor mode | 4096×3072@30, 3840×2160@60, 1080p@120/240/480, P010 | 100–6400 | 50 MP RAW verified; 50 MP JPEG closed |
| Ultra-wide | 3 | 4096×3072 | 4096×3072@30, 3840×2160@60, 1080p@120/240, P010 | 100–16000 | 0.6× |
| Tele | 4 | 4096×3072 | 4096×3072@30, 3840×2160@60, 1080p@120/240, P010 | 100–6400 | 3× |
| Front | 1 | 3280×2464 | max 3280×2464; 1080p@120, P010 | 100–6400 | no native 4K |

**Closed by hardware/HAL:** 50 MP JPEG (returns 1920×1440), 4K 120 fps, native front
4K. **Not advertised but worth one test:** 25 MP (5888×4416). **Available but not
implemented:** 10-bit P010 video, 4K 60, 1080p 120, 720p 60, 720p 480 high-speed.

Legend:

- ✅ verified on device
- 🟡 partial / quality work remains
- 🔵 planned, standard Android/Camera2 path
- 🟣 research/experiment; no promise until measured
- ⛔ not portable cleanly (vendor algorithm, trademark, license or hardware)
- ❌ closed by HAL evidence

---

## 1. Stills and image quality

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| 12 MP JPEG | yes | existing | done | ✅ |
| 50 MP RAW/DNG | 50 MP | existing | done | ✅ |
| 50 MP JPEG | stock system path | reject mismatched JPEG; use RAW 50 | closed | ❌ |
| 25 MP Turbo RAW / JPEG | vendor flags | request 5888×4416 once; if HAL returns smaller, close | P0 | 🟣 |
| HDR stills | stock HDR | 3-frame NCC + Mertens; quality tuning | P0 | 🟡 |
| Night mode | stock night | multi-frame night + tone mapping | P2 | 🔵 |
| Long exposure | Pro mode | manual shutter, up to sensor limit | P2 | 🔵 |
| 10-bit HEIF stills | module 10-bit photos | `HeifWriter`/HEVC 10-bit still; test format support | P2 | 🟣 |
| Ultra HDR / ProXDR stills | vendor ProXDR | SDR JPEG + Android gainmap, own encoder; no vendor ProXDR | P2 | 🟣 |
| HEIF background-blur edit | module | capture depth map; edit blur in our gallery | P2/P3 | 🟣 |
| AI composition / level | module AI composition | rule-of-thirds, horizon/level, framing hints | P2 | 🔵 |
| Tap zoom → focal length | module | tap 0.6/1/3/6× to switch lens/zoom | P1 | 🔵 |
| Custom watermark name | module | own watermark + user text; no Hasselblad mark | P2 | 🔵 |
| Hasselblad ultra-clear | vendor | clean high-res still/Live Photo; no Hasselblad colour | — | ⛔ |
| Extended ISO 12800 | module | expose native ISO (main 6400, UW 16000); no 12800 claim | P2 | 🟡 |
| Burst / motion / shadow-free | module | multi-frame burst + alignment; motion mode | P2 | 🔵 |
| Panorama | stock | stitch gyro/feature-aligned frames | P2 | 🔵 |

## 2. Pro / Master mode

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| Manual ISO/shutter/WB/focus/EV | Pro mode | Camera2 manual controls, live readout | P1 | 🔵 |
| Focus peaking / magnified focus | module | GPU/Compose overlay + magnifier | P1/P2 | 🔵 |
| Histogram / level / grid | stock | Compose overlays | P1 | 🔵 |
| RAW/DNG in Pro | module RAW MAX | existing DNG path, Pro UI | P1 | ✅/🔵 |
| Presets (import/export/edit) | module | JSON presets for exposure/WB/filter | P2 | 🔵 |
| Filter badges / preset watermark | module | optional UI + watermark | P3 | 🔵 |
| Master Mode 2.0 vendor UI | vendor | clean Pro UI, not a clone | — | ⛔ |
| Light-field UI | vendor | not planned | — | ⛔ |

## 3. Zoom, portrait and macro

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| 0.6× / 1× / 3× lens switch | yes | existing session restart; improve latency | done | ✅ |
| Smooth zoom / seamless lens | stock | zoom ratio + logical camera; measure hand-off | P1 | 🟡 |
| Portrait mode / bokeh | stock | depth map or segmentation; natural edge handling | P2 | 🟣 |
| Portrait zoom | module | zoom within portrait mode | P2 | 🔵 |
| Front camera zoom | module | digital zoom up to front sensor limit | P1 | 🔵 |
| Rear flash in portrait | module | flash + portrait mode | P2 | 🔵 |
| Portrait lens pack (35/50/85) | module | equivalent focal-length presets/crops | P2 | 🔵 |
| Macro mode | stock/module | close focus + macro UI; tele macro | P2 | 🔵 |
| Macro depth fusion | module | focus bracketing + stack | P2/P3 | 🟣 |
| AI super-clear tele / SDSR | vendor AI | clean multi-frame super-res research only | P3 | ⛔/🟣 |

## 4. Video and audio

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| 4K 4:3 30 fps | stock | Camera2 PRIVATE/P010 → HEVC → MP4 | P1 | 🔵 |
| 4K 16:9 60 fps | stock | same pipeline, 3840×2160@60 | P1 | 🔵 |
| 1080p 120 fps | module | high-speed session + 120 fps timeline | P1 | 🔵 |
| 720p 60 fps | module | standard video | P1 | 🔵 |
| 720p 480 slow motion | module | high-speed 720p480; experimental, audio/slow-mo mux | P2/P3 | 🟣 |
| 4K 120 fps | module | HAL does not advertise; module reports freezes | — | ❌ |
| Front 4K | module | front max 3280×2464; vendor upscale only | — | ⛔ |
| Front 1080p 60/120 | module | front camera video | P1 | 🔵 |
| 10-bit P010 / HDR10/HLG | module Dolby/10-bit | P010 + HEVC Main10; HDR10/HLG metadata | P2 | 🔵 |
| Dolby Vision | module | requires Dolby licence/vendor pipeline; not portable | — | ⛔ |
| Lock lens while recording | module | disable lens hand-off | P1 | 🔵 |
| Lock white balance | module | lock AWB during recording | P1 | 🔵 |
| Mic status detection | module | `AudioRecord`/`AudioManager` state + warning | P1 | 🔵 |
| Stop-recording sound | module | immediate UI sound | P1 | 🔵 |
| Sound focus / directional audio | module | stereo beamforming/zoom audio research | P2 | 🟣 |
| Auto frame rate | module | 24/30/60 selection by light/scene | P2 | 🟣 |
| Filters at all resolutions | module | our filter pipeline is resolution-independent | P1/P2 | 🔵 |
| Stabilization (EIS/OIS) | stock | Camera2 stabilization modes; measure crop/FOV | P1 | 🔵 |
| Video bitrate control | stock | 40–120 Mbps presets | P1 | 🔵 |
| Timelapse | stock | interval capture + encode | P2 | 🔵 |
| Dual-view video (front+rear) | stock | concurrent-camera research; device support TBD | P3 | 🟣 |
| Astro/star/moon | stock | long exposure + stacking; no vendor algorithm | P3 | 🟣 |
| AI scene detection | stock | optional clean classifier; no vendor model | P3 | 🟣 |

## 5. Live Photo / Motion Photo

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| Live Photo 6 s (3+3) | stock/module | encoded HEVC ring + audio ring + mux | P1 | 🔵 |
| 4:3 4096×3072, 30 fps | stock/module | target spec | P1 | 🔵 |
| Stereo AAC 256 kbps | stock | AudioRecord stereo + AAC | P1 | 🔵 |
| High bitrate | module | 45–120 Mbps configurable | P1 | 🔵 |
| Adjustable duration | module | 3+3 default; 1–6 s presets | P2 | 🔵 |
| Master mode Live Photo | module | Pro + Live Photo | P2 | 🔵 |
| FOV optimization | module | match still/video crop | P2 | 🟣 |
| HEIF / 10-bit Live Photo | module | HEIF container + 10-bit video | P3 | 🟣 |
| Live cover ProXDR | module | gainmap cover | P3 | 🟣 |

## 6. Colour, filters and watermarks

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| LUT import (33³ `.cube`) | module | user-supplied LUT engine, no bundled LUTs | P1/P2 | 🔵 |
| LUT analysis from before/after | module | research; fit a colour transform | P3 | 🟣 |
| Filter repository / upload | module | later, with moderation/backend; not near-term | P3 | 🟣 |
| 70+ branded/vendor filters | module | **not copied**; original looks only | — | ⛔ |
| Hasselblad / GR / LUMO / XPAN marks | vendor | original watermark/UI only | — | ⛔ |
| XPAN 65:24 panorama | module | generic panoramic aspect + crop | P2 | 🔵 |
| Vignette / grain / soft light | module | original film-look shaders | P2 | 🔵 |
| Watermark editor (text/image) | module | Compose/Skia editor; user assets | P2 | 🔵 |
| Custom device name | module | text watermark field | P2 | 🔵 |
| Preset/filter ordering per mode | module | per-mode filter list | P2 | 🔵 |

## 7. Other camera features

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| EV adjustment | module/stock | global EV slider, photo/video | P1 | 🔵 |
| Beauty menu | module | optional subtle beauty; off by default | P2/P3 | 🟣 |
| Document / text scanner | module | document edge detect + crop + OCR later | P2 | 🟣 |
| Double-tap volume quick launch | module | app shortcut/quick tile; global key needs accessibility | P2 | 🟡 |
| Motion mode | module | burst + motion alignment | P2 | 🔵 |
| Underwater mode | module | niche colour/UI preset; hardware sealing not implied | P3 | 🟣 |
| Magnetic flash support | hardware | not portable without vendor flash protocol | — | ⛔ |
| Debug mode | module | developer diagnostics screen | P3 | 🔵 |
| QR/barcode scan | stock | optional scanner mode | P2 | 🔵 |

## 8. Gallery and editor

| Feature | Stock / module | Our plan | Priority | Status |
|---|---|---|---|---|
| Basic gallery | stock | grid, view, share, delete, favourite, EXIF | P1 | 🔵 |
| Basic editor | stock | crop/rotate/filter/watermark | P2 | 🔵 |
| AI inspiration / erase / deblur / enhance / dereflection / best expression / relight | vendor AI | not portable now; open-model research later | P3 | ⛔/🟣 |
| Live cover ProXDR | vendor | gainmap cover | P3 | 🟣 |
| LUMO watermark | vendor | original watermark only | — | ⛔ |
| Custom watermark editor | module | text/image/layout editor | P2 | 🔵 |

---

## 9. Not portable, and why

| Item | Reason | Clean alternative |
|---|---|---|
| 50 MP JPEG | HAL returns 1920×1440; system-only path | RAW 50 DNG |
| 25 MP Turbo RAW | vendor pipeline; untested via Camera2 | one 5888×4416 experiment, then close |
| 4K 120 / front 4K | not advertised; module reports 4K120 freezes | 4K60, 1080p120, front 1080p120 |
| Dolby Vision | Dolby licence + vendor encoder | HEVC Main10 + HDR10/HLG |
| Hasselblad colour/watermark | trademark and vendor processing | own colour science and watermark |
| GR / LUMO / XPAN branding | trademarks/vendor assets | original looks; generic 65:24 panorama |
| Branded filters (Jiang Wen, Jia Zhangke, etc.) | licensed looks/vendor LUTs | original film emulations + user LUTs |
| AI super-res / AI gallery models | proprietary models, quality and licensing | open-model research, optional/downloadable |
| ProXDR vendor pipeline | vendor-specific | Android gainmap / Ultra HDR |
| Magnetic flash, light-field UI | hardware/vendor protocol | not planned |
| Global double-tap volume | needs privileged/accessibility integration | app shortcut/quick tile |

## 10. Phased parity plan

### P0 — now (quality and evidence)

- DNG calibration; HDR quality on textured/moving scenes.
- One 25 MP experiment; close if HAL does not deliver.
- Capability probe for 10-bit still HEIF, Ultra HDR, 720p480, front 120.
- Keep 50 MP JPEG closed and documented.

### P1 — next (headline parity)

- 4K 4:3 30, 4K 16:9 60, 1080p 120, 720p 60, front 1080p.
- Stereo audio, EIS/OIS, lock lens/WB, mic status, bitrate.
- Live Photo 6 s, 4:3, stereo, Motion Photo XMP.
- Pro mode: manual ISO/shutter/WB/focus, EV, histogram/level, focus peaking.
- Smooth zoom/lens hand-off; front zoom.
- Basic gallery; tap-to-focal-length.

### P2 — later (full camera surface)

- Portrait/bokeh, portrait zoom/flash/lens pack.
- Macro and tele-macro; focus stacking.
- Night mode, long exposure, burst/motion, panorama, document scanner.
- LUT import, original filters, watermark editor, 65:24 panorama.
- 10-bit P010 video, HDR10/HLG, 10-bit HEIF, Ultra HDR stills.
- 720p480 experimental, sound focus, auto frame rate, timelapse.

### P3 — research (no promise)

- Open-model AI editing (erase/deblur/enhance/relight/best expression).
- LUT analysis/repository; multi-frame super-res.
- Underwater/debug/niche modes; multi-device support.

## 11. Acceptance criteria

A parity feature is accepted only when:

1. It works through public Camera2/Android APIs on the OnePlus 13, rootless.
2. Output dimensions, format, audio tracks and duration are verified with files and
   `ffprobe`/DNG metadata.
3. It does not copy vendor code/assets or use a trademarked look without a licence.
4. It has a clean failure path (fallback, clear UI message, no fake file).
5. `STATUS.md`, `ROADMAP.md`, `BACKLOG.md`, tests and user docs are updated.

See [ROADMAP.md](ROADMAP.md), [BACKLOG.md](BACKLOG.md), [PIPELINE.md](PIPELINE.md),
[VIDEO.md](VIDEO.md), [MOTION_PHOTO.md](MOTION_PHOTO.md).
