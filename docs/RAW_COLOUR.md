# RAW colour metadata

How the app writes and renders DNG colour, and how to verify it without using a
JPEG as a reference.

## Why JPEG is not a reference

The stock camera JPEG has already passed through the vendor ISP: auto white
balance, tone mapping, colour matrix, noise reduction and sharpening are baked
in. Matching our RAW render to that JPEG would mean matching the vendor's
aesthetic, not the sensor. It also hides real metadata defects, because a JPEG
looks fine even when the DNG has no white balance at all.

All checks below use one of:

- the DNG tags themselves;
- the stock camera's own DNG (a raw file, not a JPEG);
- a neutral surface measured in linear light;
- an independent RAW reader (LibRaw via `rawpy`).

## Writer

`camera_engine.rs` reads the static sensor description once per session
(`dng_color::read_sensor_color`) through `ACameraManager_getCameraCharacteristics`
and the per-frame white balance from the capture result:

| DNG tag | Source |
|---|---|
| `ColorMatrix1/2` | `SENSOR_COLOR_TRANSFORM1/2` (XYZ -> camera native) |
| `CameraCalibration1/2` | `SENSOR_CALIBRATION_TRANSFORM1/2` |
| `ForwardMatrix1/2` | `SENSOR_FORWARD_MATRIX1/2` |
| `CalibrationIlluminant1/2` | `SENSOR_REFERENCE_ILLUMINANT1/2` |
| `AsShotNeutral` | `SENSOR_NEUTRAL_COLOR_POINT`, else reciprocal of `COLOR_CORRECTION_GAINS` |
| `BlackLevel` + `BlackLevelRepeatDim` | `SENSOR_BLACK_LEVEL_PATTERN` (4 values, 2x2) |
| `WhiteLevel` | `SENSOR_INFO_WHITE_LEVEL` |
| `ActiveArea`, `DefaultCropOrigin/Size` | `SENSOR_INFO_ACTIVE_ARRAY_SIZE`, scaled to the delivered buffer |
| `CFAPattern`, `CFAPlaneColor`, `CFALayout` | `SENSOR_INFO_COLOR_FILTER_ARRANGEMENT` |
| `DefaultScale` | `[1/1, 1/1]` |
| `Orientation` | current display rotation |

This mirrors AOSP `DngCreator` (`core/jni/android_hardware_camera2_DngCreator.cpp`).
`COLOR_CORRECTION_TRANSFORM` is deliberately **not** used as `ColorMatrix1`: it
maps camera RGB to linear sRGB and already includes the AWB gains, so it has the
wrong direction and the wrong reference space.

The OnePlus HAL delivers `COLOR_CORRECTION_GAINS` as `FLOAT[4]` even though the
NDK documents `RATIONAL`; both are accepted.

### 50 MP streams

The HAL reports a 4096x3072 array while the ultra-high-resolution RAW stream is
8192x6144. `ActiveArea` and the default crop are scaled by the ratio between the
delivered buffer and the reported array, so `DefaultCropSize` becomes
`8176x6128` at 50 MP and `4080x3056` at 12 MP (8 px Adobe margin).

The 50 MP remosaic delivers the same RGGB order as the binned stream. This was
checked two ways: the stock DNG writes `[0,1,1,2]`, and RAW 12 / RAW 50 captured
in one scene show the same phase ordering (phase0 > phase3, symmetric greens).
Declaring BGGR would swap red and blue.

## Renderer

`raw_render.rs` (used by the gallery through `nativeRenderDngToJpeg`):

1. subtracts the per-phase `BlackLevel` and normalises by `WhiteLevel`;
2. applies gains derived from `AsShotNeutral`;
3. demosaics bilinearly;
4. converts camera RGB to linear sRGB through
   `XYZ_TO_SRGB * inverse(CameraCalibration1 * ColorMatrix1) * diag(CM1 * XYZ_D65)`;
5. applies gamma 2.2.

The `diag(CM1 * XYZ_D65)` factor is essential: without it a neutral subject maps
to `inverse(CM1) * (1,1,1)`, which is not XYZ white and produces a magenta cast.

Gray-world white balance remains only as a fallback for DNGs without
`AsShotNeutral` (for example files written by older builds).

## Verification

- **Tag parity with the stock camera.** Dump IFD0 of a stock DNG
  (`_archive/samples/IMG_20261013_163104.dng`) and of our file. Expected: same
  types and values for every colour tag; only per-shot `AsShotNeutral` and the
  50 MP scaling differ.
- **Independent reader.** `rawpy.imread(file)` must report
  `camera_whitebalance` close to `[1/neutral_r, 1, 1/neutral_b]` and a
  non-identity `color_matrix`. Before the fix it reported
  `[0, 1, 0, 0]` and a zero matrix.
- **Neutral surfaces.** Render with the example CLI
  (`cargo run --release --example render_dng -- in.dng out.jpg`), then measure
  a white cabinet / paper / blind patch in linear light. Under warm room light
  expect `R/G ~0.95-1.0`, `B/G ~0.85-0.95`; a green cast shows as `G` clearly
  above both, a magenta cast as `G` clearly below.
- **Regression tests.** `cargo test` covers tag parsing, per-CFA black levels,
  neutral-derived gains, the colour matrix and the fallback path.

## Known limitations

- No `NoiseProfile` yet: the HAL exposes per-channel coefficients, but the
  writer still omits the tag, so converters use their own noise defaults.
- No `OpcodeList2` lens-shading correction and no preview/EXIF IFD.
- No `AsShotWhiteXY`; `AsShotNeutral` covers the same information for Bayer DNG.
- The renderer is a simple bilinear demosaic without lens shading or noise
  reduction; it is intended to show the file faithfully, not to replace a RAW
  converter.
- 50 MP demosaic quality is limited by the remosaic output of the HAL.
