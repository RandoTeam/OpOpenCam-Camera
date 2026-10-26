# Agent Instructions

The canonical agent handbook is **[docs/AGENTS.md](docs/AGENTS.md)**. Read it
before any task, then read [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md),
[docs/STATUS.md](docs/STATUS.md), [docs/ROADMAP.md](docs/ROADMAP.md) and
[docs/PIPELINE.md](docs/PIPELINE.md).

The current UI/capture contract is in
**[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)**: one portrait-coordinate camera
canvas rotated with the device, selective glyph counter-rotation, letterboxed
preview and a background capture queue. Do not reintroduce a fixed landscape
layout or per-orientation reflow.

## Non-negotiables

1. No secrets, keystores, tokens, passwords, device serials or personal photos.
2. No stock APK, vendor libraries, LUTs, EEPROM/calibration, LMC-derived code or
   GPL `OPCameraPro` code in the public repo.
3. No fake success: camera features need device evidence (logs, dimensions, files).
   Vendor-only features (50 MP JPEG, 4K120, front 4K, Dolby, branded AI) stay
   documented as not portable.
4. Branch → PR → green CI → squash merge. Never commit directly to `main`.
5. Update docs, `STATUS.md`, `ROADMAP.md`/`BACKLOG.md` and `CHANGELOG.md` in the
   same PR as the code.
6. Keep version files consistent: `rust_camera/VERSION`,
   `rust_camera/rust_core/Cargo.toml`, `rust_camera/android_app/app/build.gradle`.
7. Ask before changing package identity, public visibility, license, git history,
   or uninstalling/disabling user apps.
8. Use `.\rust_camera\scripts\check.ps1` and the device procedure in
   [docs/DEVICE_TEST_PLAN.md](docs/DEVICE_TEST_PLAN.md).

## Current limitations to remember

- 50 MP JPEG is unavailable; HAL returns 1920×1440. Use RAW 50.
- DNG color is generic until user-supplied calibration.
- Live Photo is not implemented. 4K video is an experimental path; stereo AAC is
  verified, but video stays frozen until the photo core is accepted.
- The current UI is still `sensorLandscape`; UI-302 replaces it with the stock
  rotating-canvas model. Portrait layout is not implemented yet.
- CAP-301 is a prototype in `feat/background-capture`; it is not merged and the
  landscape `PHOTO/VIDEO` guard is still missing.
- Alpha APKs are debug-signed.
