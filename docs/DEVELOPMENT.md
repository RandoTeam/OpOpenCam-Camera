# Development Handbook

Current as of 2026-10-26. This is the practical entry point for working on
OpOpenCam Camera. The binding rules are in [AGENTS.md](AGENTS.md); this document
explains how the project is laid out, built, tested and changed.

## 1. Project location

| Path | Role |
|---|---|
| `C:\VietnAi\OpOpenCam-Camera` | **Active project**; public repo `RandoTeam/OpOpenCam-Camera` |
| `C:\VietnAi\OpOpenCam` | Old private research archive; not the camera codebase, never publish |
| `C:\Temp\opencode\stock-ref` | Local stock-camera reference screenshots; not committed |
| `C:\Temp\opencode\stock-lab` | Local capture/analysis scratch; not committed |

Work in the active project. The old `OpOpenCam` repository contains vendor/APK
research and historical notes; do not copy its assets into the public repo.

## 2. Repository map

```text
rust_camera/
  rust_core/        Rust Camera2 NDK engine, JNI bridge, HDR, DNG renderer
  android_app/      Compose UI, JNI declarations, MediaStore, gallery
  scripts/          build.ps1, check.ps1, install.ps1
docs/               Canonical handbook (this directory)
.local-research/    Ignored private research (vendor APKs, LUTs, calibration)
```

## 3. Read before changing code

1. [STATUS.md](STATUS.md) — what is actually verified.
2. [ROADMAP.md](ROADMAP.md) and [BACKLOG.md](BACKLOG.md) — priorities and IDs.
3. [PHOTO_UI.md](PHOTO_UI.md) — current UI contract and stock layout reference.
4. [PIPELINE.md](PIPELINE.md) / [ARCHITECTURE.md](ARCHITECTURE.md) — data flow.
5. [TESTING.md](TESTING.md) and [DEVICE_TEST_PLAN.md](DEVICE_TEST_PLAN.md) — evidence.
6. [AGENTS.md](AGENTS.md) — process, legal and reporting rules.

## 4. Toolchain

- JDK 21: `C:\Program Files\Eclipse Adoptium\jdk-21.0.11.10-hotspot`
- Gradle wrapper 8.13; Rust stable with Android targets; Android SDK/NDK.
- Device: OnePlus 13 `PJZ110`, Android 16, adb serial `a36d1946`.
- Display: `2376x1080` landscape, rotation `270`; package identity is
  `com.oneplus.camera`, JNI package `com.opopencam.camera`.

## 5. Build, check, install

```powershell
# Full Rust + Android build; copies libopopencam.so into jniLibs and assembles
.\rust_camera\scripts\build.ps1

# Format, host tests, Clippy, version consistency, Android debug build
.\rust_camera\scripts\check.ps1

# Install the debug APK and launch
.\rust_camera\scripts\install.ps1
```

Manual Gradle fallback:

```powershell
cd C:\VietnAi\OpOpenCam-Camera\rust_camera\android_app
$env:JAVA_HOME="C:\Program Files\Eclipse Adoptium\jdk-21.0.11.10-hotspot"
.\gradlew.bat :app:assembleDebug
adb install -r -g app\build\outputs\apk\debug\app-debug.apk
adb shell am force-stop com.oneplus.camera
adb shell am start -n com.oneplus.camera/com.opopencam.camera.MainActivity
```

Do not commit `target/`, `build/`, `.gradle/`, `jniLibs/*.so`, `local.properties`
or `.cargo/config.toml`.

## 6. Device workflow

1. Confirm the device is online: `adb devices` (expected `a36d1946 device`).
2. Install and launch; take a screenshot before judging any UI change.
3. Read native evidence with `adb logcat -s OpOpenCamNative`; do not treat a
   missing crash as success.
4. Pull output files and check dimensions, EXIF/DNG orientation and file size.
5. Record exactly what was tested and what remains unverified.

Safety:

- No root, no bootloader, no vendor binaries/assets/LUTs/calibration in the repo.
- Do not change stock `com.oplus.camera` settings; opening it for reference is
  allowed, changing it is not.
- Do not delete user media or reboot the device without explicit approval.
- Ask before package identity, licence, public visibility or history changes.

## 7. Current UI model (stock parity)

The stock camera uses **one layout in phone coordinates**, not two reflowed
layouts. Screenshots 01 (portrait) and 02 (landscape) prove it: rotating 02 by
90 degrees maps every control onto 01. The whole canvas rotates with the device;
positions stay fixed relative to the phone.

Implementation contract for UI-302:

- Define the screen in portrait coordinates (`1080x2376`).
- Rotate the whole canvas by the display rotation. Do not move controls to other
  edges when the phone turns.
