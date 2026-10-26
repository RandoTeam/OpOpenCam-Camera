# Agent Rules

These rules apply to every AI agent (and human) working on OpOpenCam Camera. They
exist to keep the project honest, legally clean, reproducible and fast to review.

## 1. Mission and source of truth

Build a rootless, professional OnePlus 13 camera with a Rust Camera2 engine and a
Compose UI. Public source only; vendor research stays private.

Read before changing anything:

1. [STATUS.md](STATUS.md) — what is actually verified.
2. [DEVELOPMENT.md](DEVELOPMENT.md) — build, device workflow and current UI/capture
   model.
3. [ROADMAP.md](ROADMAP.md) — what to do next.
4. [PIPELINE.md](PIPELINE.md) / [ARCHITECTURE.md](ARCHITECTURE.md) — how it works.
5. [TESTING.md](TESTING.md) — how to prove it.
6. [DECISIONS/](DECISIONS/) — why earlier choices were made.

If a document disagrees with reality, update the document in the same PR.

## 2. Non-negotiable rules

### Legal and privacy

1. **Never commit secrets.** No keystores, private keys, tokens, passwords, API
   keys, `.env` files or signing material.
2. **Never commit personal data.** No device serials, account names, email
   addresses, personal photos, raw device dumps or UI screenshots with personal
   content.
3. **Never commit vendor/proprietary assets.** No stock APK, smali, vendor `.so`,
   LUT, EEPROM, calibration data, LMC/GCam APK or LMC-derived module.
4. **Public/private separation.** The public repo is `OpOpenCam-Camera`; the old
   `OpOpenCam` repository is a private research archive and must not be published.
5. **License changes require human approval.** MIT is the current license; do not
   relicense or add incompatible assets.

### Feature parity and clean-room work

- **Clean-room only.** `com.tlsu.opluscamerapro` is GPL-3.0 and hooks the stock
  app. Never copy its code, resources, vendor tags dumps or assets into this MIT
  repository. Use public feature descriptions, observed behaviour and your own
  tests.
- **No branded lookalikes.** Do not reproduce Hasselblad/GR/LUMO/XPAN marks,
  branded filters (Jiang Wen, Jia Zhangke, etc.) or vendor LUTs. Build original
  looks and allow user-supplied LUTs.
- **Scope honesty.** Vendor-only features (50 MP JPEG, 4K120, front 4K, Dolby
  Vision, AI super-res, vendor AI gallery) are documented as not portable unless
  new evidence proves a clean path. Do not promise them in UI or release notes.

### Truthfulness

6. **No fake success.** A feature is “done” only after live device evidence:
   logs, output files, dimensions, `ffprobe`/DNG inspection.
7. **Mark partial work.** Use “implemented, validation pending” in `STATUS.md`,
   README and changelog.
8. **Never invent hardware values.** Use `dumpsys media.camera`, `/odm` configs,
   or measured output. If unknown, say unknown and add a test.
9. **Do not hide failures.** A HAL returning 1920×1440 for a 50 MP request is a
   documented limitation, not a success.

### Process

10. **No direct commits to `main`.** Use `feat/…`, `fix/…`, `docs/…`, `ci/…`
    branches and a PR.
11. **CI must be green** before merge (format, tests, Clippy, build, version).
12. **One logical change per PR.** Do not mix stock-APK research with camera code.
13. **Update docs and changelog** for every user-visible change.
14. **Version consistency:** `rust_camera/VERSION`, `rust_camera/rust_core/Cargo.toml`
    and `rust_camera/android_app/app/build.gradle` must match.
15. **Ask before destructive actions:** package identity, public visibility,
    license, history rewrite, uninstalling user apps, disabling the stock camera,
    root/flashing.

## 3. Workflow

### 3.0 Git identity and merge safety

- Before committing, set the local identity to the project identity:
  `RandoTeam <RandoTeam@users.noreply.github.com>`.
- **Do not use the GitHub “Squash and merge” or “Rebase and merge” button unless
  the account’s email privacy is enabled and the resulting commit is verified.**
  GitHub can author the merge commit with the account’s primary personal email
  even when every branch commit is generic.
- Preferred: merge locally with the generic identity, push `main`, then close the
  PR with a note that it was merged manually. Verify `git log --format='%ae %ce'`
  before pushing.
- If a personal email ever reaches a public ref, stop, rewrite the ref, and treat
  it as a PII incident.

### 3.1 Plan

