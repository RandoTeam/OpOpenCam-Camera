# Roadmap

## North star

A rootless OnePlus 13 camera that is **better than the stock app for serious
photography** and covers the full stock/module feature surface wherever a clean
Android implementation exists:

- natural, detailed 12 MP / 50 MP RAW / HDR stills;
- 4K 4:3 30, 4K 16:9 60, 1080p 120, 720p 60 video with stereo audio;
- Live Photo 6 s (3 s before + 3 s after), 4:3, stereo;
- a real Pro mode, portrait/macro/zoom, filters/LUTs/watermarks;
- a fast gallery/editor and a polished Compose UI.

Full mapping: [FEATURE_PARITY.md](FEATURE_PARITY.md).

Planning horizons:

- **Now** — next 2–4 weeks: image quality and evidence.
- **Next** — 1–3 months: video, Live Photo, Pro mode, basic gallery.
- **Later** — 3–6 months: portrait/macro, colour tools, 10-bit/HDR, editor.
- **Research** — no promise until measured.

Status legend: ✅ verified · 🟡 partial · 🚧 planned · 🟣 research · ⛔ not portable.

## Milestones

| ID | Milestone | Status | Horizon |
|---|---|---|---|
| M0 | Hybrid baseline: preview + real JPEG | ✅ | done |
| M1 | Reliable still capture + EXIF | ✅ | done |
| M2 | 50 MP RAW/DNG | ✅ | done |
| M3 | Lens switching 0.6/1/3× | ✅ | done |
| M4 | HDR 3× + NCC + Mertens | 🟡 pipeline; quality tuning | now |
| M5 | DNG calibration workflow | 🟡 generic matrix; user profiles | now |
| M6 | HDR quality on real scenes | 🚧 | now |
| M7 | Capability probe + 25 MP experiment | ✅ probe; 25 MP closed | now |
| M8 | 4K 4:3 30 / 4K 16:9 60 video | ✅ 4K4:3 + 4K16:9 verified; A/V tail trim pending | next |
| M9 | 1080p 120 / 720p 60 / front video | 🚧 | next |
| M10 | Live Photo 6 s + stereo audio | 🚧 | next |
| M11 | Pro mode: manual controls + overlays | 🚧 | next |
| M12 | Smooth zoom + front zoom + gallery | 🚧 | next |
| M13 | Portrait, portrait zoom, macro | 🚧 | later |
| M14 | LUT/filters, watermark editor, panorama | 🚧 | later |
| M15 | 10-bit P010/HDR10, 10-bit HEIF, Ultra HDR | 🚧 | later |
| M16 | Night, long exposure, burst/motion, document | 🚧 | later |
| M17 | Open-model AI editing research | 🟣 | research |

---

## Now (0–4 weeks)

### M5 — DNG calibration workflow

**Goal:** RAW 12/50 DNG opens with natural colour in Lightroom/Photoshop.

- User-supplied calibration: ColorMatrix1/2, black/white levels, CFA, illuminant.
- No vendor calibration in the public repo.
- Validate 12 MP and 50 MP DNG; record colour error and levels.
- Document in [DNG.md](DNG.md).

**Done when:** a calibrated DNG renders correctly; generic sRGB fallback remains.

### M6 — HDR quality

**Goal:** HDR beats a single frame on real scenes, not just “valid”.

- Textured static scene + moving subject.
- Compare 12 MP / HDR 3× / stock / LMC.
- Tune Mertens weights, tone curve, shadow lift, highlight roll-off.
- Measure ghosting; decide whether motion masks are needed.

**Done when:** documented comparison shows deeper shadows and unclipped highlights
with acceptable ghosting; latency and thermals recorded.

### M7 — Capability probe and 25 MP experiment

- Record Camera2 capabilities: still sizes, P010, 4K60, 1080p120, 720p60,
  high-speed, ISO, front limits.
- **25 MP is closed by evidence:** the Camera2 session accepts a 5888×4416
  JPEG reader, but the HAL returns a **1920×1440** JPEG on capture. The SOF guard
  rejects it. Use 12 MP JPEG or RAW 50; never upscale.
- Probe 10-bit still HEIF and Ultra HDR paths; do not promise until a file opens.
- Keep 50 MP JPEG closed; RAW 50 is the maximum-resolution path.

**Done when:** [STATUS.md](STATUS.md) states exactly what is possible on this device.

### Release hygiene

- Keep CI green; docs/changelog current.
- Tag alpha only after `check.ps1` and a device smoke test.

---

## Next (1–3 months)

### M8 — Video core

- ✅ 4K 4:3 4096×3072 @30, HEVC Main, ~79 Mbps.
- ✅ AAC-LC stereo 48 kHz 192 kbps (HW encoder limit; 256 kbps needs the
  software AAC encoder).
- 🟡 A/V start is aligned; the audio tail can be ~0.2–0.5 s longer. Add audio-tail
  trimming before beta.
- ✅ 4K 16:9 3840×2160 @60, HEVC Main, ~82 Mbps.
- 🚧 EIS/OIS, bitrate presets, lock lens, lock WB, mic status.
- 🚧 10-minute thermal/dropped-frame test.

**Done when:** stable recording, A/V sync <50 ms, playable MP4, no thermal shutdown.

### M9 — High-frame-rate and front video

- 1080p 120 fps (rear and front; front max 1080p).
- 720p 60 fps.
- Front 1080p 60/120.
- 720p 480 slow motion: experimental, only if the high-speed session is stable.
- 4K 120 and front 4K remain closed/not portable.

