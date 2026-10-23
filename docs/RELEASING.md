# Releasing

## Versioning

Semantic Versioning with alpha/beta suffixes:

```
0.3.0-alpha.1   first full still/RAW/HDR alpha (current)
0.4.0-beta.1    video + Live Photo + Pro mode beta
1.0.0           stable parity release
```

Version sources that must match:

- `rust_camera/VERSION`
- `rust_camera/rust_core/Cargo.toml` → `version`
- `rust_camera/android_app/app/build.gradle` → `versionName` / `versionCode`

Run `python rust_camera/scripts/check_version.py` before tagging.

## Release checklist

1. Update `VERSION`, `Cargo.toml`, `app/build.gradle`.
2. Update `CHANGELOG.md` with the new section.
3. Run:
   ```powershell
   .\rust_camera\scripts\check.ps1
   ```
   This runs `cargo fmt --check`, host tests, Clippy (`-D warnings`) and the
   Android debug build.
4. Commit with Conventional Commits, e.g.
   `chore(release): v0.2.0-beta.1`.
5. Tag and push:
   ```bash
   git tag v0.2.0-beta.1
   git push origin v0.2.0-beta.1
   ```
6. The `Rust Camera Release` workflow builds the APK and publishes a **pre-release**
   with auto-generated notes and a SHA-256 file.

## Merge commits and email privacy

GitHub’s merge button can author a squash/rebase commit with the account’s primary
email, even when the PR’s commits use a generic identity. Before merging, verify
the generated commit author; if it is personal, merge locally with the project
identity and push `main` instead. Never publish a personal email in a merge commit.

## Signing

Alpha/beta APKs are **debug-signed** and clearly marked as such. The CI-generated
debug key is not the same as a developer machine's debug key, so installing a
release APK over a local debug build requires uninstalling the old package first.
A production
release requires a dedicated keystore configured through GitHub Actions secrets:

- `ANDROID_KEYSTORE_BASE64`
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`

Do not commit keystores or passwords.

## Branching

- `main` is the integration branch.
- Feature work goes through `feat/<scope>` branches and a PR.
- CI must be green before merge.
- Hotfixes use `fix/<scope>`.

## Release notes template

```markdown
## What changed
- ...

## Verified on
- OnePlus 13 PJZ110, Android 16, build ...

## Known limitations
- ...

## Install
Download the APK, `adb install -r -g app-release.apk`, grant Camera/Audio.
This is a debug-signed alpha; do not use it as a daily camera yet.
```
