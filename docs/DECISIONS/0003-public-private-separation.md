# ADR 0003 — Public/private separation

**Status:** accepted

## Context

The original `RandoTeam/OpOpenCam` repository contains:

- `tools/debug.keystore` and signing material;
- device serial numbers in tests/docs;
- personal screenshots and UI dumps;
- stock APK/smali, vendor libraries, LUTs, EEPROM, LMC analysis.

Publishing that history would leak PII and redistribute proprietary assets. Git
history cannot be cleaned by deleting files from HEAD alone.

## Decision

- Keep `RandoTeam/OpOpenCam` **private** as a research archive.
- Create a new public repository `RandoTeam/OpOpenCam-Camera` containing only the
  `rust_camera` project and original documentation.
- Use a fresh git history with a non-personal author email.
- Public build excludes vendor assets and LMC-derived modules.

## Alternatives

1. Rewrite the old repository history (filter-repo/BFG) and publish it. High risk,
   easy to miss a blob, and the research archive is still useful privately.
2. Publish with disclaimers. Not acceptable: proprietary binaries and PII remain.
3. Keep everything private. Safe but does not meet the open-source goal.

## Consequences

- Clean public repo: 60 files, no PII/secrets/vendor binaries.
- Two repositories: public camera and private research.
- The public repo has its own CI, releases and version history.
- Do not publish the old repo without a full history rewrite and legal review.
