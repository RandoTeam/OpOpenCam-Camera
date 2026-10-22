# HDR

## Goal

Deep shadows and unclipped highlights with natural texture — not a flat “HDR look”
and not a neural watercolor.

## Current pipeline (verified on device)

1. Three YUV_420_888 frames at 4096×3072 are requested with
   `AE_EXPOSURE_COMPENSATION = -6 / 0 / +6` (1/3 EV units, i.e. −2/0/+2 EV).
2. Each frame is converted to RGB with BT.601 coefficients.
3. Frames are aligned to the middle bracket with normalised cross-correlation on
   luma (coarse-to-fine, translation only).
4. Mertens-style weights are computed per pixel:
   `contrast × saturation × well-exposedness`.
5. The fused image is rotated 90° to portrait and encoded as JPEG (quality 92).

The engine returns a JPEG 3072×4096. Capture + fusion took ~2.3 s on the OnePlus 13.

## What is still weak

- Translation-only alignment: rotation, parallax and moving subjects can ghost.
- Tone mapping is conservative; the “professional” look needs tuning on real scenes.
- The bracket is AE-compensated; manual exposure/ISO brackets may give better control.
- No motion masks or optical flow yet.

## Next experiments

| Experiment | Purpose |
|---|---|
| Textured static scene | Measure detail/DR vs single frame |
| Moving subject | Quantify ghosting; decide on motion masks |
| 5-frame bracket | More headroom vs capture time |
| Manual exposure bracket | Stable EV steps independent of AE |
| Tone-curve variants | Natural iPhone-like look without clipping |
| HAL `EnableSHDR`/`EnableQHDR` | In-sensor HDR as an alternative path |

## Metrics

- highlight clipping percentage;
- shadow noise / SNR in dark patches;
- detail retention (no halo/oversharpening);
- ghosting on a moving subject;
- capture + fusion latency;
- thermal behavior.

## Test protocol

1. Use a textured static scene (bookshelf, text, fabric) with a bright window.
2. Capture 12 MP and HDR back-to-back.
3. Capture a scene with a moving person/vehicle.
4. Pull files; record dimensions and sizes.
5. Compare shadows/highlights and ghosting side by side.
6. Repeat on logical, ultra-wide and tele.

## Acceptance

HDR is accepted for beta when a documented comparison shows better shadow/highlight
detail than a single frame, with no obvious halos and acceptable ghosting on static
scenes. Moving subjects may remain a known limitation until motion masks land.

See [PIPELINE.md](PIPELINE.md), [QUALITY.md](QUALITY.md), [BACKLOG.md](BACKLOG.md).
