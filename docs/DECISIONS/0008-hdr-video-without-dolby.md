# ADR 0008 — HDR video without Dolby Vision

**Status:** accepted

## Context

The stock camera and the module expose Dolby Vision (30/60/120 fps, tele/front).
Dolby Vision requires a Dolby licence and a vendor encoder/display pipeline; the
module reaches it by hooking the stock app. Our app is rootless and MIT.

The device HAL advertises P010 (10-bit YUV) and `DYNAMIC_RANGE_TEN_BIT` on the rear
cameras, plus 4K 4:3 30 and 4K 16:9 60.

## Decision

- Do not implement or advertise Dolby Vision.
- Target **HEVC Main10 + P010**, with **HDR10 or HLG** metadata.
- Keep SDR as the default; HDR is an explicit mode with clear compatibility notes.
- Verify bit depth, colour profile and metadata with a real file.
- If the encoder rejects 10-bit, fall back to 8-bit H.264/HEVC and say so.

## Alternatives

1. Hook the stock Dolby pipeline: root/vendor, not portable.
2. Ship 8-bit “HDR” with no metadata: misleading; rejected.
3. Ignore HDR video: loses a real capability; rejected.

## Consequences

- No Dolby Vision badge or compatibility claim.
- HDR10/HLG files play on many devices but not all; UI must explain.
- P010 and 10-bit encoding are first-class test targets.
- The same rule applies to still HDR: use Android gainmap/Ultra HDR, not vendor
  ProXDR.
