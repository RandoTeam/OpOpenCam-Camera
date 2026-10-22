# CI

## Workflows

| Workflow | Trigger |
|---|---|
| `Rust Camera CI` | push to `main` / PR touching `rust_camera/**` |
| `Rust Camera Release` | tag `v*` |

Both live in `.github/workflows/` and run on GitHub-hosted `ubuntu-latest`
runners. A public repository has free GitHub Actions minutes.

## CI steps

`Rust Camera CI` runs:

- `cargo fmt --check`
- `cargo test` — host tests for capture modes, HDR fusion, DNG, Motion Photo
- `cargo clippy --target aarch64-linux-android -- -D warnings`
- `cargo build --release --target aarch64-linux-android`
- `python3 rust_camera/scripts/check_version.py`
- `./gradlew :app:assembleDebug`
- uploads the debug APK artifact

`Rust Camera Release` checks that the tag matches `VERSION`, builds
`app-release.apk`, writes a SHA-256 file and publishes a pre-release.

## Local verification

```powershell
.\rust_camera\scripts\check.ps1
```

The same suite passes locally (29 host tests at the time of writing).

## Release history

| Tag | Date | Notes |
|---|---|---|
| `v0.4.0-alpha.1` | 2026-10-22 | 4K 4:3 30 / 4K 16:9 60 video, stereo AAC; 25 MP closed |
| `v0.3.0-alpha.1` | 2026-10-22 | RAW 50 DNG, vendor identity, ~50 ms preview start |
| `v0.2.2-alpha.1` | 2026-10-22 | fix `libmediandk` startup crash |
| `v0.2.1-alpha.1` | 2026-10-22 | NCC HDR alignment |
| `v0.2.0-alpha.1` | 2026-10-22 | 50 MP/RAW/HDR/lens modes |
| `v0.1.0-alpha.1` | 2026-10-22 | first device-verified engine; debug-signed alpha |

## Signing

Alpha/beta APKs are debug-signed. A production release needs a keystore supplied
through GitHub Actions secrets; see [RELEASING.md](RELEASING.md).
