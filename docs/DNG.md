# DNG and RAW

## Current pipeline

1. Camera2 `RAW_SENSOR` reader at 4096×3072 (RAW 12) or 8192×6144 (RAW 50).
2. Row stride is respected; 16-bit words are copied into a `u16` buffer.
3. `dng_writer` writes Adobe DNG 1.4:
   - RGGB CFA, 16 bits per sample;
   - black level 64, white level 1023 (10-bit default);
   - DNGVersion 1.4, TIFF little-endian.
4. File is saved as `.dng` in app-specific Pictures.

Verified: RAW 12 ≈ 25 MB; RAW 50 ≈ 96 MB, 8192×6144, 16-bit, DNG 1.4.

## DNG fallback correction

The public fallback now writes the **inverse sRGB matrix (XYZ → camera)** as
`ColorMatrix1` with the DNG-required **SRATIONAL** type, not the previous
camera→XYZ matrix. It also writes the TIFF orientation (6 for rear, 8 for front)
instead of always `1`. This is still a generic fallback, not a per-sensor
calibration, but it removes a mathematically wrong matrix and sideways RAW.

## Public-build colour policy

The public repository does **not** contain vendor ColorMatrix, LUT, EEPROM or
per-sensor calibration data. The DNG writer uses a generic **sRGB D65 RGB-to-XYZ**
matrix as a neutral fallback. It is structurally valid but not colour-accurate for
the sensor.

## User-supplied calibration

Planned format (JSON, stored outside the repo):

```json
{
  "sensor": "main",
  "cfa": "RGGB",
  "black_level": 64,
  "white_level": 1023,
  "color_matrix_1": [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
  "color_matrix_2": [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
  "illuminant_1": "D65",
  "illuminant_2": "A"
}
```

The app will load a user-selected file; the public build will never ship vendor
calibration. See `DNG-101`/`DNG-102` in [BACKLOG.md](BACKLOG.md).

## Validation

- Open in Lightroom/Photoshop/RawTherapee.
- Check dimensions, bit depth, CFA, black/white levels, DNG version.
- Render a neutral scene and a ColorChecker; record color error.
- Compare 12 MP and 50 MP; watch memory/thermal.

## Known issues

- Generic matrix → color may look flat or shifted.
- Black/white levels may not match the sensor’s actual 10/12-bit range.
- 50 MP DNG is ~96 MB; storage and memory pressure.
- 50 MP RAW requires the `com.oneplus.camera` package and vendor session parameters.

## Legal

No vendor calibration, LUT, EEPROM or stock APK is redistributed. Calibration data
is user-supplied. See [DATA.md](DATA.md) and [TRADEMARKS.md](../TRADEMARKS.md).
