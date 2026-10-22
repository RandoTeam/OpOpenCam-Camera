# Architecture Decision Records

Each ADR captures a decision that is expensive to reverse. Format:
Context → Decision → Alternatives → Consequences → Status.

| ADR | Decision |
|---|---|
| [0001](0001-hybrid-rust-compose.md) | Hybrid Rust engine + Compose UI |
| [0002](0002-package-identity.md) | Use `com.oneplus.camera` for HAL access |
| [0003](0003-public-private-separation.md) | Separate public camera repo from private research |
| [0004](0004-hdr-bracketing-and-alignment.md) | HDR bracketing + NCC alignment + Mertens |
| [0005](0005-no-vendor-assets-in-public.md) | No vendor assets or LMC modules in public |
| [0006](0006-50mp-jpeg-not-pursued.md) | 50 MP JPEG closed; use RAW 50 |
| [0007](0007-clean-room-feature-parity.md) | Clean-room parity; no GPL/vendor code |
| [0008](0008-hdr-video-without-dolby.md) | HDR10/HLG instead of Dolby Vision |
| [0009](0009-25mp-jpeg-closed.md) | 25 MP JPEG closed by evidence |
| [0010](0010-video-encoder-ownership.md) | Rust Camera2 + Kotlin MediaCodec video |

Add a new ADR for changes to architecture, package identity, licensing, public
scope, or a hard-to-reverse pipeline choice. Update the index.
