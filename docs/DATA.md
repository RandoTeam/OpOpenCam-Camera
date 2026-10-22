# Device Data Policy

## What stays local

Device captures are stored under:

```
test_reports/device_capture_<YYYYMMDD>/
```

They contain HAL dumps, reference Motion Photos, config files and test images.
These files can include personal scenes and are **not committed** to the repository.

The repository only keeps:

- `DEVICE_TEST_PLAN.md` (the plan);
- derived, non-identifying tables in `docs/` and `README.md`;
- this policy.

## What was captured for the alpha baseline

| Artifact | Purpose |
|---|---|
| `media_camera.txt` | full CameraService/HAL dump (UTF-16LE) |
| `camera_capabilities_summary.txt` / `.json` | parsed camera matrix |
| `CameraHWConfiguration.config`, `fwk_config.json` | sensor/feature configuration |
| `odm_config/` | per-sensor tuning, APS/preview configs |
| `dng/` | color matrices, tone curves, hue/sat maps |
| `media_codecs_sun_vendor.xml`, `media_profiles_sun.xml` | encoder capabilities |
| Motion Photo JPEG/MP4 samples | container/audio/video reference |
| `motion_analysis.txt` | ffprobe/XMP analysis |

## Re-capturing

Follow [DEVICE_TEST_PLAN.md](DEVICE_TEST_PLAN.md). After the session, move the
new directory under `test_reports/` and keep it out of git.

## Sanitising before sharing

If data must be shared, remove:

- the still JPEGs and MP4s (personal scenes);
- device serial numbers and account names;
- any `dumpsys` sections containing package/UID/account data.

Share only the derived capability tables and code.
