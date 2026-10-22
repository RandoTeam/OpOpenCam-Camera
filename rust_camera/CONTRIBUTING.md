# Contributing

Thanks for helping build a real camera engine.

## Setup

- Rust 1.98+ and the `aarch64-linux-android` target.
- Android SDK 36 + NDK 28.2.
- JDK 21.
- Gradle wrapper is included.

```powershell
rustup target add aarch64-linux-android
.\rust_camera\scripts\check.ps1
```

## Checks

`scripts/check.ps1` runs:

```
cargo fmt --check
cargo test
cargo clippy --target aarch64-linux-android -- -D warnings
gradlew :app:assembleDebug
```

All four must pass before a PR.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(camera): add 50 MP JPEG reader
fix(jni): release the preview window on session failure
docs(motion-photo): document the 6 s ring buffer
chore(release): v0.1.0-alpha.1
```

Keep commits focused. Do not mix the stock-APK patching work with the Rust camera
project.

## Branches and PRs

- `feat/<scope>`, `fix/<scope>`, `docs/<scope>`, `chore/<scope>`.
- PR body: what changed, how it was verified, known limitations.
- CI must be green.
- No personal photos, device dumps or APKs in git.

## Code style

- Rust: `rustfmt`, Clippy clean, module-level docs for public APIs.
- Kotlin: official style, KDoc for composables and JNI declarations.
- Comments explain *why*, not *what*.

## Device testing

Live tests require a OnePlus 13. Follow `DEVICE_TEST_PLAN.md` and attach the
resulting summary (not the raw personal media) to the PR.

## Security and privacy

- Never commit keystores, tokens or personal media.
- Raw device captures stay in `test_reports/device_capture_*` (ignored).
