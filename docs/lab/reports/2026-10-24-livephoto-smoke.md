# Stock Live Photo Smoke Test - 2026-10-24

First verified stock Live Photo from the lab scene (two bottles, napkins,
blurred kitchen background). The file is local only; no image or proprietary
asset is committed.

## Run

| Item | Value |
|---|---|
| Device | OnePlus 13 `PJZ110`, Android 16 |
| Stock camera | `com.oplus.camera` 6.020.171 |
| Orientation | landscape, rotation 3 |
| Mode | Photo, rear main, 1x, 4:3 |
| Live Photo | on |
| Flash | off |
| Airplane mode / DND | on during the run, restored afterwards |
| Scene | bottles + napkins, blurred lamp/kitchen background |
| File | `DCIM/Camera/IMG20261024011618.jpg` (local copy) |

The first MediaStore pull was truncated at 2,515,667 bytes because the file was
still being written. A second pull after the size stabilised produced the full
**14,227,748-byte** file. Timing from this run is not used.

### Provenance and reproducibility

The file timestamp is `01:16:18` in device time. The host clock is ~15 s ahead
of the device (measured after the run), so this corresponds to the host shutter
at `01:16:32`. The user confirms no manual shot immediately before the run.
This is therefore the **first automated Live Photo capture**; the file is valid
evidence.

After that sample, toggling Live Photo off/on and pulling ~2-14 s after later
captures produced ~2.5 MB stills with no MotionPhoto and no gain map. This was
**not** a broken muxing path. The stock app writes the JPEG first and appends the
gain map and MP4 only when the camera preview pauses (Home or the 120 s
`ScreenOffActivity`). The first stable size is the unfinished JPEG. With the
pause step, every tested trigger produced a full Live Photo; no reboot or reset
is needed. See
[2026-10-24-livephoto-finalization.md](2026-10-24-livephoto-finalization.md).

## File structure

| Part | Bytes | Details |
|---|---|---|
| Primary JPEG | 5,185,558 | 4096x3072, EXIF orientation 1, Display P3 ICC profile |
| HDR gain map | 437,573 | 2048x1536 grayscale JPEG, `hdrgm` XMP |
| Motion Photo MP4 | 8,604,617 | starts with a 4-byte box size, then `ftypmp42` |
| **Total** | **14,227,748** | |

The MP4 offset is `file_size - 8,604,617 = 5,623,131`. Extracting from the
`ftyp` bytes alone (4 bytes later) makes `ffprobe` report `moov atom not found`;
the 4-byte box size must be included.

## XMP (stock values)

```text
GCamera:MotionPhoto = 1
GCamera:MotionPhotoVersion = 1
GCamera:MotionPhotoPresentationTimestampUs = 1666350
OpCamera:MotionPhotoPrimaryPresentationTimestampUs = 1666350
OpCamera:MotionPhotoOwner = oplus
OpCamera:OLivePhotoVersion = 2
OpCamera:VideoLength = 8117204
hdrgm:Version = 1.0

Container:Directory
  Item 1: image/jpeg, Semantic=Primary, Length=0
  Item 2: image/jpeg, Semantic=GainMap, Length=437573
  Item 3: video/mp4, Semantic=MotionPhoto, Length=8604617
```

## Embedded video

| Property | Value |
|---|---|
| Duration | 3.179667 s |
| Codec | HEVC Main, level 150 |
| Resolution | 1728x1296 (4:3) |
| Frame rate | 30 fps (95 frames) |
| Bitrate | ~20.3 Mbps |
| Pixel format | `yuvj420p` |
| Audio | AAC LC, 48 kHz, **mono**, 192 kbps, 148 frames |
| Creation time | 2026-10-23 22:16:20 UTC |

The still presentation timestamp is **1.66635 s** into a **3.179667 s** clip:

- approximate pre-roll: **1.666 s**;
- approximate post-roll: **1.513 s**.

This is not the requested 3 s + 3 s. Stock Live Photo is roughly 1.7 s before
and 1.5 s after.

