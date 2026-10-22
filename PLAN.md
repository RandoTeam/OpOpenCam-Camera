# Plan

## Publication checklist

- [x] Separate the public project from the private research archive.
- [x] Remove vendor binaries, stock APK/smali, LUTs, EEPROM and calibration assets.
- [x] Remove LMC-derived modules from the public build.
- [x] Replace the vendor DNG colour matrix with a generic sRGB D65 matrix.
- [x] Remove personal data from the public export: no serials, emails, device dumps or screenshots.
- [x] Add MIT license, third-party notices, trademark and interoperability notes.
- [x] Add security and privacy policies.
- [x] Keep the old `RandoTeam/OpOpenCam` private; it contains research history.
- [x] Public CI workflows (GitHub-hosted runners).
- [ ] Final human/legal review of `TRADEMARKS.md` and `THIRD_PARTY_NOTICES.md`.
- [ ] Make the public repository visible after review.

## Next technical milestones

### 1. DNG colour calibration

- Add a user-supplied calibration path: per-sensor `ColorMatrix1/2`, black/white
  levels, CFA pattern.
- Validate RAW 12/RAW 50 DNG in Lightroom/Photoshop.
- Do not bundle vendor calibration data; load it from a user-selected file.

### 2. HDR quality

- Capture a textured static scene and a moving subject.
- Compare 12 MP, HDR 3×, LMC and stock.
- Improve alignment (rotation/parallax) and ghost handling.
- Tune Mertens weights and tone mapping for a natural, iPhone-like look without
  “watercolor” denoise.

### 3. Live Photo

Target: 4096×3072 4:3 HEVC 30 fps, 6.0 s (3 s before + 3 s after),
AAC 48 kHz stereo 256 kbps, JPEG 4096×3072, Motion Photo XMP v1.

- Add a video output surface to the Camera2 session.
- Implement encoded video/audio ring buffers (MediaCodec/MediaMuxer/AudioRecord).
- Assemble the JPEG + XMP + MP4 container with the existing Rust muxer.

### 4. Video

- 4K 4:3 30 fps and 4K 16:9 60 fps.
- 10-bit P010 / dynamic-range profile where the encoder supports it.
- Stabilisation and long-recording thermal test.

### 5. UI/UX

- Gallery/MediaStore integration.
- Clear 50 MP JPEG message (already in the alpha) and mode labels.
- Animations, settings, permission flows.

## Known limitations

- 50 MP JPEG is unavailable on this device; the HAL returns 1920×1440. Use RAW 50.
- DNG colour is generic until calibration is supplied.
- Live Photo, video and stereo audio are not implemented yet.
- Alpha APKs are debug-signed.