### M10 — Live Photo

**Spec:** 4096×3072 4:3 HEVC 30 fps, 6.0 s (3 s before + 3 s after),
AAC 48 kHz stereo 256 kbps, JPEG 4096×3072, Motion Photo XMP v1.

- Video output surface + encoded-sample ring buffer.
- AudioRecord + AAC ring.
- MediaMuxer + XMP; verify with `ffprobe` and Google Photos.

**Done when:** one video + one stereo audio track, 6.0 s ±50 ms, A/V sync <50 ms.

### M11 — Pro mode

- Manual ISO/shutter/WB/focus/EV.
- Histogram, level, grid, focus peaking, magnified focus.
- RAW/DNG in Pro.
- Presets import/export/edit (original format).

**Done when:** a photographer can expose and focus manually with live feedback and
save JPEG/DNG.

### M12 — Zoom and gallery

- Smooth zoom and lens hand-off; 0.6/1/3/6× presets; tap-to-focal-length.
- Front camera digital zoom.
- Basic gallery: grid, view, share, delete, favourite, EXIF.
- MediaStore integration and clear mode labels.

---

## Later (3–6 months)

### M13 — Portrait and macro

- Portrait bokeh using depth map or segmentation; natural edges.
- Portrait zoom, rear flash, lens pack (35/50/85 equivalent).
- Macro and tele-macro; focus bracketing + depth fusion.
- No AI super-res/tele claim; clean multi-frame research only.

### M14 — Colour, filters, watermarks, panorama

- User-supplied 33³ `.cube` LUT import and per-mode ordering.
- Original film looks (no branded/vendor LUTs); vignette/grain/soft-light.
- Watermark editor (text/image), custom device name; original designs only.
- Generic 65:24 panorama mode (no XPAN branding).
- LUT analysis/repository: research only.

### M15 — 10-bit and HDR stills

- 10-bit P010 video with HEVC Main10; HDR10/HLG metadata.
- 10-bit HEIF stills via `HeifWriter`/HEVC 10-bit (if supported).
- Ultra HDR stills via SDR JPEG + Android gainmap; no vendor ProXDR.
- Dolby Vision remains not portable.

### M16 — Extended camera surface

- Night mode, long exposure, burst/motion, shadow-free burst.
- Panorama, document scanner, QR/barcode, timelapse.
- Sound focus, auto frame rate, 720p480 if stable.
- Optional subtle beauty (off by default), underwater colour preset, debug screen.

### M17 — Gallery/editor and AI research

- Basic editor: crop, rotate, filters, watermark.
- Open-model experiments for erase/deblur/enhance/relight/best expression.
- No vendor models or branded assets; optional/downloadable models only.

---

## Research (no promise)

- 25 MP if the HAL ever exposes it cleanly.
- Multi-frame super-resolution for tele.
- LUT analysis from before/after images.
- Local AI editing with open models.
- Multi-device support.
- 8K / 4K120 / front 4K only if a future device/HAL supports them.

## Not portable (documented, not planned)

| Item | Reason |
|---|---|
| 50 MP JPEG | HAL returns 1920×1440; system-only |
| 4K 120, front 4K | not advertised; module reports freezes/upscale |
| Dolby Vision | Dolby licence + vendor encoder |
| Hasselblad/GR/LUMO/XPAN branding | trademarks/vendor assets |
| Branded filters | licensed looks/vendor LUTs |
| AI super-res and AI gallery models | proprietary models/licensing |
| Vendor ProXDR pipeline | vendor-specific; use Android gainmap |
| Magnetic flash / light-field UI | hardware/vendor protocol |
| Global double-tap volume | privileged/accessibility integration |

## Priorities

| Priority | Work |
|---|---|
| P0 | DNG calibration, HDR quality, 25 MP closed by evidence, CI/release, no fake 50 MP |
| P1 | Video core, high-frame-rate, Live Photo, Pro mode, zoom, basic gallery |
| P2 | Portrait/macro, LUT/filters/watermark, night/long exposure/burst, 10-bit/HDR, editor |
| P3 | AI/open models, LUT analysis/repo, super-res, niche modes, multi-device |

## Dependencies and risks

| Risk | Mitigation |
|---|---|
| HAL returns unexpected sizes | SOF/dimension guards, fallback, document |
| 25 MP is vendor-only | one experiment, then close |
| DNG colour wrong | user calibration; compare in Lightroom |
| HDR ghosting | short bracket, NCC, motion masks later |
| Live Photo A/V drift | shared monotonic clock, `ffprobe` |
| 4K thermal throttling | temperature log, clip limits, bitrate control |
| Video encoder/format mismatch | capability probe; fallback H.264 |
| Audio focus API limits | stereo/beamforming research; document |
| Trademark/legal exposure | original assets; no vendor lookalikes |
| Scope explosion | one feature per PR; parity matrix status |

## Release cadence

- **Alpha:** each verified increment, debug-signed.
- **Beta:** video + Live Photo + Pro mode feature-complete.
- **Stable:** after DNG calibration, HDR quality, video/Live Photo and gallery are
  accepted.

See [FEATURE_PARITY.md](FEATURE_PARITY.md), [BACKLOG.md](BACKLOG.md),
[PIPELINE.md](PIPELINE.md) and [RELEASING.md](RELEASING.md).
