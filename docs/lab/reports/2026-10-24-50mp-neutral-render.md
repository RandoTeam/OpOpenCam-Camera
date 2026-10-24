# 50 MP: Stock JPEG vs Our RAW50 Neutral Render - 2026-10-24

## Purpose

Compare the stock High Resolution 50 MP JPEG with our 8192x6144 RAW50 DNG on
the same scene, using a deliberately neutral render: no AI, no local tone
mapping, no sharpening, no vendor LUT. The goal is to decide whether our RAW50
path is a viable replacement for stock 50 MP JPEG.

## Captures

| Item | Stock 50 MP | Our RAW50 |
|---|---|---|
| Device time | 04:44:15 | 04:42:22 |
| File | `IMG20261024044415.jpg` | `IMG_1792806129970.dng` |
| Bytes | 16,020,898 | 100,663,734 |
| Dimensions | 8192x6144 | 8192x6144 |
| Format | JPEG | DNG 1.4, uncompressed 16-bit container |
| Sensor data | unknown vendor JPEG | white 1023, black 64, CFA RGGB |
| Lens / exposure | 23 mm, f/1.6, 1/81 s, ISO 400 | main sensor, no EXIF exposure |
| Gain map / MotionPhoto | none | n/a |
| Orientation | 1 | 6 |

Both were taken in the one-lamp scene. The framing is not identical: the stock
50 MP image is tighter (the bottles are larger), while our RAW50 has a wider
field of view. The comparison is therefore about texture and processing
character, not a pixel-aligned sharpness score.

## Neutral render

The local analysis render used:

- bilinear demosaic of the RGGB CFA;
- black/white normalization;
- gray-world white balance;
- gamma 2.2;
- JPEG quality 95;
- **no** sharpening, local contrast, denoise or neural enhancement.

Output: 8192x6144 JPEG, 11,008,357 bytes. It is intentionally flat and soft;
that is the point of a neutral baseline.

## Bottle crop

A 1200x1200 crop around the left bottle was used. To separate sharpening from
exposure, our crop was linearly matched to the stock crop's mean and standard
deviation before computing gradient metrics.

| Metric | Our raw render | Our render, mean/std matched | Stock 50 MP |
|---|---|---|---|
| Mean luma | 80.95 | 101.88 | 103.07 |
| Std luma | 47.43 | 52.73 | 56.58 |
| Laplacian variance | 6.63 | 10.64 | 23.65 |
| Mean gradient magnitude | 13.90 | 15.63 | 32.86 |

After matching brightness and contrast, the stock JPEG still carries about
**2.2x the Laplacian variance** and **2.1x the gradient energy** in this crop.
That is consistent with strong local contrast/sharpening, not necessarily more
real detail. The stock render also exaggerates specular highlights and label
edges; our neutral render keeps the droplets and label smoother and more
natural.

Caveats:

- the framing differs (stock is tighter);
- the autofocus state and exact subject distance are not proven identical;
- our render uses bilinear demosaic and no tone curve, so it is softer than a
  production renderer would be;
- a single crop is not a complete quality metric.

## What this supports

1. Our RAW50 file is a real 8192x6144 capture with 10-bit sensor data; the
   50 MP path is viable without the vendor JPEG.
2. The stock 50 MP JPEG is heavily processed: strong local contrast, sharpening
   and saturation. It is not a neutral reference and it is not the look we
   want to copy.
3. A neutral RAW50 render preserves natural texture and color, but needs:
   - a calibrated color matrix / white balance;
   - a gentle filmic tone curve to lift shadows without local tone mapping;
   - a small-radius unsharp mask if desired, clearly weaker than stock;
   - no AI/watercolor processing.
4. The stock framing is tighter than our RAW50. This may be a different sensor
   mode/crop rather than a lens difference; repeat with identical framing.

## Next

1. Finish the standalone Rust DNG renderer and use it for the production
   neutral render instead of the analysis script.
2. Repeat the comparison on a tripod with identical framing, focus and
   exposure; measure texture, noise, clipping and color error.
3. Compare RAW50 and RAW12 against stock 12 MP in the same scene.
4. Decide whether the app should save a neutral companion JPEG for RAW50, or
   render on demand in the gallery.
