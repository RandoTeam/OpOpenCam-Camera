# Stock Live Photo Finalization - 2026-10-24

## Summary

The apparent "Live Photo muxing stopped after toggling" was a measurement
error. The stock camera writes the still JPEG first and appends the gain map and
MP4 later, when the camera activity pauses. Pulling 1-14 s after the shutter
returns a valid still without MotionPhoto; it is not a broken Live Photo and it
does not need a reboot or a settings reset.

All tested triggers produced a Live Photo once finalization was allowed:

| Trigger | File | Initial pull | Final size | MotionPhoto | Gain map |
|---|---|---|---|---|---|
| ADB tap, Live on | `IMG20261024033037.jpg` | 1.5 MB class | 14,042,128 | yes | yes |
| 100 ms press, Live on | `IMG20261024033057.jpg` | 1.5 MB class | 19,301,619 | yes | yes |
| 250 ms press, Live **off** | `IMG20261024033110.jpg` | 1.5 MB class | 5,546,815 | no | yes |
| no-focus tap, Live on | `IMG20261024033204.jpg` | 2.4 MB class | 13,531,355 | yes | yes |
| volume-down, Live on | `IMG20261024033303.jpg` | 2.4 MB class | 13,340,263 | yes | yes |
| instrumented tap, Live on | `IMG20261024041703.jpg` | 2,211,045 at 1.87 s | 13,357,174 | yes | yes |
| timeline tap, Live on | `IMG20261024042027.jpg` | 2,340,732 at 14.39 s | 13,683,830 | yes | yes |
| tap + Home, Live on | `IMG20261024042508.jpg` | 2,256,409 at 1.42 s | 13,059,994 | yes | yes |

The 250 ms run was the only one with Live Photo off; its 5.5 MB result is the
expected gain-map-only still. The other rows are genuine Live Photos.

## Finalization timeline

The clearest timeline is `IMG20261024042027.jpg`:

| Time (device) | Event |
|---|---|
| 04:20:27 | shutter; JPEG inserted in MediaStore |
| +1.2 s | first file visible |
| +14.4 s | file still 2,340,732 bytes, no `MotionPhoto` |
| 04:22:27.677 | stock `ScreenOffActivity` opens after 120 s idle |
| 04:22:28 | file finalized to 13,683,830 bytes; `MotionPhoto` present |

The stock app defers the muxing until the preview is paused. The same happens
immediately if the camera is backgrounded with Home:

| Time relative to Home | Size | MotionPhoto |
|---|---|---|
| 0.11 s | 2,256,409 | no |
| 1.90 s | 13,059,994 | yes |

This is why manual captures "work": the user eventually leaves the camera or
the screen idles. The earlier automation pulled before the append and then
misread the still as a failed Live Photo.

## Final file structure

Example: `IMG20261024041703.jpg`, total 13,357,174 bytes.

| Part | Bytes | Details |
|---|---|---|
| Primary JPEG | 4,105,346 | 4096x3072, Display P3 ICC |
| Gain map JPEG | 368,054 | 2048x1536 grayscale, `GainMapMax=1.82109` |
| Motion Photo item | 8,883,774 | MP4, starts with a 4-byte box size before `ftyp` |
| Main clip | 8,443,173 | 3.214 s, HEVC 1728x1296 30 fps, ~20.9 Mbps |
| Second MP4 fragment | 440,605 | appended after the main clip; still under investigation |

The main clip contains AAC LC 48 kHz mono at 192 kbps and HEVC video. The XMP
container is:

```text
GCamera:MotionPhoto = 1
GCamera:MotionPhotoVersion = 1
GCamera:MotionPhotoPresentationTimestampUs = 1666350
OpCamera:MotionPhotoPrimaryPresentationTimestampUs = 1666350
OpCamera:MotionPhotoOwner = oplus
OpCamera:OLivePhotoVersion = 2
OpCamera:VideoLength = 8443173
Container:Directory
  Primary   image/jpeg  Length=0
  GainMap   image/jpeg  Length=368050
  MotionPhoto video/mp4 Length=8883778
```

The 4-byte box size before `ftyp` must be included when extracting the MP4;
starting exactly at `ftyp` makes `ffprobe` report `moov atom not found`.

## Why the earlier conclusion was wrong

1. `find -newer` + two equal size samples returned the completed JPEG, not the
   completed Motion Photo.
2. MediaStore `is_pending=0` and the file mtime did not change when the MP4 was
   appended, so neither signalled pending work.
3. The stock encoder and muxer threads remained alive, which looked like a stuck
   pipeline but was the deferred writer.
4. The Live Photo off/on toggle happened in the same period, so it looked
   causal. It was not.
5. A reboot was considered; it was never needed.

## Automation protocol

Use this order for every stock capture that may contain a gain map or motion:

1. Trigger the shutter.
2. Wait for the new file and record its initial size.
3. Press Home (or launch the gallery) to pause the preview.
4. Poll every 0.3-0.5 s: size, then `grep -aob MotionPhoto` and `GainMap`.
5. Stop when the size is stable and the expected marker is present; pull.
6. Restore `com.oplus.camera/.Camera` and verify the foreground activity.

Typical finalization after Home is **1.5-2.0 s**. If Home is not allowed, wait
for the stock `ScreenOffActivity` (about 120 s of idle) and then finalize.

Never treat the first stable size, `is_pending=0`, or an unchanged mtime as
proof that the file is complete.

## Implications for OpOpenCam

- The stock deferred write is a UX bug; our writer should finalize the Motion
  Photo synchronously or expose an explicit pending state.
- Our gallery must tolerate a visible file that is still being appended.
- The writer should build primary + gain map + MP4, then update XMP and
  MediaStore atomically.
- Stock's 1.67 s pre-roll + 1.51 s post-roll and mono audio remain below our
  3+3 stereo target.
- The stock cover is the primary JPEG; the vendor neural cover enhancer is a
  separate, optional path. Our natural still should remain the cover.

## Next

- Use the Home-finalize protocol for all remaining stock tests.
- Stop treating Live Photo as blocked; no reboot/reset is required.
- Implement our own synchronous 3+3 stereo writer later.
