# ADR 0005 — No vendor assets or LMC modules in the public build

**Status:** accepted

## Context

The research archive contains stock APK/smali, vendor libraries, Hasselblad LUTs,
EEPROM/calibration and LMC analysis. Some code was explicitly derived from LMC
configuration (`lmc_tone_curve.rs`, `ms_lut_parser.rs`). Redistributing these
creates legal risk.

## Decision

- Public repository contains only original code and generic formats.
- Remove `lmc_tone_curve.rs` and `ms_lut_parser.rs` from the public build.
- Replace the vendor DNG ColorMatrix with a generic sRGB D65 matrix.
- Do not include LUT/EEPROM/calibration/stock APK/vendor `.so` files.
- User-supplied calibration/LUTs may be supported later, outside the repo.

## Alternatives

1. Publish with a disclaimer: still redistributes derived/proprietary material.
2. Keep the modules but mark “research only”: unclear and risky.
3. Keep the entire project private: safe but not open.

## Consequences

- Public build loses the vendor-specific look and LUT parsing.
- Safer legal surface; MIT can apply to the original code.
- Private research can continue with LMC/vendor analysis.
- Future color work uses user-supplied profiles, not bundled assets.
