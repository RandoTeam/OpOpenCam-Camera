# Roadmap

Planning horizons:

- **Now** — next 2 weeks, unblocks quality and release confidence.
- **Next** — 2–8 weeks, the missing headline features.
- **Later** — 2–6 months, polish and depth.
- **Vision** — the product we are building toward.

Status legend: ✅ verified on device · 🟡 implemented/partial · 🚧 planned.

## Milestones

| ID | Milestone | Status | Horizon |
|---|---|---|---|
| M0 | Hybrid baseline: preview + real JPEG | ✅ | done |
| M1 | Reliable still capture + EXIF | ✅ | done |
| M2 | 50 MP / RAW / DNG | ✅ RAW; 50 MP JPEG unavailable | done |
| M3 | Lens switching and zoom | ✅ cameras 3/0/4 | done |
| M4 | HDR 3× + NCC alignment + Mertens | ✅ pipeline; quality tuning | now |
| M5 | DNG calibration workflow | 🟡 generic matrix; user profiles | now |
| M6 | HDR quality on real scenes | 🚧 | now/next |
| M7 | Live Photo 6 s + stereo audio | 🚧 | next |
| M8 | 4K 4:3 / 16:9 video + audio | 🚧 | next |
| M9 | UI, gallery, beta/stable | 🚧 | later |

---

## Now (0–2 weeks)

### M5 — DNG calibration workflow

**Goal:** RAW 12/50 DNG opens with natural color in Lightroom/Photoshop.

- Define a user-supplied calibration file (ColorMatrix1/2, black/white levels,
  CFA, illuminant).
- Load it from app storage; do not bundle vendor data.
- Validate 12 MP and 50 MP DNG; record color error and black level.
- Document the workflow in [DNG.md](DNG.md).

**Done when:** a calibrated DNG renders correctly; the public build still ships a
generic sRGB fallback.

### M6 — HDR quality

**Goal:** HDR looks better than a single frame on real scenes, not just “valid”.

- Capture a textured static scene and a moving subject.
- Compare 12 MP, HDR 3×, LMC and stock.
- Tune Mertens weights, tone curve, shadow lift and highlight roll-off.
- Measure ghosting; improve alignment if needed.

**Done when:** documented comparison shows deeper shadows and unclipped highlights
with acceptable ghosting; latency and thermals recorded.

### Close the 50 MP JPEG question

- Document that the HAL returns 1920×1440 for an 8192×6144 JPEG reader.
- Keep the SOF guard; never save a fake 50 MP file.
- Use RAW 50 as the maximum-resolution path.
- Do not spend more time on vendor 50 MP JPEG without new evidence.

### Release hygiene

- Keep CI green; docs and changelog current.
- Tag alpha releases only after `check.ps1` and a device smoke test.

---

## Next (2–8 weeks)

### M7 — Live Photo

**Spec:** 4096×3072 4:3 HEVC 30 fps, 6.0 s (3 s before + 3 s after),
AAC 48 kHz stereo 256 kbps, JPEG 4096×3072, Motion Photo XMP v1.

- Add a video output surface to the Camera2 session.
- MediaCodec HEVC encoder + encoded-sample ring buffer.
- AudioRecord 48 kHz stereo + AAC encoder + audio ring.
- MediaMuxer; Motion Photo XMP assembly; optional GainMap.
- Verify with `ffprobe` and Google Photos.

**Done when:** one video track + one stereo audio track, 6.0 s ±50 ms, A/V sync
<50 ms, opens as Motion Photo.

### M8 — Video

- 4K 4:3 4096×3072 @30 fps.
- 4K 16:9 3840×2160 @60 fps.
- HEVC, stereo AAC, stabilization.
- 10-bit P010 where the encoder accepts it.
- 10-minute thermal/dropped-frame test.

**Done when:** stable recording, audio in sync, no thermal shutdown, documented
bitrate/size.

### UI and gallery

- MediaStore/gallery integration.
- Clear mode labels (50 MP JPEG unavailable → use R50).
- Animations, settings, permission flows.

---

## Later (2–6 months)

- Advanced HDR alignment: rotation/parallax, motion masks, optical flow.
- User-supplied LUT/color profiles (no vendor assets in public).
- 10-bit HDR video and HEIF/AV1 stills.
- Multi-device support (if requested).
- Beta → stable release, signing via CI secrets.
- Performance: zero-copy video, GPU processing, thermal budgeting.

---

## Priorities

| Priority | Work |
|---|---|
| P0 | DNG calibration, HDR quality, CI/release, no fake 50 MP |
| P1 | Live Photo, 4K video, stereo audio, gallery |
| P2 | Advanced alignment, 10-bit, LUTs, multi-device |

## Dependencies and risks

| Risk | Mitigation |
|---|---|
| HAL returns unexpected stream sizes | SOF/dimension guards, fallback, document |
| DNG color looks wrong | user-supplied calibration; compare in Lightroom |
| HDR ghosting | short bracket, NCC alignment, future motion masks |
| Live Photo A/V drift | shared monotonic clock, `ffprobe` verification |
| Thermal throttling at 4K | record temperature, limit clip length, test |
| Package identity conflicts | uninstall old clone; document restore path |
| Legal exposure | public repo only original code; private research archive |

## Release cadence

- Alpha: each verified increment, debug-signed.
- Beta: feature-complete for M7/M8, still pre-release.
- Stable: after DNG calibration, HDR quality and video are accepted.

See [BACKLOG.md](BACKLOG.md) for task-level detail and [RELEASING.md](RELEASING.md)
for the release process.
