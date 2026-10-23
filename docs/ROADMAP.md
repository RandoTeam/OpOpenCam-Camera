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

- **Now** — photo core: speed, saving/gallery, quality, smooth zoom, 25 MP RAW
  probe, photo-first UI. Video work is frozen as experimental.
- **Next** — Live Photo, Pro mode, portrait/macro, gallery/editor.
- **Later** — full video matrix (4K/5K/6K/8K, 30/60, all lenses, HDR/Dolby),
  colour tools, 10-bit/Ultra HDR.
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
| M7 | Capability probe + 25 MP JPEG | ✅ JPEG closed; RAW25 probe pending | now |
| M8 | 4K 4:3 30 / 4K 16:9 60 video | 🟡 experimental; video frozen | later |
| M9 | 1080p 120 / 720p 60 / front video | 🚧 | later |
| M10 | Live Photo 6 s + stereo audio | 🚧 | next |
| M11 | Pro mode: manual controls + overlays | 🚧 | next |
| M12 | Photo speed + MediaStore/gallery | 🟡 MediaStore verified; gallery/quality next | now |
| M12B | Gallery editor | 🚧 | next |
| M13 | Smooth zoom + lens hand-off (photo/video) | 🚧 | now |
| M14 | Photo-first UI redesign | 🚧 | now |
| M15 | 25 MP RAW probe | 🚧 | now |
| M16 | Portrait, portrait zoom, macro | 🚧 | next |
| M17 | LUT/filters, watermark editor, panorama | 🚧 | later |
| M18 | 10-bit P010/HDR10, HEIF, Ultra HDR | 🚧 | later |
| M19 | Night, long exposure, burst/motion, document | 🚧 | next/later |
| M20 | 5K/6K/8K 30/60 video, HDR/Dolby, all lenses | 🟣 | later/research |
| M21 | Open-model AI editing research | 🟣 | research |

---

## Now (0–4 weeks)

### M12 — Photo speed and saving/gallery

**Goal:** a photo is fast, visible, and trustworthy.

- Measure end-to-end latency for 12 MP JPEG, RAW 12, RAW 50 and HDR 3×.
- Add a clear capture/saving state; never leave the user wondering.
- Publish JPEG/DNG/MP4 to MediaStore (`Pictures/OpOpenCam`,
  `Movies/OpOpenCam`) and keep the app-specific copy.
- Add an in-app latest thumbnail and basic gallery overlay.
- Add 3A/focus readiness before capture if measurements show soft/slow shots.

**Done when:** photo appears in the system gallery and in-app gallery; latency is
recorded per mode; failures are explicit.

### M13 — Smooth zoom and lens hand-off

- Use logical camera 0 with `CONTROL_ZOOM_RATIO` from 0.6× to 6×.
- Apply the current zoom to **still requests**, not only preview.
- Replace lens-chip session restarts with zoom presets; keep session alive.
- Measure hand-off between ultra-wide/main/tele at 0.6/1/2/3/6.
- Later: allow zoom while video recording without restarting the session.

**Done when:** slider and pills zoom continuously; photo at each stop uses the
requested FOV; no crash/restart flash.

### M14 — Photo-first UI redesign

Design contract: [PHOTO_UI.md](PHOTO_UI.md).

- Separate **Photo** and **Video** top-level segments; Video is not a chip in the
  Photo control row.
- Photo screen: top quick controls, focus/exposure, zoom pills/slider, shutter,
  gallery thumb, mode switch.
- Show RAW/HDR/50 MP/25 MP honestly; hide unavailable JPEG modes from the main
  path.
- Keep the SurfaceView lifecycle; do not recreate it on UI mode changes.
- Defer Pro/Night/Portrait panels to their milestones.

**Done when:** a user can open the app, take a photo, see it in the gallery, zoom
smoothly and switch to Video without the Photo controls leaking into Video.

### M15 — 25 MP RAW probe

- Do not stop at the failed JPEG result.
- Add a controlled RAW_SENSOR 5888×4416 probe with the vendor resolution session
  mode used by RAW 50.
- Validate the DNG dimensions/CFA/black-white levels; reject upscaled/fake files.
- If it works, expose RAW 25; if not, document the exact failure.

**Done when:** a real 5888×4416 DNG opens, or the probe is closed with evidence.

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
- **25 MP JPEG is closed by evidence:** the session accepts a 5888×4416 JPEG
  reader, but the HAL returns **1920×1440**. The SOF guard rejects it.
- **25 MP RAW is not closed:** run the controlled RAW_SENSOR probe in M15.
- Probe 10-bit still HEIF and Ultra HDR paths; do not promise until a file opens.
- Keep 50 MP JPEG closed; RAW 50 is the maximum-resolution JPEG-era path.

**Done when:** [STATUS.md](STATUS.md) states exactly what is possible on this device.

### Release hygiene

- Keep CI green; docs/changelog current.
- Tag alpha only after `check.ps1` and a device smoke test.

---

## Next (1–3 months)

