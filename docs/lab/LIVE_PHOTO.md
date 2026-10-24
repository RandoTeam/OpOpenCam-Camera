# Live Photo Forensics

The stock camera's Live Photo is a still plus a short motion/audio clip. The
user requirement is 3 s before + 3 s after, 4:3, stereo audio, packaged as a
Motion Photo that the stock gallery can play.

We do not yet know whether the stock app records continuously in preview,
what it buffers, or how the cover frame is enhanced. This document defines the
experiments that answer those questions.

## 1. What we already know

- The stock photo top bar has a dedicated control:
  `com.oplus.camera:id/live_photo`, content description
  `Живое фото off/on`.
- A separate control `motion_capture` is the sports/action mode, not Live
  Photo.
- The vendor tree contains `livephoto_coverenhancer` with:
  - `libOPAlgoCamCoverEnhancer.so`;
  - `RRDBNet_sm8750.bin`;
  - QNN HTP libraries (`libQnnHtp.so`, V79 stub, system library).
  This strongly suggests the cover frame is enhanced by an on-device neural
  network on the Hexagon HTP/NPU. It does not prove when or how it runs.
- The stock camera writes to `DCIM/Camera`; our app writes to
  `DCIM/OpOpenCam`.

## 1a. First stock measurement (2026-10-24)

A verified stock Live Photo was captured in landscape with Live Photo on:

| Property | Stock value |
|---|---|
| File | 14,227,748 bytes |
| Primary JPEG | 4096x3072, 5,185,558 bytes |
| HDR gain map | 2048x1536 grayscale JPEG, 437,573 bytes |
| Embedded MP4 | 8,604,617 bytes |
| Clip duration | 3.179667 s |
| Still presentation timestamp | 1.66635 s |
| Approx. pre-roll / post-roll | 1.666 s / 1.513 s |
| Video | HEVC Main L150, 1728x1296, 30 fps, ~20.3 Mbps |
| Audio | AAC LC, 48 kHz, **mono**, 192 kbps |
| XMP | `GCamera:MotionPhoto`, `MotionPhotoPresentationTimestampUs`, `OpCamera:OLivePhotoVersion=2`, `hdrgm:Version=1.0` |
| Container items | Primary JPEG, GainMap JPEG, MotionPhoto MP4 |

This is **not** 3+3: stock is shorter, and stock audio is mono. The gain map
also suggests the gallery's extra dynamic range is already in the file, not a
later rewrite. An A/B with Live Photo off produced a 5,499,799-byte file:
4096x3072 primary + 2048x1536 gain map, **no MP4**. The gain map is therefore a
normal still feature, not a Live Photo feature. Details and automation lessons:
[reports/2026-10-24-livephoto-smoke.md](reports/2026-10-24-livephoto-smoke.md).

**Provenance:** the 14.2 MB sample's device timestamp is `01:16:18`; the host
clock is ~15 s ahead, so it matches the host shutter at `01:16:32`. The user
confirms no manual shot immediately before. This is the first automated Live
Photo capture. Later captures after toggling Live Photo off/on produced ~2.5 MB
stills without MotionPhoto, even after a clean restart, while the stock encoder
threads were present. The muxing path stopped; a clean reset/reboot is needed to
make it repeatable.

## 2. Hypotheses to test

| # | Hypothesis | Evidence that would confirm | Evidence that would reject |
|---|---|---|---|
| H1 | Video is encoded continuously during preview into a ring buffer | encoder active before shutter; steady codec session in `media.metrics`; buffer/memory growth; pre-roll present on first shot | encoder starts only at shutter; first shot has no pre-roll |
| H2 | Audio is recorded continuously into a matching ring buffer | audio track active before shutter; pre-roll audio present | audio starts at shutter |
| H3 | At shutter, the app writes the JPEG and muxes pre+post into an MP4, then embeds it in the JPEG | file appears after mux; mdat offset; XMP/MPF fields; write burst | two separate files, no embedding |
| H4 | The cover frame is enhanced separately by the RRDBNet/QNN path | cover differs from the JPEG still; QNN/cover logs; NPU-like latency | cover equals the still; no model activity |
| H5 | Pre/post is exactly 3 s + 3 s | scene clock measurement | different duration |
| H6 | The clip is 4:3 and matches the still FOV | ffprobe dimensions and frame compare | cropped/scaled mismatch |
| H7 | Audio is stereo AAC 48 kHz | ffprobe/stream metadata | mono/other rate |
| H8 | The result is a standard Motion Photo (XMP/MPF), not a private sidecar | XMP fields, MPF, gallery playback in other apps | private format only |

