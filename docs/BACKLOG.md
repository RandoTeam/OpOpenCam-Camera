# Backlog

Task-level view of [ROADMAP.md](ROADMAP.md). IDs are stable; use them in branches
and PRs (`feat/hdr-101-...`).

## P0 — now

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| DNG-101 | User-supplied calibration file format | — | Loads ColorMatrix1/2 + black/white; documented |
| DNG-102 | Load calibration in the DNG writer | DNG-101 | 12/50 MP DNG uses supplied profile |
| DNG-103 | Validate RAW 12/50 in Lightroom/Photoshop | DNG-102 | Correct color/levels; screenshots not committed |
| HDR-101 | Textured static HDR comparison | — | 12 MP vs HDR vs LMC/stock; metrics |
| HDR-102 | Moving-subject ghosting test | HDR-101 | Ghosting documented; go/no-go for motion masks |
| HDR-103 | Tone/weight tuning | HDR-101 | Natural look; no clipped highlights |
| JPEG-101 | Document 50 MP JPEG limitation | — | README/STATUS/CAPTURE_MODES updated |
| CI-101 | Keep public CI/release green | — | Green on main and tags |
| DOC-101 | Handbook + agent rules | — | This docs set reviewed |

## P1 — next

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| LIVE-101 | Camera2 video output surface | — | Preview + video encoder surface in one session |
| LIVE-102 | HEVC encoder + encoded ring buffer | LIVE-101 | 3 s pre-roll without raw-frame memory |
| LIVE-103 | AudioRecord + AAC ring | — | 48 kHz stereo, 256 kbps |
| LIVE-104 | MediaMuxer 3 s + 3 s | LIVE-102/103 | 6.0 s ±50 ms |
| LIVE-105 | Motion Photo XMP assembly | LIVE-104 | Opens in Google Photos |
| VIDEO-101 | 4K 4:3 30 fps recording | LIVE-101 | 4096×3072, HEVC, stable |
| VIDEO-102 | 4K 16:9 60 fps | VIDEO-101 | 3840×2160, stable |
| VIDEO-103 | Stereo audio + sync | LIVE-103 | A/V drift <50 ms |
| UI-101 | Gallery/MediaStore | — | Saved photo appears in gallery |
| UI-102 | Mode labels and 50 MP message | — | Clear “use R50” state |

## P2 — later

| ID | Task | Depends on | Acceptance |
|---|---|---|---|
| HDR-201 | Rotation/parallax alignment | HDR-102 | Lower ghosting on handheld scenes |
| HDR-202 | Motion masks | HDR-201 | Moving subjects do not smear |
| COLOR-201 | User LUT/color profiles | DNG-102 | No vendor assets; user-supplied only |
| VIDEO-201 | 10-bit P010 / HDR profile | VIDEO-101 | Encoder accepts; metadata correct |
| VIDEO-202 | Stabilization tuning | VIDEO-101 | Smooth handheld 4K |
| REL-201 | Beta signing via CI secrets | — | Non-debug key; documented |
| DEV-201 | Multi-device support | — | At least one non-OP13 device |

## Blocked / closed

| ID | Reason |
|---|---|
| JPEG-50 | HAL returns 1920×1440 for 8192×6144 JPEG; use RAW 50 |
| LUT-PUBLIC | Vendor LUT/calibration assets are not redistributable; public build omits them |

## How to pick work

1. P0 before P1; unblock dependencies first.
2. One task per PR.
3. If a task needs a hardware decision, add an ADR before coding.
4. If evidence contradicts the plan, update the plan — do not force the feature.