- Read `STATUS`, `ROADMAP`, `PIPELINE`, and the relevant ADR.
- Write a short plan in the PR description: goal, approach, risks, test plan.
- Pick the smallest increment that produces evidence.

### 3.2 Branch and implement

```bash
git switch -c feat/<scope> origin/main
```

- Keep changes focused.
- Add comments explaining *why* (camera/HAL decisions, fallbacks, threading).
- Rust: `Result` errors, no panics in camera paths, no blocking UI calls.
- Kotlin: KDoc for composables/JNI; never call blocking JNI on the UI thread.
- Prefer reusable modules and pure functions with host tests.

### 3.3 Test

Run:

```powershell
.\rust_camera\scripts\check.ps1
```

This covers `cargo fmt --check`, host tests, Clippy `-D warnings`, version
consistency and the Android debug build.

If camera behavior changed, run the device procedure in
[DEVICE_TEST_PLAN.md](DEVICE_TEST_PLAN.md) and attach evidence (not personal media).

### 3.4 Document

Update, as applicable:

- `docs/STATUS.md` — verified/partial/known issues.
- `docs/ROADMAP.md` / `docs/BACKLOG.md` — status and priorities.
- `docs/PIPELINE.md` — data flow or module changes.
- `rust_camera/CHANGELOG.md` — user-visible changes.
- `docs/DECISIONS/` — architecture/identity/licensing decisions.

