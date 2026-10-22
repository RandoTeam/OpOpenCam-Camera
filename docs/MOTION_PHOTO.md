# Motion Photo Specification

## Reference measurements (OnePlus 13, 2026-10-22)

| Source | Still | Motion MP4 | Video | Audio | Duration |
|---|---|---|---|---|---|
| Stock `IMG20261021224738.jpg` | 3072×4096 | 6.22 MB | HEVC 1728×1296, ~30 fps, ~19.8 Mbps | AAC 48 kHz **mono**, 191 kbps | 2.32 s |
| Stock `IMG20260922153310.jpg` | 3072×4096 | 8.20 MB | HEVC 1728×1296, ~24.7 fps, ~18.5 Mbps | AAC 48 kHz **mono**, 191 kbps | 3.35 s |
| LMC 8.4 `GCam_…MP.jpg` | 3072×4096 | 2.57 MB | H.264 1024×768 + 2048×1536 | **none** | 1.43 s |
| LMC 8.3 `PXL_…jpg` | 3072×4096 | — | — | — | no Motion Photo |

Stock also embeds a GainMap JPEG item (244–435 KB).

## Target specification

| Field | Target |
|---|---|
| Still | JPEG 4096×3072 (12.5 MP), EXIF orientation |
| Video | HEVC, 4096×3072 (4:3), 30 fps |
| Duration | 6.0 s total: 3.0 s before + 3.0 s after the shutter |
| Audio | AAC-LC, 48 kHz, **stereo**, 256 kbps |
| Container | Google Motion Photo XMP v1 (JPEG + APP1 XMP + appended MP4) |
| Optional | GainMap / HDR gain map |
| Max size | ~60–90 MB at 80–120 Mbps video |

## Why 4096×3072 rather than 3840×2880

The HAL advertises 4096×3072 output at 30 fps for YUV, PRIVATE and P010 on every
rear camera; 3840×2880 is not in the stream list. 4096×3072 is the native 4:3
binned size and is slightly above UHD 4K.

## Architecture

```
Camera2 PRIVATE/P010 4096×3072 @30
        │
        ├─ MediaCodec HEVC encoder ──► encoded-sample ring buffer (3 s)
        ├─ AudioRecord 48 kHz stereo ─► AAC encoder ─► audio ring buffer (3 s)
        └─ on shutter: keep 3 s before, collect 3 s after
                          │
                          ├─ MediaMuxer (video + audio)
                          ├─ JPEG still
                          └─ motion_photo_xmp::assemble
```

Encode continuously; keep **encoded** samples, not raw frames. A raw 6 s 4K 4:3
ring would be ~3 GB at 30 fps; an 80–120 Mbps HEVC ring is ~60–90 MB.

## Implementation notes

- `motion_photo_xmp.rs` already implements the JPEG APP1 XMP block and appends the
  MP4 at the correct offset; it has host tests.
- `MediaCodec` keyframes: force an IDR at the start of the pre-roll to make trimming
  deterministic.
- Presentation timestamp: the still’s position inside the clip must be written to
  `GCamera:MotionPhotoPresentationTimestampUs`.
- Audio/video drift: use the same monotonic clock for both encoders; verify with
  `ffprobe`.
- Thermal: 4K 4:3 30 fps encode is the worst case; test 1/3/10 minutes.

## Module parity

The module exposes Live Photo high bitrate, adjustable duration, Master-mode Live
Photo, FOV optimization, HEIF/10-bit Live Photo and cover ProXDR. Our clean plan:

| Feature | Plan | Priority |
|---|---|---|
| High bitrate | 45–120 Mbps preset | P1 |
| Adjustable duration | 3+3 default; 1–6 s presets | P2 |
| Master/Pro Live Photo | Pro controls + Live Photo | P2 |
| FOV optimization | match still/video crop; document FOV | P2 |
| HEIF/10-bit Live Photo | HEIF container + 10-bit video | P3 |
| Cover ProXDR | Android gainmap cover | P3 |

No vendor Live Photo code or assets are used.

## Verification checklist

- `ffprobe` shows exactly one HEVC video track and one AAC stereo track.
- Duration 6.0 s ± 50 ms.
- `GCamera:MotionPhoto=1`, `MotionPhotoVersion=1`, correct timestamp.
- Google Photos opens it as Motion Photo.
- File size within budget and no dropped frames.
