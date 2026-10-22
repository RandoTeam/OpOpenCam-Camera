# Plan

This file is a short entry point. The canonical plan lives in:

- [docs/ROADMAP.md](docs/ROADMAP.md) — now / next / later milestones.
- [docs/BACKLOG.md](docs/BACKLOG.md) — task IDs, priorities and dependencies.
- [docs/STATUS.md](docs/STATUS.md) — what is actually verified.
- [docs/DECISIONS/0003-public-private-separation.md](docs/DECISIONS/0003-public-private-separation.md)
  — publication and public/private rules.

## Current focus

1. DNG calibration with user-supplied profiles.
2. HDR quality on textured/moving scenes.
3. 25 MP probe; 50 MP JPEG remains closed (RAW 50 is the max path).
4. Video core: 4K 4:3 30 / 4K 16:9 60, 1080p 120, stereo audio.
5. Live Photo 6 s (3 s before + 3 s after), 4:3, stereo AAC.
6. Pro mode, portrait/macro/zoom, LUTs/watermarks, gallery/editor.
7. 10-bit P010/HDR10, Ultra HDR stills; no Dolby Vision.

Full parity map: [docs/FEATURE_PARITY.md](docs/FEATURE_PARITY.md).

## Publication status

The public repository is open and CI/release workflows are green. Remaining
human/legal review: `TRADEMARKS.md` and `THIRD_PARTY_NOTICES.md`.
