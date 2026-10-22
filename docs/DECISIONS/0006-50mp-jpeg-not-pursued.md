# ADR 0006 — 50 MP JPEG closed; use RAW 50

**Status:** accepted; revisit only with new evidence

## Context

The device advertises 8192×6144 in `availableRawSizes`. A 8192×6144 JPEG reader
receives a **1920×1440** JPEG from the HAL, even with:

- `com.oneplus.camera` package identity;
- vendor session parameters (`ultra.resolution.mode`, `EnableIdealRAW`, package
  name);
- a 50 MP JPEG reader.

The stock camera’s 50 MP JPEG path is system/vendor-only.

## Decision

- Parse the JPEG SOF and reject any JPEG whose dimensions do not match the request.
- Do not save or upscale the 1920×1440 result as “50 MP”.
- Use **RAW 50** (8192×6144 DNG) as the maximum-resolution path.
- Document the limitation in README/STATUS/CAPTURE_MODES.

## Alternatives

1. Upscale the 1920×1440 JPEG: fake detail; rejected.
2. Keep chasing vendor tags/system signatures: no evidence it is reachable rootless.
3. Remove the 50 MP mode entirely: loses the RAW 50 path.

## Consequences

- No 50 MP JPEG on this device.
- RAW 50 works and is higher quality.
- The UI shows “50 MP JPEG unavailable — use R50”.
- Future devices/ROMs may differ; re-test before changing the decision.
