# OpOpenCam Camera

**A rootless, professional camera for the OnePlus 13 — Rust Camera2 engine, Jetpack Compose UI.**

[![Rust Camera CI](https://github.com/RandoTeam/OpOpenCam-Camera/actions/workflows/rust-camera-ci.yml/badge.svg?branch=main)](https://github.com/RandoTeam/OpOpenCam-Camera/actions/workflows/rust-camera-ci.yml)
[![Version](https://img.shields.io/badge/version-0.3.0--alpha.1-orange)](rust_camera/CHANGELOG.md)
[![Platform](https://img.shields.io/badge/platform-Android%2016%20%7C%20arm64-blue)](https://www.android.com/)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

OpOpenCam is a hybrid camera stack for the OnePlus 13 (`PJZ110`, Snapdragon 8 Elite,
Android 16). The camera session and image processing are written in **Rust**; the
interface is **Jetpack Compose**.

It is not another GCam port. The goal is a camera we control end to end: direct
Camera2 NDK access, detail-preserving multi-frame HDR, 50 MP RAW, and eventually a
Live Photo implementation with real stereo sound.

## Status

| Area | Status |
|---|---|
| Camera2 NDK preview | ✅ verified on device |
| 12 MP JPEG | ✅ 4096×3072 |
| RAW 12 / RAW 50 → DNG | ✅ 25 MB / 96 MB, DNG 1.4 |
| 50 MP JPEG | ❌ HAL returns 1920×1440; use RAW 50 |
| HDR 3× (NCC alignment + Mertens) | ✅ logical / ultra-wide / tele |
| Lens switching 0.6× / 1× / 3× | ✅ cameras 3 / 0 / 4 |
| Preview restart | ✅ ~50 ms |
| Live Photo / 4K video / stereo audio | 🚧 next milestone |
| Full stock/OPCameraPro parity | 🚧 mapped in [FEATURE_PARITY.md](docs/FEATURE_PARITY.md) |

Full details: [rust_camera/README.md](rust_camera/README.md),
[docs/STATUS.md](docs/STATUS.md) and
[docs/reports/LIVE_2026-10-22.md](docs/reports/LIVE_2026-10-22.md).

## Documentation

Start at [docs/README.md](docs/README.md). The most useful entry points:

| Document | What it answers |
|---|---|
| [PROJECT.md](docs/PROJECT.md) | Vision, goals, non-goals |
| [STATUS.md](docs/STATUS.md) | What is verified today |
| [FEATURE_PARITY.md](docs/FEATURE_PARITY.md) | Full stock + OPCameraPro feature map |
| [PIPELINE.md](docs/PIPELINE.md) | How a frame becomes a file |
| [ROADMAP.md](docs/ROADMAP.md) | Now / next / later milestones |
| [BACKLOG.md](docs/BACKLOG.md) | Prioritized tasks |
| [AGENTS.md](docs/AGENTS.md) | Rules for AI agents |
| [TESTING.md](docs/TESTING.md) | Test pyramid and evidence |
| [DECISIONS/](docs/DECISIONS/) | Why key choices were made |

## Quick start

Requirements: Rust 1.98+, Android SDK 36 / NDK 28.2, JDK 21.

```powershell
.\rust_camera\scripts\build.ps1
.\rust_camera\scripts\install.ps1
```

The APK installs as `com.oneplus.camera` (Chi HAL whitelist). Uninstall the old
patched clone first:

```powershell
adb uninstall com.oneplus.camera
```

## Repository layout

```
rust_camera/          Rust engine, Compose app, build scripts
  rust_core/          Camera2 NDK, capture modes, HDR, DNG, Motion Photo
  android_app/        Jetpack Compose shell
  scripts/            build / check / install
docs/                 project handbook, pipeline, roadmap, agent rules, ADRs
  DECISIONS/          architecture decision records
  reports/            live verification reports
CONTRIBUTING.md
PLAN.md               short pointer to docs/ROADMAP.md
THIRD_PARTY_NOTICES.md
TRADEMARKS.md
SECURITY.md
PRIVACY.md
```

## Public build scope

This repository contains **only original source code and documentation**. It does
not include stock APKs, extracted vendor libraries, Hasselblad LUTs, EEPROM or
calibration assets, any LMC-derived modules, or GPL `OPCameraPro` code. Those remain
in a private research archive and are not redistributed.

The public build uses a generic sRGB D65 matrix for DNG output; per-sensor
calibration is intentionally not bundled. See [TRADEMARKS.md](TRADEMARKS.md).

## Roadmap

The canonical plan is [docs/ROADMAP.md](docs/ROADMAP.md), with task detail in
[docs/BACKLOG.md](docs/BACKLOG.md):

1. DNG colour calibration with user-supplied profiles.
2. HDR quality on textured/moving scenes.
3. 25 MP probe; 50 MP JPEG remains closed, RAW 50 is the max path.
4. Video core: 4K 4:3 30 / 4K 16:9 60, 1080p 120, stereo audio.
5. Live Photo: 6 s (3 s before + 3 s after), 4:3, stereo AAC.
6. Pro mode, portrait/macro/zoom, LUTs/watermarks, gallery/editor.
7. 10-bit P010/HDR10, Ultra HDR stills; no Dolby Vision.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). AI agents must also read
[docs/AGENTS.md](docs/AGENTS.md). Use Conventional Commits, keep PRs focused, and
never commit personal photos, device serials or keystores.

## License

MIT for the original code. Third-party dependencies and trademarks are covered by
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [TRADEMARKS.md](TRADEMARKS.md).
