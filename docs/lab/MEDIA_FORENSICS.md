# Media Forensics and Quality Metrics

This document defines how captured files are inspected. It applies to stock
media, our media and the specialist video camera.

## 1. General rules

- Keep originals immutable; hash them on pull.
- Work on copies for metadata experiments.
- Record the tool and version used for every derived number.
- Do not commit originals, vendor metadata dumps or screenshots with personal
  content.
- Prefer open tools: ffprobe/ffmpeg, ExifTool, Python, OpenCV, rawpy.

## 2. JPEG

### 2.1 Structure

Record:

- file size, SHA-256;
- SOF marker, width, height, components, subsampling;
- orientation tag;
- EXIF: make, model, lens, focal length, exposure, ISO, aperture, timestamp,
  GPS presence (not value), software, `MakerNote` presence;
- XMP packets (full text, local only);
- ICC profile presence and name;
- APP2 MPF offsets and image sizes;
- gain-map / Ultra HDR / ProXDR metadata;
- quantization tables and Huffman tables;
- restart interval;
- comment segments;
- trailing data after EOI.

### 2.2 Embedded motion

Search for an embedded MP4 (`ftyp`, `moov`, `mdat`) and extract it. Record offset
and length. See [LIVE_PHOTO.md](LIVE_PHOTO.md).

### 2.3 Quality metrics

On a fixed crop of the lab chart:

- **Texture / sharpness:** gradient energy, Laplacian variance, MTF50 from a
  slanted edge.
- **Noise:** sigma in a flat gray patch, before and after denoise.
- **Dynamic range:** clipping percentage in highlights/shadows, SNR curve.
- **Colour:** patch mean sRGB/Lab, ΔE2000 against reference.
- **Artifacts:** halo/overshoot at high-contrast edges, ringing, banding,
  chroma noise, watercolour patches.
- **Detail preservation:** high-frequency energy vs the RAW reference.
- **File efficiency:** bytes per pixel at a given quality.

Always compare at the same output size and with the same crop. Do not compare a
downscaled preview with a full-resolution file without saying so.

## 3. DNG / RAW

Record:

- DNG version, endianness, IFD layout;
- width/height, orientation, bit depth, compression;
- CFA pattern, black level, white level;
- `ColorMatrix1/2`, `ForwardMatrix1/2`, calibration illuminants;
- `AsShotNeutral`, `AsShotWhiteXY`, baseline exposure;
- noise profile;
- opcode lists, gain maps, previews;
- thumbnail sizes;
- maker notes / vendor tags presence.

Validate by opening in at least two independent raw developers
(Lightroom/RawTherapee/darktable) and comparing colour, orientation and
highlight recovery. Do not bundle vendor calibration data; use user-supplied or
public profiles.

## 4. MP4 / HEVC / AVC

Use:

```bash
ffprobe -v error -show_streams -show_format -show_frames input.mp4
```

Record:

- container brands and structure;
- video codec, profile, level, tier, pixel format, bit depth;
- width/height, sample aspect ratio, rotation matrix;
- frame rate, average frame rate, duration, frame count, variable-frame-rate
  flags;
- bitrate, buffer size, GOP/keyframe interval;
- colour primaries, transfer, matrix, range;
- HDR10 mastering display and content light level;
- Dolby Vision configuration/RPU presence;
- audio codec, profile, sample rate, channels, bitrate, delay;
- track timescales, edit lists, start offset;
- A/V sync: compare a known beep/visual event in both tracks.

For 8K60 and 4K120, check for dropped frames by comparing the frame count with
`duration * nominal_fps` and by inspecting packet timestamps.

## 5. Live Photo

See [LIVE_PHOTO.md](LIVE_PHOTO.md) for the full protocol. Minimum extraction:

1. XMP packet and all Motion Photo fields.
2. MPF offsets.
3. Embedded MP4 offset/length.
4. Clip metadata and audio.
5. Still vs cover comparison.
6. Pre/post measurement against the scene clock.

## 6. UI screenshots

- Crop to a fixed region; record the crop rectangle.
- Use the same display brightness and colour mode.
- Do not compare screenshots taken at different HDR/brightness states.
- Measure edge energy and local contrast, not "looks sharper".
- Keep screenshots local; publish only our own recreations.

## 7. Comparison protocol

For every stock-vs-ours comparison:

1. Same scene, light, distance, zoom, focus and exposure.
2. Same output size; if not possible, record both sizes and compare at the
   smaller size.
3. Capture at least 5 frames per mode; report median and spread.
4. Include a RAW reference where possible.
5. Record processing time separately from quality.
6. Keep a contact sheet with 100% crops for human review.
7. State which differences are visible at 100%, at fit-to-screen, and on the
   phone display.

## 8. Report template

```markdown
### <mode> / <device>

- File:
- Dimensions / size / codec:
- Exposure / ISO / shutter:
- Crop:
- Sharpness:
- Noise:
- Dynamic range:
- Colour:
- Artifacts:
- Latency:
- Verdict:
```

## 9. Tooling status

Available on the workstation:

- Python 3.12;
- ffmpeg/ffprobe 9.x;
- Pillow/OpenCV/rawpy can be added to the lab environment;
- ExifTool is not installed yet; install a portable copy for full XMP/MPF/DNG
  inspection.

The lab tooling must be reproducible; pin versions in `tools/lab/requirements.txt`
when the scripts are added.
