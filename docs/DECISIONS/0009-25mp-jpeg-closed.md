# ADR 0009 — 25 MP JPEG closed by evidence

**Status:** accepted

## Context

The `com.tlsu.opluscamerapro` module exposes a vendor “25 MP Turbo RAW” path with
an advertised 5888×4416 output. Camera2 static metadata on the OnePlus 13 does not
advertise a 25 MP stream, but the 50 MP RAW path succeeds through a vendor session
mode, so a controlled probe was justified.

## Probe

- Build: temporary branch from `0.3.0-alpha.1`.
- Mode: 5888×4416 JPEG reader.
- Session parameters: the same vendor resolution mode used by RAW 50
  (`ultra.resolution.mode`, `EnableIdealRAW`, package name).
- Result:
  - session creation: accepted;
  - preview: active;
  - capture: HAL returned a **1920×1440** JPEG;
  - the JPEG SOF guard rejected the mismatch; no file was written.

## Decision

- Close 25 MP. Do not expose a 25 MP chip or label.
- Use 4096×3072 JPEG or 8192×6144 RAW 50.
- Never upscale a 1920×1440 JPEG and call it 25 MP.
- Re-open only with new HAL/ROM evidence (a real 5888×4416 file).

## Consequences

- The UI has no 25 MP mode; the 12/50/RAW/R50/HDR modes remain.
- `STILL-101` is closed in the backlog.
- 50 MP JPEG remains closed separately; RAW 50 is the maximum-resolution path.
