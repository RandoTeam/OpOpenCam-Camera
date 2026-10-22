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

Full details: [rust_camera/README.md](rust_camera/README.md) and
[rust_camera/docs/LIVE_2026-10-22.md](rust_camera/docs/LIVE_2026-10-22.md).

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
rust_camera/          Rust engine, Compose app, docs, build scripts
  rust_core/          Camera2 NDK, capture modes, HDR, DNG, Motion Photo
  android_app/        Jetpack Compose shell
  docs/               architecture, roadmap, quality, Motion Photo, CI
  scripts/            build / check / install
PLAN.md               next milestones and publication checklist
THIRD_PARTY_NOTICES.md
TRADEMARKS.md
SECURITY.md
PRIVACY.md
```

## Public build scope

This repository contains **only original source code and documentation**. It does
not include stock APKs, extracted vendor libraries, Hasselblad LUTs, EEPROM or
calibration assets, or any LMC-derived modules. Those remain in a private research
archive and are not redistributed.

The public build uses a generic sRGB D65 matrix for DNG output; per-sensor
calibration is intentionally not bundled. See [TRADEMARKS.md](TRADEMARKS.md).

## Roadmap

The next milestones are in [PLAN.md](PLAN.md):

1. DNG colour calibration with user-supplied profiles.
2. HDR quality on textured/moving scenes.
3. Live Photo: 6 s (3 s before + 3 s after), 4:3, stereo AAC.
4. 4K 4:3 30 fps / 4K 16:9 60 fps video.
5. UI/UX and beta releases.

## Contributing

See [rust_camera/CONTRIBUTING.md](rust_camera/CONTRIBUTING.md). Use Conventional
Commits, keep PRs focused, and never commit personal photos, device serials or
keystores.

## License

MIT for the original code. Third-party dependencies and trademarks are covered by
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [TRADEMARKS.md](TRADEMARKS.md).