## 3. Test scene

Use a monitor or tablet showing:

- a large millisecond clock, updated every frame;
- a frame counter;
- a colour/edge target in the same frame;
- a speaker with a 1 kHz beep every second, or a known click track.

The clock lets us measure pre/post without trusting the app's metadata. The
beep lets us measure audio offset and drift.

Camera setup:

- rear main, 1x, 4:3, HDR off, AI off, flash off, grid off;
- fixed focus on the clock;
- fixed brightness and refresh rate;
- phone on a stand, no hand shake;
- 20 GB free, cool device.

Repeat for front 1x and rear 3x/6x after the baseline.

## 4. Experiment matrix

| ID | Experiment | Purpose |
|---|---|---|
| LP01 | Live Photo off -> capture | baseline still only; file list |
| LP02 | Live Photo on -> wait 5 s -> capture | normal pre-roll |
| LP03 | Enable Live Photo -> capture immediately | does pre-roll exist on the first shot? |
| LP04 | Wait 15 s in preview -> capture | is pre-roll fixed or buffer-limited? |
| LP05 | Capture 5 shots 2 s apart | buffer reuse, file sizes, thermal |
| LP06 | 4:3 vs 16:9 | clip/still framing and dimensions |
| LP07 | Rear 1x / 3x / 6x | lens, FOV, stabilization |
| LP08 | Front 1x | front pipeline, orientation, audio |
| LP09 | HDR on | does HDR still change the clip/cover? |
| LP10 | Timer 3 s / 10 s | pre-roll alignment with timer |
| LP11 | Burst / long-press | Live Photo interaction |
| LP12 | Low light | noise, bitrate, frame rate, pre-roll |
| LP13 | Gallery playback | duration, loop, audio, cover |
| LP14 | Share/export | exported format, quality, metadata |
| LP15 | Edit/crop | what happens to motion/audio |
| LP16 | Airplane mode | any cloud dependency |

For each: capture a screen recording of the shutter press, pull the still and
any sidecars, and record `media.metrics` + logcat.

## 5. Instrumentation

### Before shutter

- `dumpsys media.metrics` - look for an active encoder and audio track.
- `dumpsys media.audio_flinger` - active record tracks, sample rate, channels.
- `ps -AT -p <pid>` - look for encoder/muxer/audio threads.
- Perfetto `video`, `audio`, `camera`, `hal` - encoder and muxer slices.
- `dumpsys media.camera` - number and format of output streams.
- `/proc/<pid>/task/*/comm` - thread names (quote the path correctly).

### At/after shutter

- `dumpsys media.metrics` - encoder component, width/height, fps, profile,
  frames, duration, latency.
- logcat tags: `MediaCodec`, `CCodec`, `C2`, `MPEG4Writer`, `MediaMuxer`,
  `MotionPhoto`, `LivePhoto`, `XMP`, `QNN`, `HTP`, `Cover`.
- File write timing: poll the directory every 100 ms; record when the file
  appears and when its size stabilises.
- Pull the result and run the forensics below.

## 6. File forensics

### 6.1 Container identification

1. Hash the file.
2. Read the first bytes: JPEG `FF D8`, HEIC `ftyp`, MP4 `ftyp`.
3. Scan JPEG markers:
   - `APP1` EXIF;
   - `APP1` XMP;
   - `APP2` ICC;
   - `APP2` MPF (Multi-Picture Format);
   - `APP15` or vendor segments.
4. Extract every XMP packet and pretty-print it.
5. Parse MPF offsets and image sizes.
6. Search for an embedded MP4: `ftyp` + `moov` + `mdat` inside the JPEG.
7. If no embedded MP4, look for a sidecar `.mp4`, `.xmp`, or a private
   directory.

### 6.2 XMP fields to record

Common Motion Photo fields (names may vary by vendor):

