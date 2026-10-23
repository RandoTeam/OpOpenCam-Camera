# Backlog

Task-level view of [ROADMAP.md](ROADMAP.md) and [FEATURE_PARITY.md](FEATURE_PARITY.md).
IDs are stable; use them in branches (`feat/video-101-...`). One task per PR.

## P0 — now (photo core first)

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| PHOTO-101 | Capture latency instrumentation | — | 12 MP/RAW/HDR timings in logcat; no UI block |
| PHOTO-102 | MediaStore publication + latest thumbnail | — | JPEG/DNG/MP4 visible in system gallery; in-app thumb |
| PHOTO-103 | Apply zoom to still requests | — | 0.6/1/2/3/6 captures match requested FOV |
| PHOTO-104 | Smooth logical zoom + lens hand-off | PHOTO-103 | Slider continuous; no session restart flash |
| PHOTO-105 | Photo quality baseline | — | Same-scene 12 MP/HDR/RAW vs stock/LMC |
| PHOTO-106 | Photo-first UI | — | Photo/Video separate; Photo controls only in Photo |
| PHOTO-107 | 25 MP RAW probe | ? | **Tested:** session rejected `-10009` on logical/main; alternate vendor path pending |
| PHOTO-108 | 3A/focus readiness before capture | PHOTO-101 | Sharper, repeatable stills; latency recorded |
| STILL-102 | 25 MP RAW_SENSOR probe | ? | **Session rejected `-10009`**; try vendor/pixel-mode alternatives before closing |
| DNG-101 | User-supplied calibration file format | — | Loads ColorMatrix1/2 + black/white; documented |
| DNG-102 | Load calibration in the DNG writer | DNG-101 | 12/50 MP DNG uses supplied profile |
| DNG-103 | Validate RAW 12/50 in Lightroom/Photoshop | DNG-102 | Correct colour/levels; no personal screenshots |
| HDR-101 | Textured static HDR comparison | — | 12 MP vs HDR vs stock/LMC; metrics |
| HDR-102 | Moving-subject ghosting test | HDR-101 | Ghosting documented; motion-mask decision |
| HDR-103 | Tone/weight tuning | HDR-101 | Natural look; no clipped highlights |
| STILL-101 | 25 MP (5888×4416) probe | — | **Closed:** session accepts size, capture returns 1920×1440; no file, no upscale |
| JPEG-101 | Document 50 MP JPEG limitation | — | README/STATUS/CAPTURE_MODES updated |
| CAP-101 | Capability baseline | — | `docs/STATUS.md` lists still/video/ISO limits |
| CI-101 | Keep public CI/release green | — | Green on main and tags |
| DOC-101 | Full parity documentation | — | FEATURE_PARITY/ROADMAP/BACKLOG merged |

## P1 — next

> Photo core (P0) must be accepted before new video work. Video rows below are
> existing experiments/next-step candidates, not the current priority.

### Video and audio

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| VIDEO-101 | 4K 4:3 4096×3072 @30 | — | **Verified:** HEVC Main 4096×3072 30 fps, ~79 Mbps, playable |
| VIDEO-102 | 4K 16:9 3840×2160 @60 | VIDEO-101 | **Verified:** HEVC Main ~60 fps, ~82 Mbps |
| VIDEO-103 | Stereo AAC + mux | — | **Partial:** 48 kHz stereo 192 kbps verified; audio tail ~0.15–0.2 s needs trimming |
| VIDEO-104 | EIS/OIS + crop/FOV measurement | VIDEO-101 | Stable handheld; documented FOV |
| VIDEO-105 | Bitrate, lock lens, lock WB, mic status | VIDEO-103 | Controls work; no mid-clip switch |
| VIDEO-106 | 1080p 120 | VIDEO-101 | 120 fps timeline, audio |
| VIDEO-107 | 720p 60 | VIDEO-101 | Stable |
| VIDEO-108 | Front 1080p 60/120 | VIDEO-101 | Front max 8 MP; no 4K claim |

### Live Photo

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| LIVE-101 | Camera2 video output surface | — | Preview + encoder in one session |
| LIVE-102 | HEVC encoder + encoded ring | LIVE-101 | 3 s pre-roll, bounded memory |
| LIVE-103 | AudioRecord + AAC ring | — | 48 kHz stereo 256 kbps |
| LIVE-104 | Mux 3 s + 3 s | LIVE-102/103 | 6.0 s ±50 ms |
| LIVE-105 | Motion Photo XMP | LIVE-104 | Opens in Google Photos |

### Pro, zoom, gallery

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| PRO-101 | Manual ISO/shutter/WB/focus/EV | — | Camera2 manual controls, live readout |
| PRO-102 | Histogram, level, grid | PRO-101 | Accurate, non-blocking overlays |
| PRO-103 | Focus peaking + magnifier | PRO-101 | Usable at 1×/3×/macro |
| PRO-104 | Presets import/export/edit | PRO-101 | JSON format documented |
| ZOOM-101 | Smooth zoom/lens hand-off | — | No crash; latency/FOV measured |
| ZOOM-102 | Front camera digital zoom | — | Works within front sensor limit |
| UI-101 | Gallery/MediaStore | — | Saved photo appears in gallery |
| UI-102 | Mode labels + 50 MP message | — | Clear “use R50” state |
| UI-103 | Global EV slider | — | Photo/video EV works |
| UI-104 | Tap zoom → focal length | ZOOM-101 | 0.6/1/3/6× presets |