- Counter-rotate only glyphs that must stay readable: settings icons, zoom
  numbers, shutter/flip/gallery, quick toggles. Do **not** counter-rotate the
  `PHOTO/VIDEO/PORTRAIT/MASTER` labels; they run along the rail.
- Preview is fitted to the sensor aspect, never stretched:
  - photo/portrait/master: 4:3 (`1080x1440` portrait, `1440x1080` landscape);
  - video: 16:9 (`1080x1920` portrait, `1920x1080` landscape).
- Stock portrait anchors: top toolbar above the preview; preview starts at
  `y≈299`; zoom row over the preview bottom (`y≈1515`); mode strip below the
  preview (`y≈1810`); shutter row at `y≈2050` (thumb left, shutter centre, flip
  right). In landscape the same canvas maps to: settings on the left rail, mode
  strip vertical on the right, shutter right centre, thumb bottom right, flip top
  right.
- The current code is still `sensorLandscape`; UI-302 will replace it with a
  sensor-aware activity (`fullSensor` + `configChanges`) and a rotating canvas.
  Do not claim UI-302 is done until all four rotations are device-validated.

Full reference: [PHOTO_UI.md](PHOTO_UI.md#stock-camera-layout-reference-2026-10-26).

## 8. Capture queue contract (CAP-301)

- Bounded `Channel<CaptureJob>(capacity = 3)`, one serial worker on
  `Dispatchers.IO`; the shutter returns immediately.
- No full-screen blocking saving overlay. Show a compact status with the pending
  count and the result of each shot.
- Block mode, lens, format and `PHOTO/VIDEO` changes while `queuedCaptures > 0`.
  The landscape `ModeSegment` must have the same guard as the portrait fallback.
- Store the orientation with the job so a queued shot uses the orientation at tap
  time.
- Publish through MediaStore; a failed shot must not leave a fake gallery entry.

Current state: implemented and device-validated (2026-10-26). Three rapid JPEG
taps and three rapid HDR taps each produced three files; the queue is serial, the
UI stays responsive, mode/lens/format/PHOTO-VIDEO are blocked, and the pending
count is shown. Queue-full feedback is limited to the disabled shutter. Portrait
orientation is not fixed here; it depends on UI-302 unlocking the activity.

## 9. Current priorities

1. Finish CAP-301: guard `PHOTO/VIDEO`, pending status, HDR/JPEG queue tests,
   merge.
2. UI-302: rotating portrait canvas, stock positions, selective counter-rotation,
   letterboxed preview.
3. UI-303: `x` vs `mm` zoom readout.
4. UI-304: long-press zoom arc.
5. Quality: HDR gain map from the bracket, neutral RAW tone/WB.
6. Video/front/Live Photo only after the photo core is accepted.

## 10. Documentation map

| Document | Purpose |
|---|---|
| [STATUS.md](STATUS.md) | Verified state and known limitations |
| [ROADMAP.md](ROADMAP.md) | Milestones and acceptance |
| [BACKLOG.md](BACKLOG.md) | Task IDs and current acceptance text |
| [PHOTO_UI.md](PHOTO_UI.md) | UI contract and stock layout reference |
| [PIPELINE.md](PIPELINE.md) / [ARCHITECTURE.md](ARCHITECTURE.md) | Data flow and design |
| [TESTING.md](TESTING.md) / [DEVICE_TEST_PLAN.md](DEVICE_TEST_PLAN.md) | Evidence protocol |
| [AGENTS.md](AGENTS.md) | Binding rules for agents and humans |
| [RELEASING.md](RELEASING.md) / [CI.md](CI.md) | Release and CI |

## 11. Troubleshooting

- **Preview sideways, still upright**: the preview stream arrives 90 degrees
  rotated relative to the JPEG stream. Use the display counter-rotation for the
  preview (`(360 - displayRotation * 90) % 360`) while the JPEG uses
  `(sensorOrientation - displayRotation * 90) % 360`. Fixed in
  `feat/background-capture`; not on `main` yet.
- **RAW 50 on the tele lens**: the session fails with `-10000` and falls back to
  12 MP. The UI badge must update to the active mode; do not present RAW 50.
- **Mode changes during capture**: the landscape `ModeSegment` lacks the
  `queuedCaptures == 0` guard. Check both layout branches.
- **Standby orientation**: `OrientationEventListener` reports physical rotation;
  when the phone is flat, use the display rotation for the preview, not the raw
  sensor angle.
- **Stock reference**: keep screenshots in `C:\Temp\opencode\stock-ref`; do not
  commit personal content.