### M10 — Live Photo

**Spec:** 4096×3072 4:3 HEVC 30 fps, 6.0 s (3 s before + 3 s after),
AAC 48 kHz stereo 256 kbps, JPEG 4096×3072, Motion Photo XMP v1.

- First gate: preflight a three-output session (preview + encoder + JPEG).
- If supported: encoded-sample ring + audio ring + muxer + XMP.
- If not supported: document the HAL limit and propose the next clean path; do not
  fake a silent/low-res Motion Photo.

**Done when:** one video + one stereo audio track, 6.0 s ±50 ms, A/V sync <50 ms.

### M11 — Pro mode

- Manual ISO/shutter/WB/focus/EV; read back applied values from capture results.
- Histogram, level, grid, focus peaking, magnified focus.
- RAW/DNG in Pro.
- Presets import/export/edit (original format).

**Done when:** a photographer can expose and focus manually with live feedback and
save JPEG/DNG.

### M16 — Portrait and macro

- Portrait bokeh using depth map or segmentation; natural edges.
- Portrait zoom, rear flash, lens pack (35/50/85 equivalent).
- Macro and tele-macro; focus bracketing + depth fusion.
- No AI super-res/tele claim; clean multi-frame research only.

### M17 — Colour, filters, watermarks, panorama

- User-supplied 33³ `.cube` LUT import and per-mode ordering.
- Original film looks (no branded/vendor LUTs); vignette/grain/soft-light.
- Watermark editor (text/image), custom device name; original designs only.
- Generic 65:24 panorama mode (no XPAN branding).

### M19 — Night, long exposure, burst, document

- Night mode, long exposure, burst/motion, shadow-free burst.
- Panorama, document scanner, QR/barcode, timelapse.
- Optional subtle beauty (off by default), underwater colour preset.

### M12B — Gallery editor

- Basic editor: crop, rotate, filters, watermark.
- Non-destructive export and share; MediaStore-first workflow.

---

## Later (3–6 months)

### M20 — Full video matrix

Video is frozen until the photo core is accepted. Then:

- 4K 4:3 30 / 4K 16:9 60 (already experimental).
- Test 5K (5120×3840 / 5120×2880), 6K (6016×4512 / 6016×3384) and 8K
  (7680×4320) at 24/30/60 fps on all rear lenses.
- Test real HDR: P010/Main10, HDR10/HLG, and whether the vendor HEVC encoder
  exposes a real Dolby Vision profile. Do not label an 8-bit vendor file Dolby.
- Smooth zoom across ultra-wide/main/tele while recording, without stopping.
- EIS/OIS, lock lens/WB, mic status, audio focus, thermal/dropped-frame tests.
- If the HAL does not advertise a mode, test a controlled vendor-session probe
  before closing it.

### M18 — 10-bit and HDR stills

- 10-bit P010 video with HEVC Main10; HDR10/HLG metadata.
- 10-bit HEIF stills via `HeifWriter`/HEVC 10-bit (if supported).
- Ultra HDR stills via SDR JPEG + Android gainmap; no vendor ProXDR.
- Dolby Vision stills/video only if a real encoder profile and licence-compatible
  path are found; otherwise stay HDR10/HLG.

### M21 — Gallery/editor AI research

- Open-model experiments for erase/deblur/enhance/relight/best expression.
- No vendor models or branded assets; optional/downloadable models only.

---

## Research (no promise)

- 25 MP RAW (controlled 5888×4416 RAW_SENSOR probe).
- 5K/6K/8K 30/60 vendor-session probes on all rear lenses.
- Real Dolby Vision profile/encoder probe; HDR10/HLG is the fallback.
- Smooth lens hand-off while recording at high resolutions.
- Multi-frame super-resolution for tele.
- LUT analysis from before/after images.
- Local AI editing with open models.
- Multi-device support.

## Not portable (documented, not planned)

| Item | Reason |
|---|---|
| 50 MP JPEG | HAL returns 1920×1440; system-only |
| 4K 120, front 4K | not advertised; module reports freezes/upscale |
| Dolby Vision | not yet proven; requires a real encoder profile/licence; HDR10/HLG fallback |
| Hasselblad/GR/LUMO/XPAN branding | trademarks/vendor assets |
| Branded filters | licensed looks/vendor LUTs |
| AI super-res and AI gallery models | proprietary models/licensing |
| Vendor ProXDR pipeline | vendor-specific; use Android gainmap |
| Magnetic flash / light-field UI | hardware/vendor protocol |
| Global double-tap volume | privileged/accessibility integration |

## Priorities

| Priority | Work |
|---|---|
| P0 | Photo speed, MediaStore/gallery, zoom/lens hand-off, DNG/HDR quality, 25 MP RAW probe, photo-first UI |
| P1 | Live Photo, Pro mode, portrait/macro, gallery editor |
| P2 | Video matrix 5K/6K/8K 30/60, HDR/Dolby probe, 10-bit/HDR, colour tools |
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