## P2 — later

### Still modes

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| STILL-201 | 10-bit HEIF stills | CAP-101 | File opens; 10-bit verified or closed |
| STILL-202 | Ultra HDR / gainmap JPEG | — | Gainmap renders in Android/Photos |
| STILL-203 | Night mode | HDR-103 | Better shadows/noise; no watercolor |
| STILL-204 | Long exposure | PRO-101 | Up to sensor limit; noise handling |
| STILL-205 | Burst/motion/shadow-free | HDR-102 | Sharp moving subject; aligned |
| STILL-206 | Panorama | — | Stitch seams acceptable |
| STILL-207 | Document/QR scanner | — | Crop/perspective; QR decode |

### Portrait, macro, video extras

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| PORT-101 | Portrait bokeh (depth/segmentation) | — | Natural edges; depth saved |
| PORT-102 | Portrait zoom/flash/lens pack | PORT-101 | 35/50/85 equivalents |
| MACRO-101 | Macro mode + close focus UI | — | Focus at minimum distance |
| MACRO-102 | Tele macro | MACRO-101 | Tele close-up works |
| MACRO-103 | Focus stack/depth fusion | MACRO-101 | Sharp front-to-back stack |
| VIDEO-201 | P010/HDR10 or HLG | VIDEO-101 | 10-bit file, correct metadata |
| VIDEO-202 | Sound focus | VIDEO-103 | Directional audio documented |
| VIDEO-203 | Auto frame rate | VIDEO-101 | Switches 24/30/60 safely |
| VIDEO-204 | 720p480 experimental | VIDEO-106 | Stable or closed |
| VIDEO-205 | Timelapse | VIDEO-101 | Interval + encode |
| VIDEO-206 | Filters at all resolutions | COLOR-201 | No 1080p30 restriction |
| VIDEO-801 | 5K/6K/8K 30/60 probe on all rear lenses | VIDEO-101 | Real file dimensions/fps or documented HAL rejection |
| VIDEO-802 | Real HDR/Dolby encoder probe | VIDEO-201 | HDR10/HLG file; Dolby only with a real profile/licence |
| VIDEO-803 | Smooth zoom while recording | PHOTO-104 | No session restart; FOV continuity on UW/main/tele |
| VIDEO-804 | High-resolution thermal test | VIDEO-801 | 1/3/10 min; dropped frames/temperature recorded |

### Colour, watermark, editor

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| COLOR-201 | 33³ `.cube` LUT import | — | User LUT applies; no bundled LUTs |
| COLOR-202 | Original film looks + grain/vignette | — | No branded lookalikes |
| COLOR-203 | Watermark editor (text/image) | — | Original templates; user assets |
| COLOR-204 | 65:24 panorama mode | STILL-206 | Generic aspect; no XPAN branding |
| UI-201 | Basic editor (crop/rotate/filter/watermark) | UI-101 | Non-destructive save |
| UI-202 | Optional subtle beauty | — | Off by default; natural look |
| UI-203 | Underwater colour preset | — | Niche; no hardware claim |
| UI-204 | Debug/diagnostics screen | — | Logs/export, no PII |

## P3 — research

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| AI-301 | Open-model object erase | UI-201 | Optional model; no vendor assets |
| AI-302 | Open-model deblur/enhance | UI-201 | Quality/size measured |
| AI-303 | Relight/best-expression | AI-301 | Optional; privacy-safe |
| COLOR-301 | LUT analysis from before/after | COLOR-201 | Fitted transform quality |
| COLOR-302 | Filter repository | COLOR-201 | Moderation/backend design |
| STILL-301 | Multi-frame super-res tele | PRO-103 | Measurable detail gain |
| STILL-302 | Astro/star/moon stacking | STILL-204 | Clean star field; no vendor algorithm |
| VIDEO-301 | Dual-view video (front+rear) | VIDEO-101 | Works only if concurrent cameras supported |
| AI-304 | Clean scene detection | — | Optional; no vendor model |
| DEV-301 | Multi-device support | — | At least one non-OP13 device |
| UI-301 | Niche modes (light-field, magnetic flash) | — | Only with real hardware support |

## Blocked / closed

| ID | Reason |
|---|---|
| JPEG-50 | HAL returns 1920×1440 for 8192×6144 JPEG; use RAW 50 |
| JPEG-25 | Session accepts 5888×4416, capture returns 1920×1440 JPEG; closed, no upscale |
| VIDEO-4K120 | Not advertised; module reports OnePlus 13 freezes |
| VIDEO-FRONT4K | Front max 3280×2464; vendor upscale only |
| VIDEO-DOLBY | Dolby licence/vendor encoder; use HDR10/HLG |
| LUT-BRANDED | Vendor/branded LUTs are not redistributable |
| AI-VENDOR | Vendor models are proprietary; clean/open alternatives only |
| MARK-BRANDED | Hasselblad/GR/LUMO/XPAN marks require rights |

## How to pick work

1. P0 before P1; unblock dependencies first.
2. One task per PR; update the parity matrix and `STATUS.md`.
3. If the HAL contradicts the plan, close the task with evidence — do not fake it.
4. If a task needs a hardware/licensing decision, add an ADR before coding.