### 3.5 Commit

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(camera): add 50 MP RAW DNG
fix(jni): link libmediandk for AImageReader
docs(agents): define evidence requirements
ci: install Android SDK/NDK non-interactively
chore(release): v0.3.0-alpha.1
```

One logical change per commit. Do not commit build artifacts (`target/`, `build/`,
`.gradle/`, `jniLibs/*.so`, `local.properties`, `.cargo/config.toml`).

### 3.6 Pull request

Include:

- Summary and motivation.
- What was verified (commands, device, output dimensions).
- What is **not** verified / known limitations.
- Risk/rollback.
- Linked issue/milestone.

Merge squash after CI is green. Do not force-push `main`.

### 3.7 Release

Follow [RELEASING.md](RELEASING.md):

1. Bump version and changelog.
2. Run `check.ps1`.
3. Tag `vX.Y.Z-alpha.N`.
4. The release workflow builds the APK, SHA-256 and pre-release.
5. Verify the release asset and version.

## 4. Definition of Done

A task is done when all are true:

- [ ] Code compiles; comments explain non-obvious HAL/threading decisions.
- [ ] Host tests and/or device tests cover the change.
- [ ] `check.ps1` passes; CI is green.
- [ ] Device evidence exists for camera behavior (dimensions, format, logs).
- [ ] `STATUS.md`, `ROADMAP.md`/`BACKLOG.md`, `CHANGELOG.md` are updated.
- [ ] `FEATURE_PARITY.md` status is updated for stock/module features.
- [ ] No secrets, PII, vendor assets, GPL module code or build artifacts.
- [ ] Version files match (if user-visible).
- [ ] Known limitations are stated honestly.

## 5. Camera-specific rules

- **Photo core comes first.** Do not expand video, 5K/6K/8K or Live Photo until
  photo capture speed, MediaStore saving, quality and smooth zoom are accepted.
- **Test every photo feature at all relevant lenses/zoom stops**, not only 1×:
  0.6/1/2/3/6, rear and front where applicable.
- **Use `SelfTestActivity` for the matrix** instead of repeated manual taps. Read
  `files/selftest/report.jsonl`; release builds must refuse to start it.
- **Do not dismiss a mode after one API path.** If JPEG fails, test RAW/vendor
  variants before closing; document exactly what was tried.

- **50 MP JPEG is unavailable** on this device; the HAL returns 1920×1440. Never
  “fix” it by renaming or resizing. Use RAW 50 or document the failure.
- **RAW 50 requires `com.oneplus.camera`** and vendor session parameters. Do not
  remove the package identity without a replacement test.
- **DNG calibration is user-supplied.** Do not bundle vendor ColorMatrix, LUT or
  EEPROM data in the public repo.
- **HDR evidence** must include a textured scene, not a blurry close-up.
- **25 MP is closed by evidence.** Session accepts 5888×4416 but capture returns
  1920×1440; do not re-open it without new HAL evidence. No upscale.
- **4K120, front 4K and Dolby Vision are not portable** on this device/module
  evidence. Do not add UI that implies they work.
- **10-bit/HDR:** verify the actual bit depth, colour profile and metadata with a
  file; “HDR” in the UI without metadata is a failure.
- **Branded looks:** no Hasselblad/GR/LUMO/XPAN marks or branded filters; use
  original designs and user-supplied LUTs.
- **Lens switching** is a session restart in the current implementation; do not
  claim seamless zoom until measured.
- **Thermals:** long HDR/RAW/video tests must record device temperature and
  dropped frames.
- **Parity matrix:** when a stock/module feature is implemented or closed, update
  [FEATURE_PARITY.md](FEATURE_PARITY.md) and the matching backlog ID.
- **Stock camera:** if you disable it for a test, re-enable it and record the
  command in the PR.

## 5.1 Current UI and capture rules (2026-10-26)

These are binding until the related backlog items are closed.

### UI parity with the stock camera

- The stock camera uses **one layout in phone coordinates**, not two reflowed
  layouts. Rotating the landscape reference by 90 degrees maps every control onto
  the portrait reference.
- Implement the screen as a portrait-coordinate canvas (`1080x2376`) rotated by
  the display rotation. Control positions stay fixed relative to the phone; do
  not move them to other edges when the device turns.
- Counter-rotate only glyphs that must stay readable: settings icons, zoom
  numbers, shutter/flip/gallery, quick toggles. Do **not** counter-rotate the
  `PHOTO/VIDEO/PORTRAIT/MASTER` labels; they run along the rail.
- Fit the preview to the sensor aspect, never stretch it:
  4:3 for photo/portrait/master, 16:9 for video. Black bars/rails are part of the
  stock layout.
- The current `sensorLandscape` UI is a known mismatch. Do not claim UI-302 is
  done until all four rotations are device-validated against the stock reference.
- Stock reference screenshots stay in `C:\Temp\opencode\stock-ref` (local, not
  committed) because they contain personal content.

### Background capture (CAP-301)

- Use a bounded queue and a serial worker off the UI thread. The shutter and mode
  controls must return immediately; no full-screen blocking saving overlay.
- Block mode, lens, format and `PHOTO/VIDEO` changes while the queue is non-empty.
  Check the landscape `ModeSegment` as well as the portrait fallback.
- Store orientation with the job; process each shot with its tap-time orientation.
- Show the pending count and the result of each shot. A failed shot must not leave
  a fake gallery entry.
- CAP-301 is a prototype in `feat/background-capture`; it is not merged and the
  landscape `PHOTO/VIDEO` guard is still missing.

## 6. Documentation rules

- Canonical docs live in `docs/`.
- `rust_camera/README.md` is the component deep-dive; link to root docs.
- Use relative links; run a link check mentally before committing.
- Prefer tables and Mermaid diagrams for pipelines and decisions.
- No marketing claims without evidence.

## 7. Decision records

Add an ADR in `docs/DECISIONS/` when changing:

- the Rust/Compose split;
- package identity or HAL access strategy;
- public/private boundaries or licensing;
- HDR/RAW/Live Photo architecture;
- any choice that is hard to reverse.

ADR format: Context → Decision → Alternatives → Consequences → Status.

## 8. Reporting and escalation

Report to the human:

- what changed;
- exact commands/tests run;
- what was observed on device (logs, dimensions, file sizes);
- what remains unverified;
- next step.

**Stop and ask** before:

- changing `applicationId`, public visibility or license;
- deleting/rewriting git history;
- committing a keystore, token or personal data;
- uninstalling a user app or disabling the stock camera without a restore plan;
- using vendor/LMC assets in the public repo.

## 9. Anti-patterns

- Rebuild → install → guess loops without a hypothesis.
- Claiming a camera feature works because the app did not crash.
- Treating `AImageReader` width/height as the JPEG dimensions.
- Copying stock/vendor code into the public repo.
- Adding “temporary” binary patches without documenting and reverting them.
- Mixing unrelated stock-APK research with camera PRs.
- Committing screenshots with personal content.
- Changing version in only one of the three version files.

## 10. Quick checklist

```text
[ ] Read STATUS / ROADMAP / PIPELINE
[ ] Branch from origin/main
[ ] Implement smallest increment + comments
[ ] Host tests / clippy / build
[ ] Device evidence if camera behavior changed
[ ] Update STATUS / ROADMAP / CHANGELOG
[ ] No secrets / PII / vendor assets / build artifacts
[ ] PR with verification and limitations
[ ] CI green, squash merge
[ ] Release only via documented process
```
