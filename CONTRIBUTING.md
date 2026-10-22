# Contributing

Thanks for helping build a real camera engine.

> AI agents must also read [docs/AGENTS.md](docs/AGENTS.md). It contains the
> non-negotiable rules and the Definition of Done.

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

`rust_camera/scripts/check.ps1` runs:

```
cargo fmt --check
cargo test
cargo clippy --target aarch64-linux-android -- -D warnings
python rust_camera/scripts/check_version.py
gradlew :app:assembleDebug
```

All must pass before a PR. CI runs the same suite on GitHub-hosted runners.

## Where to start

1. [docs/PROJECT.md](docs/PROJECT.md) — vision and scope.
2. [docs/STATUS.md](docs/STATUS.md) — what is verified.
3. [docs/PIPELINE.md](docs/PIPELINE.md) — how the camera works.
4. [docs/ROADMAP.md](docs/ROADMAP.md) / [docs/BACKLOG.md](docs/BACKLOG.md) — what
   to do next.
5. [docs/TESTING.md](docs/TESTING.md) — how to prove it.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(camera): add 50 MP RAW DNG
fix(jni): link libmediandk for AImageReader
docs(motion-photo): document the 6 s ring buffer
chore(release): v0.3.0-alpha.1
```

Keep commits focused. Do not mix stock-APK research with the public camera project.

## Branches and PRs

- `feat/<scope>`, `fix/<scope>`, `docs/<scope>`, `ci/<scope>`, `chore/<scope>`.
- PR body: what changed, how it was verified, known limitations.
- CI must be green; squash merge.
- One logical change per PR.

## Code style

- Rust: `rustfmt`, Clippy clean, module-level docs for public APIs, `Result` errors.
- Kotlin: official style, KDoc for composables and JNI declarations.
- Comments explain *why* (HAL behavior, threading, fallbacks), not *what*.
- Never block the UI thread on JNI.

## Device testing

Live tests require a OnePlus 13. Follow [docs/DEVICE_TEST_PLAN.md](docs/DEVICE_TEST_PLAN.md)
and attach a sanitized summary (not raw personal media) to the PR. Camera changes
without device evidence are “validation pending”, not done.

## Security, privacy and legal

- Never commit keystores, tokens, private keys, passwords, device serials, personal
  photos or raw device dumps.
- Never commit stock APKs, vendor libraries, LUTs, EEPROM/calibration data or
  LMC/GCam-derived modules.
- The public repo is `RandoTeam/OpOpenCam-Camera`; the old `OpOpenCam` repo is a
  private research archive.
- Original code is MIT. See [LICENSE](LICENSE),
  [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [TRADEMARKS.md](TRADEMARKS.md).

## Definition of Done

See [docs/AGENTS.md](docs/AGENTS.md#4-definition-of-done). In short: code + tests +
device evidence + docs + changelog + green CI, with no secrets/PII/vendor assets.
