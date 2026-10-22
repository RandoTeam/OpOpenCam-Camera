# ADR 0002 — Package identity `com.oneplus.camera`

**Status:** accepted; verified on device

## Context

The OnePlus Chi HAL exposes vendor features through a package whitelist. With
`com.opopencam.camera`, physical cameras worked but 50 MP RAW fell back to 12 MP.
The whitelist includes `com.oneplus.camera`.

## Decision

- `applicationId = "com.oneplus.camera"`.
- The Kotlin/JNI class package stays `com.opopencam.camera`, so JNI symbols do not
  change.
- Use vendor session parameters (`ultra.resolution.mode`, `EnableIdealRAW`, package
  name) for 50 MP modes.
- Document that the old patched clone with the same package must be uninstalled.

## Alternatives

1. Keep `com.opopencam.camera`: safe, but no RAW 50.
2. Spoof the package name only in a vendor tag: works possibly, but is less honest
   than using the actual application ID.
3. Use the stock `com.oplus.camera`: not possible without the vendor key/system
   signature.

## Consequences

- RAW 50 (8192×6144 DNG) works.
- 50 MP JPEG still returns 1920×1440; identity alone is not enough.
- Package conflict with the old clone; one-time uninstall.
- Trademark/interoperability note is required; see `TRADEMARKS.md`.
