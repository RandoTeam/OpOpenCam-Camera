# ADR 0007 — Clean-room feature parity with OPCameraPro

**Status:** accepted

## Context

The `com.tlsu.opluscamerapro` module is a GPL-3.0 Xposed project that unlocks
vendor features inside the stock `com.oplus.camera` app. Its public source mirror
contains code, settings and vendor-tag manipulation. Our project is MIT, rootless,
and must not copy GPL code or vendor assets into the public repository.

At the same time, the product goal is to cover the full stock/module feature
surface wherever a clean Android/Camera2 implementation exists.

## Decision

- Treat the module as a **feature-discovery reference only**.
- Implement behaviour from public descriptions, observed Camera2 capabilities and
  our own tests; never copy module code, resources, LUTs or vendor-tag dumps.
- Keep the public repo MIT and rootless.
- Build clean equivalents where possible:
  - Pro mode instead of Master Mode 2.0;
  - original filters/LUT engine instead of branded looks;
  - Android gainmap/Ultra HDR instead of vendor ProXDR;
  - HEVC Main10 + HDR10/HLG instead of Dolby Vision;
  - own watermark editor instead of Hasselblad/GR/LUMO marks.
- Document vendor-only features as **not portable** in
  [FEATURE_PARITY.md](../FEATURE_PARITY.md); do not promise them in UI.

## Alternatives

1. **Fork/copy the GPL module.** Rejected: licence incompatibility, root/LSPosed
   dependency, vendor hooks, and no clean public build.
2. **Ignore the module.** Rejected: the user wants full feature coverage.
3. **Reimplement every vendor algorithm.** Rejected: proprietary models, licences,
   trademarks and no clean data.

## Consequences

- Feature coverage is broad but not a pixel-for-pixel clone.
- Some stock/module features remain closed (50 MP JPEG, 4K120, front 4K, Dolby,
  branded AI/gallery).
- Every parity feature needs its own evidence and tests.
- The public repo stays legally clean and MIT-compatible.
