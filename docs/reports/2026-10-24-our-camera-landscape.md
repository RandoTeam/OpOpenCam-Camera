# Our Camera - Landscape Device Pass - 2026-10-24

## Context

The stock camera is landscape on the OnePlus 13. The first UI v2 build was
portrait-locked; on the device it produced a sideways preview and sideways
stills. This pass moved the app to a landscape camera composition and fixed the
capture orientation.

## Build

| Item | Value |
|---|---|
| Device | OnePlus 13 `PJZ110`, Android 16 |
| Package | `com.oneplus.camera` (debug) |
| Activity | `sensorLandscape` |
| Preview | `TextureView`, 1440x1080 sensor buffer |
| Host tests | 35 passed |
| CI | pending at the time of writing |

## What changed

- `TextureView` replaces `SurfaceView` so the preview can be rotated without
  affecting the camera session.
- Landscape UI: gallery/shutter/flip on the left, zoom stops on the right,
  PHOTO/VIDEO at the bottom, format badge at the top.
- The format sheet, saving card, REC timer and format badge are shared with the
  portrait layout.
- JPEG/DNG orientation is computed from the sensor and display rotation and is
  applied through a new JNI setter.
- HDR 3x rotates the fused frame to the display orientation.

## Device results

| Test | Result |
|---|---|
| 1x JPEG | 4096x3072, 23 mm eq, f/1.6, orientation 3, upright |
| 3x JPEG | 4096x3072, 73 mm eq, f/2.6, orientation 3, upright |
| 6x JPEG | 4096x3072, tele + 2x crop, upright |
| 0.6x JPEG | 4096x3072, 15 mm eq, f/2.0, orientation 3, upright |
| RAW 50 | 8192x6144 DNG, orientation 3; neutral render upright |
| HDR 3x | 4096x3072, upright landscape, ~3.1 s save |
| Format sheet | opens; 12 MP/RAW 12/RAW 50/HDR selectable |
| Saving card | visible during RAW 50/HDR save |
| Physical lenses | 3x -> camera 4; 6x -> camera 4 ratio 2.0; 0.6x -> camera 3; 1x -> camera 0 |

## Findings

1. A portrait-locked activity cannot correctly frame a landscape sensor without
   either a preliminary crop or a custom preview transform; the app is now
   landscape-first like the stock camera.
2. JPEG orientation alone is insufficient if the sensor buffer is upside down.
   The correct landscape tag for this device/rotation is 180 degrees
   (`Orientation 3`); the previous 90-degree tag produced sideways stills.
3. The ultra-wide and main/tele sensors share the same rear orientation in this
   configuration; no per-lens offset is needed after the landscape fix.
4. HDR was hardcoded to rotate 90 degrees into portrait. It now follows the
   display orientation and produces a 4096x3072 landscape JPEG.
5. The physical-lens presets work without the stock low-light main-sensor
   avoidance: camera 4 is opened at 3x, and 6x applies a 2x crop on it.

## Not yet validated

- Front camera orientation and JPEG.
- Video orientation/rotation metadata.
- Pinch gesture on a real two-finger touch (ADB injection is not reliable).
- Portrait crop mode; portrait UI is retained but the app is landscape-first.
- Final visual polish of the preview crop, thumbnail and format sheet.
- Photo quality against stock 12 MP/50 MP and HDR gain-map output.

## Conclusion

The camera is now usable in the stock orientation: preview and stills are
upright, the physical lenses are selected honestly, and the format/saving UI is
visible. The next quality work is the photo pipeline itself, not another
orientation fix.
