# Third-Party Notices

OpOpenCam original code is licensed under the MIT License (see `LICENSE`).

## Runtime and build dependencies

| Component | License |
|---|---|
| AndroidX / Jetpack Compose / Kotlin | Apache License 2.0 |
| Android NDK (Camera2 NDK, media NDK) | Apache License 2.0 |
| Gradle wrapper | Apache License 2.0 |
| Rust crates: `jni`, `ndk`, `ndk-sys`, `log`, `android_logger`, `parking_lot`, `jpeg-encoder`, `thiserror`, `libc` | MIT and/or Apache License 2.0 |

The authoritative dependency list is `rust_camera/rust_core/Cargo.lock`. For a
full machine-readable audit, run:

```bash
cargo install cargo-license
cd rust_camera/rust_core
cargo license
```

## Not included

This repository does **not** contain:

- the OnePlus/Oppo stock camera APK or its decompiled resources;
- extracted vendor libraries from `/odm`, `/product` or `/system_ext`;
- Hasselblad/Leica LUT files, EEPROM dumps or per-device calibration data;
- LMC/GCam APKs or LMC-derived modules;
- device dumps, screenshots or personal media.

Those materials remain in a private research archive and are not redistributed.

## Trademarks

See `TRADEMARKS.md`. No third-party trademark, logo or proprietary asset is
licensed by this repository.
