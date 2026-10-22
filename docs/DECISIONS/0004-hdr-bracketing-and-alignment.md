# ADR 0004 — HDR bracketing, NCC alignment and Mertens fusion

**Status:** accepted; quality tuning ongoing

## Context

A single frame clips highlights or crushes shadows. OPlus APS/neural processing
loses texture. We need a controllable, testable multi-frame pipeline.

## Decision

- Capture 3 YUV frames at 4096×3072 with AE compensation −2/0/+2 EV.
- Convert to RGB (BT.601).
- Align frames to the middle bracket with normalised cross-correlation on luma
  (coarse-to-fine, translation only).
- Fuse with Mertens weights: contrast × saturation × well-exposedness.
- Rotate to portrait and encode JPEG quality 92.

## Alternatives

1. Single frame + tone curve: faster, but limited dynamic range.
2. Rely only on HAL `EnableSHDR`/`EnableQHDR`: less control, vendor-dependent.
3. RAW fusion/demosaic: best potential, much more work.
4. 5-frame bracket: more headroom, longer capture.

## Consequences

- Verified pipeline: 3 frames → JPEG 3072×4096 in ~2.3 s.
- Translation-only alignment can ghost on rotation/parallax/moving subjects.
- Tone mapping and weights still need tuning on textured scenes.
- Future: motion masks, optical flow, manual exposure bracket, 10-bit output.