- `GCamera:MicroVideo` / `GCamera:MicroVideoVersion`;
- `GCamera:MicroVideoOffset`;
- `GCamera:MicroVideoPresentationTimestampUs`;
- `MotionPhoto`, `MotionPhotoVersion`, `MotionPhotoPresentationTimestampUs`;
- `Container:Directory`, `Item:Semantic`, `Item:Mime`, `Item:Length`;
- vendor-specific `oplus`/`oneplus` fields.

Record exact names and values; do not normalise them away.

### 6.3 Video stream

Use `ffprobe -show_streams -show_format -show_frames` and record:

- codec, profile, level, pixel format, bit depth;
- width/height, sample aspect ratio, rotation;
- frame rate, average frame rate, duration, frame count;
- bitrate, colour primaries/transfer/matrix/range;
- HDR metadata (mastering display, content light level, Dolby Vision RPU);
- keyframe interval;
- audio codec, sample rate, channels, bitrate, delay.

### 6.4 Pre/post measurement

1. Read the scene clock in the still JPEG -> `t0`.
2. Extract all video frames; read the clock in each frame.
3. Find the frame nearest `t0`.
4. `pre = t0 - first_frame_time`; `post = last_frame_time - t0`.
5. Repeat on 10 captures; report mean, min, max, standard deviation.
6. Check whether the clip starts before the shutter UI press, at the press, or
   after it.

If the clock is unreadable, use the audio beep pattern and cross-correlate the
audio track with a known recording; document the method.

### 6.5 Cover enhancement

1. Extract the JPEG's primary image and the first/last video frame.
2. Compare at 100%: texture, noise, sharpening halos, colour.
3. Check whether the still differs from the cover when the video is played.
4. Correlate with QNN/cover logs and timing.
5. If the cover is enhanced, record the latency and whether the original still
   is preserved.

## 7. Metrics

| Metric | Unit | Notes |
|---|---|---|
| pre_roll | ms | scene clock, not metadata |
| post_roll | ms | scene clock |
| clip_duration | ms | ffprobe |
| clip_fps | fps | average and min |
| clip_resolution | px | must match still FOV |
| video_codec | string | HEVC/AVC, profile/level |
| video_bitrate | Mbps | average |
| audio_codec | string | AAC/other |
| audio_rate/channels | Hz / count | stereo required |
| audio_offset | ms | beep cross-correlation |
| file_size | bytes | still + clip |
| mux_latency | ms | shutter -> final file |
| cover_enhance_ms | ms | if measurable |
| buffer_memory | MB | inferred from RSS/dmabuf |
| thermal_delta | C | before/after |
| first_shot_pre_roll | ms | LP03 critical case |

## 8. Implementation implications for OpOpenCam

Before UI work, prove the session shape in `SelfTestActivity`:

1. Open logical camera 0 with preview + JPEG + encoder input surface.
2. Confirm the session is accepted and the preview stays smooth.
3. Start an HEVC encoder at 4:3, 30 fps, with a ring buffer of encoded frames.
4. Start an audio record ring buffer at 48 kHz stereo.
5. On shutter: capture JPEG, stop after 3 s, mux pre+post, embed the MP4 and
   XMP/MPF.
6. Verify the result in the stock gallery and in a neutral player.
7. Only then expose the Live Photo toggle in the product UI.

Open questions for our engine:

- Can we keep the encoder surface alive while taking a JPEG on the same
  session, or do we need a second session/reprocessing path?
- What is the memory cost of a 3 s + 3 s 4:3 HEVC ring buffer at 30 fps?
- Do we need a cover-enhancement model? The stock one is vendor-only; our
  default should preserve the natural still and make enhancement optional.
- How do we keep A/V sync within 50 ms without copying the vendor's private
  offset constants?

## 9. Acceptance

- 10/10 captures play in the stock gallery and a neutral player.
- Pre-roll 3.0 s +/- 0.2 s and post-roll 3.0 s +/- 0.2 s on the scene clock.
- 4:3 clip matching the still FOV; stereo 48 kHz audio; sync within 50 ms.
- No dropped frames in the pre-roll under normal indoor light.
- File opens after a cold reboot; no dependency on our app process.
- Stock gallery can play it; if it cannot, the limitation is documented with
  the exact reason.
