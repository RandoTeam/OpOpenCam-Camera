# OpOpenCam Documentation

This directory is the project handbook. Start here, then follow the document that
matches your role.

## Read in this order

| Document | Purpose |
|---|---|
| [PROJECT.md](PROJECT.md) | Vision, goals, non-goals, constraints and success criteria |
| [STATUS.md](STATUS.md) | What is verified today, what is partial, known issues |
| [PIPELINE.md](PIPELINE.md) | End-to-end camera pipeline and data flow |
| [ROADMAP.md](ROADMAP.md) | Now / next / later milestones and acceptance criteria |
| [BACKLOG.md](BACKLOG.md) | Prioritized task list with IDs and dependencies |
| [AGENTS.md](AGENTS.md) | Rules for AI agents working on the project |
| [TESTING.md](TESTING.md) | Test pyramid, device matrix and evidence requirements |
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

**New contributor:** PROJECT → STATUS → PIPELINE → ROADMAP → CONTRIBUTING → TESTING.

**AI agent:** AGENTS → STATUS → ROADMAP → PIPELINE → TESTING. Follow the workflow
and Definition of Done in AGENTS.md.

**Release manager:** RELEASING → CI → STATUS → CHANGELOG.

**Researcher:** DATA → DEVICE_TEST_PLAN → reports/ → DECISIONS/0003.

## Source of truth

When documents disagree:

1. `STATUS.md` for what is actually verified.
2. `ROADMAP.md` for priorities.
3. `PIPELINE.md` / `ARCHITECTURE.md` for design.
4. `AGENTS.md` for process rules.
5. `DECISIONS/` for why a design was chosen.

Update the relevant document in the same PR as the code change.

## Reports

- [reports/LIVE_2026-10-22.md](reports/LIVE_2026-10-22.md) — first full live
  verification of 0.3.0-alpha.1 (12 MP, RAW 12/50, HDR, lenses, 50 MP JPEG limit).
