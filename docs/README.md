# OpOpenCam Documentation

This directory is the project handbook. Start here, then follow the document that
matches your role.

## Read in this order

| Document | Purpose |
|---|---|
| [PROJECT.md](PROJECT.md) | Vision, goals, non-goals, constraints and success criteria |
| [STATUS.md](STATUS.md) | What is verified today, what is partial, known issues |
| [FEATURE_PARITY.md](FEATURE_PARITY.md) | Full stock + OPCameraPro feature map, feasibility, priorities |
| [PHOTO_UI.md](PHOTO_UI.md) | Photo-first UI, separate Video mode, format sheet, states |
| [PIPELINE.md](PIPELINE.md) | End-to-end camera pipeline and data flow |
| [ROADMAP.md](ROADMAP.md) | Now / next / later milestones and acceptance criteria |
| [BACKLOG.md](BACKLOG.md) | Prioritized task list with IDs and dependencies |
| [AGENTS.md](AGENTS.md) | Rules for AI agents working on the project |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Build, device workflow, current UI/capture model and troubleshooting |
| [TESTING.md](TESTING.md) | Test pyramid, device matrix and evidence requirements |
| [lab/](lab/) | Stock-camera ground-truth lab and specialist-camera checklist |
| [QUALITY.md](QUALITY.md) | Image-quality contract and comparison protocol |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Rust/Compose split, Camera2 session model, JNI |
| [CAPTURE_MODES.md](CAPTURE_MODES.md) | 12/50 MP, RAW, HDR modes and fallbacks |
| [HDR.md](HDR.md) | Bracketing, NCC alignment, Mertens fusion and tone mapping |
| [DNG.md](DNG.md) | RAW/DNG pipeline, calibration and validation |
| [MOTION_PHOTO.md](MOTION_PHOTO.md) | Live Photo / Motion Photo specification |
| [VIDEO.md](VIDEO.md) | 4K 4:3 / 16:9, audio, codecs, thermal constraints |
| [CI.md](CI.md) | CI and release workflows |
| [RELEASING.md](RELEASING.md) | Versioning, tags, signing and release process |
| [DATA.md](DATA.md) | Device-data policy and public/private separation |
| [DEVICE_TEST_PLAN.md](DEVICE_TEST_PLAN.md) | Concrete live test procedure |
| [GLOSSARY.md](GLOSSARY.md) | Terms and abbreviations |
| [DECISIONS/](DECISIONS/) | Architecture Decision Records (ADRs) |

## Quick paths

**New contributor:** PROJECT → STATUS → FEATURE_PARITY → PIPELINE → ROADMAP →
CONTRIBUTING → TESTING.

**AI agent:** start at the repository root `AGENTS.md` (short pointer), then read
DEVELOPMENT → STATUS → FEATURE_PARITY → ROADMAP → PIPELINE → TESTING. Follow the
workflow and Definition of Done in AGENTS.md.

**Release manager:** RELEASING → CI → STATUS → CHANGELOG.

**Researcher:** lab/ ? DATA ? DEVICE_TEST_PLAN ? reports/ ? DECISIONS/0003.

## Source of truth

When documents disagree:

1. `STATUS.md` for what is actually verified.
2. `FEATURE_PARITY.md` for the full stock/module surface and feasibility.
3. `ROADMAP.md` for priorities.
4. `PIPELINE.md` / `ARCHITECTURE.md` for design.
5. `AGENTS.md` for process rules.
6. `DECISIONS/` for why a design was chosen.

Update the relevant document in the same PR as the code change.

## Reports

- [reports/LIVE_2026-10-22.md](reports/LIVE_2026-10-22.md) — first full live
  verification of 0.3.0-alpha.1 (12 MP, RAW 12/50, HDR, lenses, 50 MP JPEG limit).