## A/B: Live Photo off

A second smoke capture used the same scene, camera and 4:3 still, with Live
Photo switched off through the stock UI. The runner recovered from the stock
`ScreenOffActivity`, verified the toggle, used a marker + `find -newer`, waited
for the on-disk size to be stable, and pulled the full file.

| Property | Live Photo off |
|---|---|
| File | `DCIM/Camera/IMG20261024012832.jpg` |
| Total bytes | 5,499,799 |
| Primary JPEG | 4096x3072, 5,060,495 bytes |
| Gain map | 2048x1536 grayscale JPEG, 439,304 bytes |
| Motion Photo MP4 | none |
| XMP | `hdrgm:Version=1.0`, Container Primary + GainMap |
| MediaStore | full local size matches on-device size |

So the **Ultra HDR gain map is present even without Live Photo**. Live Photo
adds the MotionPhoto XMP and the embedded HEVC/AAC clip, increasing the file
from 5.5 MB to 14.2 MB in this scene.

## Codec evidence

`dumpsys media.metrics` for the stock app shows long-lived encoder sessions:

- `c2.qti.hevc.encoder`, 1728x1296, 20 Mbps VBR, profile 1;
- second HEVC session at 1728x1296, 4.19 Mbps;
- `c2.android.aac.encoder`, 48 kHz, mono, 192 kbps;
- one earlier HEVC/AAC session had `lifetimeMs ~122000` and `flush-count=0`.

That is consistent with continuous encoding / a ring buffer, but this run did
not A/B Live Photo on/off. The continuous-buffer hypothesis still needs the
dedicated experiment.

## What this changes

1. **Stock Live Photo is shorter than our target.** We should implement the
   requested 3+3, not copy stock's ~1.67+1.51.
2. **Stock audio is mono.** Our stereo requirement is an improvement, not a
   parity requirement.
3. **Stock stills are Ultra HDR.** The gain map is a separate 2048x1536
   grayscale JPEG inside the same file, and it is present with Live Photo off as
   well. This is the likely explanation for the gallery's deeper dynamic range:
   the file already contains HDR data, and the gallery can render it without
   rewriting the primary JPEG.
4. **Stock video is 1728x1296, not full 4K.** Our implementation can choose
   1728x1296 or a higher 4:3 size; memory and bitrate must be measured.
5. **The cover is the primary JPEG, not a video frame.** The video frame at
   1.666 s is a separate, lower-quality stream.
6. **The Live Photo encoder appears long-lived.** Confirm with an on/off A/B and
   `media.metrics` before/after a single capture.

## Automation lessons

- The stock camera enters `ScreenOffActivity` ("Коснитесь для предварительного
  просмотра") after a period of no interaction. The runner must detect it, tap
  the preview center to recover, and re-check the foreground package.
- Never use two system `Back` presses to leave the camera: that exits to the
  launcher. Use the UI back/close controls and verify the foreground package.
- MediaStore `is_pending=0`, an unchanged mtime and two equal size samples are
  still not enough: the JPEG is complete before the gain map/MP4 append. Pause
  the preview (Home or `ScreenOffActivity`) and wait for the final size and the
  `MotionPhoto`/`GainMap` marker before pulling.
- A full MediaStore query is huge (15k+ rows) and can be truncated. Use a
  marker file plus `find -newer`, and MediaStore only for metadata.
- Keep airplane mode and DND on during a run, and restore them afterwards.
- Do not delete user media; the test file is left in the user's gallery.

## Next

1. ~~Resolve the Live Photo muxing state~~ **Resolved:** deferred append; pause
   the preview and wait for the final file. See the finalization report.
2. Use the marker + `find -newer` + pause + final-marker protocol; no system
   Back; recover `ScreenOffActivity` with an orientation-aware tap.
3. A/B Live Photo off/on and inspect the encoder before/after the shutter.
4. Measure the gallery display with and without the gain map, in airplane mode
   and online.
5. Run the photo matrix with controlled state assertions and the test scene.
