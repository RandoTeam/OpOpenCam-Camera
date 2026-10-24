//! Neutral Bayer DNG -> JPEG renderer.
//!
//! This is a deliberately small, deterministic CPU pipeline:
//!
//! 1. Parse the little-endian TIFF/DNG IFD0 written by [`crate::dng_writer`].
//! 2. Normalize the 16-bit CFA samples with the DNG black/white levels.
//! 3. Estimate a gray-world white balance from the raw CFA means.
//! 4. Bilinearly demosaic RGGB/BGGR/GRBG/GBRG.
//! 5. Apply the DNG orientation, gamma 2.2 and encode JPEG at quality 95.
//!
//! There is no AI, no local tone mapping, no sharpening, no lens correction and
//! no vendor calibration. Colour is therefore neutral but not sensor-accurate;
//! the public writer only provides a generic sRGB fallback matrix and this
//! renderer intentionally does not apply it.

use std::io::Cursor;
use std::path::Path;

use jpeg_encoder::{ColorType, Encoder};
use thiserror::Error;

/// JPEG quality used by the renderer and the example CLI.
pub const JPEG_QUALITY: u8 = 95;

/// Gamma applied after white balance, before quantizing to 8-bit sRGB-ish output.
const OUTPUT_GAMMA: f32 = 2.2;

/// Gray-world gains are clamped to this range to keep a very dark or very
/// saturated scene from producing extreme channel amplification.
const MIN_WB_GAIN: f64 = 0.25;
const MAX_WB_GAIN: f64 = 4.0;

const TAG_IMAGE_WIDTH: u16 = 256;
const TAG_IMAGE_LENGTH: u16 = 257;
const TAG_BITS_PER_SAMPLE: u16 = 258;
const TAG_COMPRESSION: u16 = 259;
const TAG_PHOTOMETRIC_INTERPRETATION: u16 = 262;
const TAG_STRIP_OFFSETS: u16 = 273;
const TAG_ORIENTATION: u16 = 274;
const TAG_SAMPLES_PER_PIXEL: u16 = 277;
const TAG_ROWS_PER_STRIP: u16 = 278;
const TAG_STRIP_BYTE_COUNTS: u16 = 279;
const TAG_CFA_REPEAT_PATTERN_DIM: u16 = 33421;
const TAG_CFA_PATTERN: u16 = 33422;
const TAG_DNG_VERSION: u16 = 50706;
const TAG_BLACK_LEVEL: u16 = 50714;
const TAG_WHITE_LEVEL: u16 = 50717;

const PHOTOMETRIC_CFA: u32 = 32803;
const TIFF_MAGIC: u16 = 42;
const IFD_ENTRY_SIZE: usize = 12;

#[derive(Debug, Error)]
pub enum RawRenderError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid DNG/TIFF: {0}")]
    InvalidTiff(String),
    #[error("missing required DNG tag {tag} ({name})")]
    MissingTag { tag: u16, name: &'static str },
    #[error("unsupported {what}: {value} ({supported})")]
    Unsupported {
        what: &'static str,
        value: String,
        supported: &'static str,
    },
    #[error("invalid image dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),
    #[error("JPEG encoding failed: {0}")]
    Jpeg(String),
}

/// The 2x2 Bayer phase of the raw image. Channel order is red, green, blue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfaPattern {
    Rggb,
    Bggr,
    Grbg,
    Gbrg,
}

impl CfaPattern {
    fn from_code(code: [u8; 4]) -> Option<Self> {
        match code {
            [0, 1, 1, 2] => Some(Self::Rggb),
            [2, 1, 1, 0] => Some(Self::Bggr),
            [1, 0, 2, 1] => Some(Self::Grbg),
            [1, 2, 0, 1] => Some(Self::Gbrg),
            _ => None,
        }
    }

    /// Channel at a raw (unrotated) coordinate: 0 = red, 1 = green, 2 = blue.
    #[inline]
    fn color_index(self, x: usize, y: usize) -> usize {
        let phase = ((y & 1) << 1) | (x & 1);
        match self {
            Self::Rggb => [0, 1, 1, 2][phase],
            Self::Bggr => [2, 1, 1, 0][phase],
            Self::Grbg => [1, 0, 2, 1][phase],
            Self::Gbrg => [1, 2, 0, 1][phase],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Rggb => "RGGB",
            Self::Bggr => "BGGR",
            Self::Grbg => "GRBG",
            Self::Gbrg => "GBRG",
        }
    }
}

/// Parsed main image from IFD0 of a DNG file.
#[derive(Clone, Debug)]
pub struct DngImage {
    pub width: u32,
    pub height: u32,
    pub orientation: u16,
    pub black_level: u32,
    pub white_level: u32,
    pub cfa: CfaPattern,
    pub dng_version: [u8; 4],
    /// Row-major CFA samples in stored (before orientation) order.
    pub pixels: Vec<u16>,
}

/// Tightly packed RGB8 image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedImage {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

/// Small summary returned by [`render_dng_file`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSummary {
    pub input_width: u32,
    pub input_height: u32,
    pub output_width: usize,
    pub output_height: usize,
    pub orientation: u16,
    pub gains: [f32; 3],
    pub jpeg_bytes: usize,
}

/// Render a DNG file to a JPEG file using the neutral pipeline.
pub fn render_dng_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    output: Q,
) -> Result<RenderSummary, RawRenderError> {
    let bytes = std::fs::read(input)?;
    let dng = parse_dng(&bytes)?;
    drop(bytes);

    let gains = gray_world_gains(&dng)?;
    let rgb = render_rgb_with_gains(&dng, gains)?;
    let jpeg = encode_jpeg(&rgb, JPEG_QUALITY)?;

    let summary = RenderSummary {
        input_width: dng.width,
        input_height: dng.height,
        output_width: rgb.width,
        output_height: rgb.height,
        orientation: dng.orientation,
        gains,
        jpeg_bytes: jpeg.len(),
    };

    drop(rgb);
    drop(dng);
    std::fs::write(output, &jpeg)?;
    Ok(summary)
}

/// Parse a DNG from memory and return the main (IFD0) image.
pub fn parse_dng(bytes: &[u8]) -> Result<DngImage, RawRenderError> {
    let entries = parse_ifd0(bytes)?;

    let width = scalar_u32(bytes, required(&entries, TAG_IMAGE_WIDTH, "ImageWidth")?)?;
    let height = scalar_u32(bytes, required(&entries, TAG_IMAGE_LENGTH, "ImageLength")?)?;
    if width == 0 || height == 0 {
        return Err(RawRenderError::InvalidDimensions(width, height));
    }

    let bits_entry = required(&entries, TAG_BITS_PER_SAMPLE, "BitsPerSample")?;
    let bits = unsigned_values(bytes, bits_entry)?;
    if bits.is_empty() || bits.iter().any(|&bit| bit != 16) {
        return Err(RawRenderError::Unsupported {
            what: "bit depth",
            value: format!("{bits:?}"),
            supported: "16-bit uncompressed samples only",
        });
    }

    let compression = scalar_u32(bytes, required(&entries, TAG_COMPRESSION, "Compression")?)?;
    if compression != 1 {
        return Err(RawRenderError::Unsupported {
            what: "compression",
            value: compression.to_string(),
            supported: "uncompressed (1) only",
        });
    }

    let photometric = scalar_u32(
        bytes,
        required(
            &entries,
            TAG_PHOTOMETRIC_INTERPRETATION,
            "PhotometricInterpretation",
        )?,
    )?;
    if photometric != PHOTOMETRIC_CFA {
        return Err(RawRenderError::Unsupported {
            what: "photometric interpretation",
            value: photometric.to_string(),
            supported: "CFA/Bayer (32803) only",
        });
    }

    let samples_per_pixel = match find(&entries, TAG_SAMPLES_PER_PIXEL) {
        Some(entry) => scalar_u32(bytes, entry)?,
        None => 1,
    };
    if samples_per_pixel != 1 {
        return Err(RawRenderError::Unsupported {
            what: "samples per pixel",
            value: samples_per_pixel.to_string(),
            supported: "1 (single-channel CFA) only",
        });
    }

    let orientation = match find(&entries, TAG_ORIENTATION) {
        Some(entry) => scalar_u32(bytes, entry)?,
        None => 1,
    };
    let orientation = u16::try_from(orientation).unwrap_or(0);
    if !(1..=8).contains(&orientation) {
        return Err(RawRenderError::Unsupported {
            what: "EXIF orientation",
            value: orientation.to_string(),
            supported: "1 through 8",
        });
    }

    let cfa_dim = unsigned_values(
        bytes,
        required(&entries, TAG_CFA_REPEAT_PATTERN_DIM, "CFARepeatPatternDim")?,
    )?;
    if cfa_dim.as_slice() != [2, 2] {
        return Err(RawRenderError::Unsupported {
            what: "CFA repeat pattern",
            value: format!("{cfa_dim:?}"),
            supported: "2x2 Bayer only",
        });
    }

    let cfa_entry = required(&entries, TAG_CFA_PATTERN, "CFAPattern")?;
    if cfa_entry.count != 4 {
        return Err(RawRenderError::Unsupported {
            what: "CFA pattern length",
            value: cfa_entry.count.to_string(),
            supported: "4 entries (2x2)",
        });
    }
    let cfa_bytes = entry_bytes(bytes, cfa_entry)?;
    let cfa_code = [cfa_bytes[0], cfa_bytes[1], cfa_bytes[2], cfa_bytes[3]];
    let cfa = CfaPattern::from_code(cfa_code).ok_or_else(|| RawRenderError::Unsupported {
        what: "CFA pattern",
        value: format!("{cfa_code:?}"),
        supported: "RGGB, BGGR, GRBG or GBRG",
    })?;

    let version_entry = required(&entries, TAG_DNG_VERSION, "DNGVersion")?;
    if version_entry.count != 4 {
        return Err(RawRenderError::Unsupported {
            what: "DNG version length",
            value: version_entry.count.to_string(),
            supported: "4 bytes",
        });
    }
    let version_bytes = entry_bytes(bytes, version_entry)?;
    let dng_version = [
        version_bytes[0],
        version_bytes[1],
        version_bytes[2],
        version_bytes[3],
    ];
    if dng_version[0] != 1 {
        return Err(RawRenderError::Unsupported {
            what: "DNG version",
            value: format!(
                "{}.{}.{}.{}",
                dng_version[0], dng_version[1], dng_version[2], dng_version[3]
            ),
            supported: "DNG 1.x",
        });
    }

    let black_level = optional_scalar_u32(bytes, &entries, TAG_BLACK_LEVEL, "BlackLevel", 0)?;
    let white_level = optional_scalar_u32(
        bytes,
        &entries,
        TAG_WHITE_LEVEL,
        "WhiteLevel",
        u32::from(u16::MAX),
    )?;
    if white_level <= black_level {
        return Err(RawRenderError::InvalidTiff(format!(
            "WhiteLevel ({white_level}) must be greater than BlackLevel ({black_level})"
        )));
    }

    let rows_per_strip = match find(&entries, TAG_ROWS_PER_STRIP) {
        Some(entry) => {
            let value = scalar_u32(bytes, entry)?;
            if value == 0 {
                return Err(RawRenderError::InvalidTiff(
                    "RowsPerStrip must be greater than zero".to_string(),
                ));
            }
            value.min(height)
        }
        None => height,
    };

    let strip_offsets = unsigned_values(
        bytes,
        required(&entries, TAG_STRIP_OFFSETS, "StripOffsets")?,
    )?;
    let strip_byte_counts = unsigned_values(
        bytes,
        required(&entries, TAG_STRIP_BYTE_COUNTS, "StripByteCounts")?,
    )?;
    if strip_offsets.is_empty() {
        return Err(RawRenderError::InvalidTiff(
            "DNG contains no image strips".to_string(),
        ));
    }
    if strip_offsets.len() != strip_byte_counts.len() {
        return Err(RawRenderError::InvalidTiff(format!(
            "StripOffsets has {} entries but StripByteCounts has {}",
            strip_offsets.len(),
            strip_byte_counts.len()
        )));
    }

    let strips_needed = u64::from(height).div_ceil(u64::from(rows_per_strip)) as usize;
    if strip_offsets.len() < strips_needed {
        return Err(RawRenderError::InvalidTiff(format!(
            "DNG has {} strips, expected at least {strips_needed}",
            strip_offsets.len()
        )));
    }

    let width_usize = width as usize;
    let total_samples = usize::try_from(u64::from(width) * u64::from(height))
        .map_err(|_| RawRenderError::InvalidTiff("image is too large for this platform".into()))?;
    let row_bytes = width_usize
        .checked_mul(2)
        .ok_or_else(|| RawRenderError::InvalidTiff("row byte size overflow".into()))?;
    let mut pixels = vec![0u16; total_samples];

    let mut destination_row = 0u32;
    for strip_index in 0..strips_needed {
        let rows_left = height - destination_row;
        let rows = rows_per_strip.min(rows_left) as usize;
        let expected_bytes = rows
            .checked_mul(row_bytes)
            .ok_or_else(|| RawRenderError::InvalidTiff("strip byte size overflow".into()))?;
        let byte_count = strip_byte_counts[strip_index] as usize;
        if byte_count < expected_bytes {
            return Err(RawRenderError::InvalidTiff(format!(
                "strip {strip_index} contains {byte_count} bytes, expected {expected_bytes}"
            )));
        }
        let offset = strip_offsets[strip_index] as usize;
        let strip = bytes_at(bytes, offset, expected_bytes)?;

        for row in 0..rows {
            let source = &strip[row * row_bytes..(row + 1) * row_bytes];
            let destination_start = (destination_row as usize + row) * width_usize;
            let destination = &mut pixels[destination_start..destination_start + width_usize];
            for (x, sample) in destination.iter_mut().enumerate() {
                *sample = u16::from_le_bytes([source[x * 2], source[x * 2 + 1]]);
            }
        }
        destination_row += rows as u32;
    }

    if destination_row != height {
        return Err(RawRenderError::InvalidTiff(format!(
            "strips cover {destination_row} rows, expected {height}"
        )));
    }

    Ok(DngImage {
        width,
        height,
        orientation,
        black_level,
        white_level,
        cfa,
        dng_version,
        pixels,
    })
}

/// Render a parsed DNG to a tightly packed RGB8 image.
///
/// Gray-world white balance, bilinear demosaic, orientation, gamma 2.2 and
/// clamping happen in this step. No sharpening or tone mapping is applied.
pub fn render_rgb(image: &DngImage) -> Result<RenderedImage, RawRenderError> {
    let gains = gray_world_gains(image)?;
    render_rgb_with_gains(image, gains)
}

/// Render a parsed DNG to a quality-95 JPEG.
pub fn render_dng_to_jpeg(image: &DngImage) -> Result<Vec<u8>, RawRenderError> {
    let rgb = render_rgb(image)?;
    encode_jpeg(&rgb, JPEG_QUALITY)
}

/// Compute gray-world gains from the normalized CFA means.
///
/// The green channel is the reference (gain 1.0); red and blue are scaled to
/// match it. Gains are clamped to `[0.25, 4.0]`. This is intentionally simple
/// and documented rather than a sensor-specific white balance.
pub fn gray_world_gains(image: &DngImage) -> Result<[f32; 3], RawRenderError> {
    let (width, height) = validate_image(image)?;
    let normalizer = Normalizer::new(image);
    let mut sums = [0f64; 3];
    let mut counts = [0u64; 3];

    for y in 0..height {
        for x in 0..width {
            let channel = image.cfa.color_index(x, y);
            sums[channel] += f64::from(normalizer.sample(image, x, y));
            counts[channel] += 1;
        }
    }

    let mean = |channel: usize| {
        if counts[channel] == 0 {
            0.0
        } else {
            sums[channel] / counts[channel] as f64
        }
    };
    let means = [mean(0), mean(1), mean(2)];
    let green_mean = means[1];

    let gain = |channel_mean: f64| {
        if green_mean > 1e-9 && channel_mean > 1e-9 {
            (green_mean / channel_mean).clamp(MIN_WB_GAIN, MAX_WB_GAIN) as f32
        } else {
            1.0
        }
    };

    Ok([gain(means[0]), 1.0, gain(means[2])])
}

/// Encode a rendered RGB image as JPEG.
pub fn encode_jpeg(image: &RenderedImage, quality: u8) -> Result<Vec<u8>, RawRenderError> {
    if image.width > usize::from(u16::MAX) || image.height > usize::from(u16::MAX) {
        return Err(RawRenderError::Unsupported {
            what: "JPEG dimensions",
            value: format!("{}x{}", image.width, image.height),
            supported: "dimensions up to 65535 pixels",
        });
    }

    let mut encoded = Vec::new();
    {
        let mut cursor = Cursor::new(&mut encoded);
        Encoder::new(&mut cursor, quality)
            .encode(
                &image.data,
                image.width as u16,
                image.height as u16,
                ColorType::Rgb,
            )
            .map_err(|error| RawRenderError::Jpeg(error.to_string()))?;
    }
    Ok(encoded)
}

fn render_rgb_with_gains(
    image: &DngImage,
    gains: [f32; 3],
) -> Result<RenderedImage, RawRenderError> {
    let (width, height) = validate_image(image)?;
    let (output_width, output_height) = oriented_dimensions(width, height, image.orientation);
    let pixel_count = output_width
        .checked_mul(output_height)
        .ok_or_else(|| RawRenderError::InvalidTiff("output dimensions overflow".into()))?;
    let mut data =
        vec![
            0u8;
            pixel_count
                .checked_mul(3)
                .ok_or_else(|| RawRenderError::InvalidTiff("output buffer overflow".into()))?
        ];

    let normalizer = Normalizer::new(image);
    let gamma = gamma_lut();

    for output_y in 0..output_height {
        for output_x in 0..output_width {
            let (raw_x, raw_y) = map_to_raw(output_x, output_y, width, height, image.orientation);
            let rgb = demosaic_pixel(
                image,
                &normalizer,
                raw_x,
                raw_y,
                width,
                height,
                &gains,
                &gamma,
            );
            let index = (output_y * output_width + output_x) * 3;
            data[index] = rgb[0];
            data[index + 1] = rgb[1];
            data[index + 2] = rgb[2];
        }
    }

    Ok(RenderedImage {
        width: output_width,
        height: output_height,
        data,
    })
}

fn validate_image(image: &DngImage) -> Result<(usize, usize), RawRenderError> {
    if image.width == 0 || image.height == 0 {
        return Err(RawRenderError::InvalidDimensions(image.width, image.height));
    }
    let width = image.width as usize;
    let height = image.height as usize;
    let expected = width.checked_mul(height).ok_or_else(|| {
        RawRenderError::InvalidTiff("image dimensions overflow this platform".into())
    })?;
    if image.pixels.len() != expected {
        return Err(RawRenderError::InvalidTiff(format!(
            "raw buffer has {} samples, expected {expected} for {}x{}",
            image.pixels.len(),
            image.width,
            image.height
        )));
    }
    if image.white_level <= image.black_level {
        return Err(RawRenderError::InvalidTiff(format!(
            "WhiteLevel ({}) must be greater than BlackLevel ({})",
            image.white_level, image.black_level
        )));
    }
    if !(1..=8).contains(&image.orientation) {
        return Err(RawRenderError::Unsupported {
            what: "EXIF orientation",
            value: image.orientation.to_string(),
            supported: "1 through 8",
        });
    }
    Ok((width, height))
}

#[derive(Clone, Copy)]
struct Normalizer {
    black: f32,
    scale: f32,
}

impl Normalizer {
    fn new(image: &DngImage) -> Self {
        let black = image.black_level as f32;
        let range = image.white_level as f32 - black;
        Self {
            black,
            scale: 1.0 / range,
        }
    }

    #[inline]
    fn sample(&self, image: &DngImage, x: usize, y: usize) -> f32 {
        let index = y * image.width as usize + x;
        ((image.pixels[index] as f32 - self.black) * self.scale).clamp(0.0, 1.0)
    }
}

fn gamma_lut() -> Vec<u8> {
    (0..=u16::MAX)
        .map(|value| {
            let linear = f32::from(value) / f32::from(u16::MAX);
            (linear.powf(1.0 / OUTPUT_GAMMA) * 255.0 + 0.5).clamp(0.0, 255.0) as u8
        })
        .collect()
}

#[inline]
fn oriented_dimensions(width: usize, height: usize, orientation: u16) -> (usize, usize) {
    match orientation {
        5..=8 => (height, width),
        _ => (width, height),
    }
}

#[inline]
fn map_to_raw(x: usize, y: usize, width: usize, height: usize, orientation: u16) -> (usize, usize) {
    match orientation {
        1 => (x, y),
        2 => (width - 1 - x, y),
        3 => (width - 1 - x, height - 1 - y),
        4 => (x, height - 1 - y),
        5 => (y, x),
        6 => (y, height - 1 - x),
        7 => (width - 1 - y, height - 1 - x),
        8 => (width - 1 - y, x),
        // validate_image rejects anything outside 1..=8; keep a safe fallback.
        _ => (x, y),
    }
}

#[allow(clippy::too_many_arguments)]
#[inline]
fn demosaic_pixel(
    image: &DngImage,
    normalizer: &Normalizer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    gains: &[f32; 3],
    gamma: &[u8],
) -> [u8; 3] {
    let center_channel = image.cfa.color_index(x, y);
    let mut channels = [0f32; 3];
    channels[center_channel] = normalizer.sample(image, x, y);

    // Bilinear interpolation: the measured channel is kept as-is; every missing
    // channel is the average of all samples of that colour in the 3x3 window.
    // Depending on the CFA phase that is 2 samples (red/blue at a green site)
    // or 4 samples (green at red/blue, red/blue at the opposite site).
    for channel in 0..3 {
        if channel == center_channel {
            continue;
        }
        let mut sum = 0.0f32;
        let mut count = 0u32;
        for dy in -1i32..=1 {
            let neighbor_y = y as isize + dy as isize;
            if neighbor_y < 0 || neighbor_y >= height as isize {
                continue;
            }
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let neighbor_x = x as isize + dx as isize;
                if neighbor_x < 0 || neighbor_x >= width as isize {
                    continue;
                }
                let neighbor_x = neighbor_x as usize;
                let neighbor_y = neighbor_y as usize;
                if image.cfa.color_index(neighbor_x, neighbor_y) == channel {
                    sum += normalizer.sample(image, neighbor_x, neighbor_y);
                    count += 1;
                }
            }
        }
        channels[channel] = if count > 0 {
            sum / count as f32
        } else {
            channels[center_channel]
        };
    }

    let mut output = [0u8; 3];
    for channel in 0..3 {
        let value = (channels[channel] * gains[channel]).clamp(0.0, 1.0);
        let lut_index = (value * f32::from(u16::MAX) + 0.5) as usize;
        output[channel] = gamma[lut_index];
    }
    output
}

#[derive(Clone, Copy, Debug)]
struct IfdEntry {
    tag: u16,
    field_type: u16,
    count: u32,
    value_pos: usize,
    data_len: usize,
}

fn parse_ifd0(bytes: &[u8]) -> Result<Vec<IfdEntry>, RawRenderError> {
    if bytes.len() < 8 {
        return Err(invalid_tiff(format!(
            "file is too short for a TIFF header ({} bytes)",
            bytes.len()
        )));
    }

    match &bytes[0..2] {
        b"II" => {}
        b"MM" => {
            return Err(RawRenderError::Unsupported {
                what: "TIFF byte order",
                value: "big-endian (MM)".to_string(),
                supported: "little-endian (II) only",
            })
        }
        _ => {
            return Err(invalid_tiff(
                "missing TIFF byte-order marker (expected II)".to_string(),
            ))
        }
    }

    let magic = read_u16(bytes, 2)?;
    if magic != TIFF_MAGIC {
        return Err(if magic == 43 {
            RawRenderError::Unsupported {
                what: "TIFF variant",
                value: "BigTIFF".to_string(),
                supported: "classic TIFF/DNG",
            }
        } else {
            invalid_tiff(format!(
                "invalid TIFF magic {magic} (expected {TIFF_MAGIC})"
            ))
        });
    }

    let ifd_offset = read_u32(bytes, 4)? as usize;
    let entry_count = read_u16(bytes, ifd_offset)? as usize;
    let table_size = entry_count
        .checked_mul(IFD_ENTRY_SIZE)
        .and_then(|size| size.checked_add(6))
        .ok_or_else(|| invalid_tiff("IFD size overflow".to_string()))?;
    let table_end = ifd_offset
        .checked_add(table_size)
        .ok_or_else(|| invalid_tiff("IFD offset overflow".to_string()))?;
    if table_end > bytes.len() {
        return Err(invalid_tiff(format!(
            "IFD at offset {ifd_offset} with {entry_count} entries exceeds the file ({} bytes)",
            bytes.len()
        )));
    }

    let mut entries = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        let entry_pos = ifd_offset + 2 + index * IFD_ENTRY_SIZE;
        let tag = read_u16(bytes, entry_pos)?;
        let field_type = read_u16(bytes, entry_pos + 2)?;
        let count = read_u32(bytes, entry_pos + 4)?;
        let value_field = read_u32(bytes, entry_pos + 8)? as usize;

        let (value_pos, data_len) = match field_type_size(field_type) {
            Some(element_size) => {
                let data_len = usize::try_from(
                    element_size
                        .checked_mul(u64::from(count))
                        .ok_or_else(|| invalid_tiff("tag data size overflow".to_string()))?,
                )
                .map_err(|_| invalid_tiff("tag data size overflow".to_string()))?;
                let value_pos = if data_len <= 4 {
                    entry_pos + 8
                } else {
                    value_field
                };
                // Validate the payload now so a corrupt tag cannot later cause
                // an out-of-bounds read.
                bytes_at(bytes, value_pos, data_len)?;
                (value_pos, data_len)
            }
            // Unknown field types are kept only so a required tag can produce a
            // clear "unsupported field type" error on access.
            None => (value_field, 0),
        };

        entries.push(IfdEntry {
            tag,
            field_type,
            count,
            value_pos,
            data_len,
        });
    }

    Ok(entries)
}

fn find(entries: &[IfdEntry], tag: u16) -> Option<&IfdEntry> {
    entries.iter().find(|entry| entry.tag == tag)
}

fn required<'a>(
    entries: &'a [IfdEntry],
    tag: u16,
    name: &'static str,
) -> Result<&'a IfdEntry, RawRenderError> {
    find(entries, tag).ok_or(RawRenderError::MissingTag { tag, name })
}

fn optional_scalar_u32(
    bytes: &[u8],
    entries: &[IfdEntry],
    tag: u16,
    name: &'static str,
    default: u32,
) -> Result<u32, RawRenderError> {
    let Some(entry) = find(entries, tag) else {
        return Ok(default);
    };
    if entry.count != 1 {
        return Err(RawRenderError::Unsupported {
            what: "scalar DNG tag layout",
            value: format!("{name} has {} values", entry.count),
            supported: "a single scalar value",
        });
    }
    scalar_u32(bytes, entry)
}

fn scalar_u32(bytes: &[u8], entry: &IfdEntry) -> Result<u32, RawRenderError> {
    let values = unsigned_values(bytes, entry)?;
    values.first().copied().ok_or_else(|| {
        invalid_tiff(format!(
            "tag {} has no values (count {})",
            entry.tag, entry.count
        ))
    })
}

fn unsigned_values(bytes: &[u8], entry: &IfdEntry) -> Result<Vec<u32>, RawRenderError> {
    let data = entry_bytes(bytes, entry)?;
    let count = entry.count as usize;

    match entry.field_type {
        1 | 6 | 7 => {
            if data.len() < count {
                return Err(invalid_tiff(format!(
                    "tag {} has {} bytes for {count} values",
                    entry.tag,
                    data.len()
                )));
            }
            Ok(data[..count]
                .iter()
                .map(|&value| u32::from(value))
                .collect())
        }
        3 | 8 => {
            let mut values = Vec::with_capacity(count);
            for index in 0..count {
                let offset = index * 2;
                values.push(u32::from(u16::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                ])));
            }
            Ok(values)
        }
        4 | 9 => {
            let mut values = Vec::with_capacity(count);
            for index in 0..count {
                let offset = index * 4;
                values.push(u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]));
            }
            Ok(values)
        }
        5 | 10 => {
            let mut values = Vec::with_capacity(count);
            for index in 0..count {
                let offset = index * 8;
                let numerator = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                let denominator = u32::from_le_bytes([
                    data[offset + 4],
                    data[offset + 5],
                    data[offset + 6],
                    data[offset + 7],
                ]);
                if denominator == 0 {
                    return Err(invalid_tiff(format!(
                        "tag {} has a rational with a zero denominator",
                        entry.tag
                    )));
                }
                values.push(numerator / denominator);
            }
            Ok(values)
        }
        other => Err(RawRenderError::Unsupported {
            what: "TIFF field type",
            value: format!("{other} (tag {})", entry.tag),
            supported: "BYTE, SHORT, LONG and RATIONAL",
        }),
    }
}

fn entry_bytes<'a>(bytes: &'a [u8], entry: &IfdEntry) -> Result<&'a [u8], RawRenderError> {
    bytes_at(bytes, entry.value_pos, entry.data_len)
}

fn bytes_at(bytes: &[u8], offset: usize, length: usize) -> Result<&[u8], RawRenderError> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| invalid_tiff("data offset overflow".to_string()))?;
    bytes.get(offset..end).ok_or_else(|| {
        invalid_tiff(format!(
            "data range {offset}..{end} is outside the file ({} bytes)",
            bytes.len()
        ))
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, RawRenderError> {
    let data = bytes_at(bytes, offset, 2)?;
    Ok(u16::from_le_bytes([data[0], data[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, RawRenderError> {
    let data = bytes_at(bytes, offset, 4)?;
    Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}

fn field_type_size(field_type: u16) -> Option<u64> {
    match field_type {
        1 | 2 | 6 | 7 => Some(1),
        3 | 8 => Some(2),
        4 | 9 | 11 => Some(4),
        5 | 10 | 12 => Some(8),
        _ => None,
    }
}

fn invalid_tiff(message: String) -> RawRenderError {
    RawRenderError::InvalidTiff(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDng {
        width: u32,
        height: u32,
        rows_per_strip: u32,
        orientation: u16,
        black: u32,
        white: u32,
        compression: u16,
        bits: u16,
        photometric: u16,
        samples: Vec<u16>,
    }

    type OrientationCase = (u16, (usize, usize), (usize, usize), (usize, usize));

    impl TestDng {
        fn new(width: u32, height: u32, samples: Vec<u16>) -> Self {
            assert_eq!(samples.len(), (width * height) as usize);
            Self {
                width,
                height,
                rows_per_strip: height,
                orientation: 1,
                black: 64,
                white: 1023,
                compression: 1,
                bits: 16,
                photometric: PHOTOMETRIC_CFA as u16,
                samples,
            }
        }

        fn build(&self) -> Vec<u8> {
            let width = self.width as usize;
            let height = self.height as usize;
            let rows_per_strip = self.rows_per_strip.max(1) as usize;
            let strip_count = height.div_ceil(rows_per_strip);
            let row_bytes = width * 2;
            let entry_count = 15usize;
            let ifd_offset = 8usize;
            let ifd_end = ifd_offset + 2 + entry_count * IFD_ENTRY_SIZE + 4;
            let offsets_pos = ifd_end;
            let counts_pos = offsets_pos + strip_count * 4;
            let pixel_start = counts_pos + strip_count * 4;

            let mut strip_offsets = Vec::with_capacity(strip_count);
            let mut strip_counts = Vec::with_capacity(strip_count);
            let mut cursor = pixel_start;
            let mut row = 0usize;
            for _ in 0..strip_count {
                let rows = rows_per_strip.min(height - row);
                let bytes = rows * row_bytes;
                strip_offsets.push(cursor as u32);
                strip_counts.push(bytes as u32);
                cursor += bytes;
                row += rows;
            }

            struct Entry {
                tag: u16,
                field_type: u16,
                count: u32,
                field: u32,
            }

            let mut entries = vec![
                Entry {
                    tag: TAG_IMAGE_WIDTH,
                    field_type: 4,
                    count: 1,
                    field: self.width,
                },
                Entry {
                    tag: TAG_IMAGE_LENGTH,
                    field_type: 4,
                    count: 1,
                    field: self.height,
                },
                Entry {
                    tag: TAG_BITS_PER_SAMPLE,
                    field_type: 3,
                    count: 1,
                    field: u32::from(self.bits),
                },
                Entry {
                    tag: TAG_COMPRESSION,
                    field_type: 3,
                    count: 1,
                    field: u32::from(self.compression),
                },
                Entry {
                    tag: TAG_PHOTOMETRIC_INTERPRETATION,
                    field_type: 3,
                    count: 1,
                    field: u32::from(self.photometric),
                },
                Entry {
                    tag: TAG_STRIP_OFFSETS,
                    field_type: 4,
                    count: strip_count as u32,
                    // A single value is stored inline in the IFD entry; two or
                    // more values are stored in an external array.
                    field: if strip_count == 1 {
                        strip_offsets[0]
                    } else {
                        offsets_pos as u32
                    },
                },
                Entry {
                    tag: TAG_ORIENTATION,
                    field_type: 3,
                    count: 1,
                    field: u32::from(self.orientation),
                },
                Entry {
                    tag: TAG_SAMPLES_PER_PIXEL,
                    field_type: 3,
                    count: 1,
                    field: 1,
                },
                Entry {
                    tag: TAG_ROWS_PER_STRIP,
                    field_type: 4,
                    count: 1,
                    field: self.rows_per_strip.max(1),
                },
                Entry {
                    tag: TAG_STRIP_BYTE_COUNTS,
                    field_type: 4,
                    count: strip_count as u32,
                    field: if strip_count == 1 {
                        strip_counts[0]
                    } else {
                        counts_pos as u32
                    },
                },
                Entry {
                    tag: TAG_CFA_REPEAT_PATTERN_DIM,
                    field_type: 3,
                    count: 2,
                    field: u32::from_le_bytes([2, 0, 2, 0]),
                },
                Entry {
                    tag: TAG_CFA_PATTERN,
                    field_type: 1,
                    count: 4,
                    field: u32::from_le_bytes([0, 1, 1, 2]),
                },
                Entry {
                    tag: TAG_DNG_VERSION,
                    field_type: 1,
                    count: 4,
                    field: u32::from_le_bytes([1, 4, 0, 0]),
                },
                Entry {
                    tag: TAG_BLACK_LEVEL,
                    field_type: 4,
                    count: 1,
                    field: self.black,
                },
                Entry {
                    tag: TAG_WHITE_LEVEL,
                    field_type: 4,
                    count: 1,
                    field: self.white,
                },
            ];
            assert_eq!(entries.len(), entry_count);
            entries.sort_by_key(|entry| entry.tag);

            let mut output = vec![0u8; cursor];
            output[0..2].copy_from_slice(b"II");
            output[2..4].copy_from_slice(&TIFF_MAGIC.to_le_bytes());
            output[4..8].copy_from_slice(&(ifd_offset as u32).to_le_bytes());
            output[8..10].copy_from_slice(&(entry_count as u16).to_le_bytes());

            let mut position = 10;
            for entry in &entries {
                output[position..position + 2].copy_from_slice(&entry.tag.to_le_bytes());
                output[position + 2..position + 4].copy_from_slice(&entry.field_type.to_le_bytes());
                output[position + 4..position + 8].copy_from_slice(&entry.count.to_le_bytes());
                output[position + 8..position + 12].copy_from_slice(&entry.field.to_le_bytes());
                position += IFD_ENTRY_SIZE;
            }
            debug_assert_eq!(position + 4, offsets_pos);

            for (index, offset) in strip_offsets.iter().enumerate() {
                let position = offsets_pos + index * 4;
                output[position..position + 4].copy_from_slice(&offset.to_le_bytes());
            }
            for (index, count) in strip_counts.iter().enumerate() {
                let position = counts_pos + index * 4;
                output[position..position + 4].copy_from_slice(&count.to_le_bytes());
            }

            let mut position = pixel_start;
            for y in 0..height {
                for x in 0..width {
                    let sample = self.samples[y * width + x];
                    output[position..position + 2].copy_from_slice(&sample.to_le_bytes());
                    position += 2;
                }
            }
            assert_eq!(position, output.len());
            output
        }
    }

    fn render_samples(samples: Vec<u16>) -> RenderedImage {
        let document = TestDng::new(4, 4, samples);
        let dng = parse_dng(&document.build()).expect("synthetic DNG should parse");
        render_rgb(&dng).expect("synthetic DNG should render")
    }

    fn luma(image: &RenderedImage, x: usize, y: usize) -> u32 {
        let index = (y * image.width + x) * 3;
        (u32::from(image.data[index]) * 299
            + u32::from(image.data[index + 1]) * 587
            + u32::from(image.data[index + 2]) * 114)
            / 1000
    }

    #[test]
    fn parses_core_tags_and_multiple_strips() {
        let samples: Vec<u16> = (0..16).map(|index| 100 + index * 10).collect();
        let mut document = TestDng::new(4, 4, samples.clone());
        document.rows_per_strip = 2;
        document.orientation = 6;

        let dng = parse_dng(&document.build()).expect("DNG should parse");

        assert_eq!(dng.width, 4);
        assert_eq!(dng.height, 4);
        assert_eq!(dng.orientation, 6);
        assert_eq!(dng.black_level, 64);
        assert_eq!(dng.white_level, 1023);
        assert_eq!(dng.cfa, CfaPattern::Rggb);
        assert_eq!(dng.dng_version, [1, 4, 0, 0]);
        assert_eq!(dng.pixels, samples);
    }

    #[test]
    fn rejects_unsupported_compression_bit_depth_and_photometric() {
        let mut compression = TestDng::new(4, 4, vec![128; 16]);
        compression.compression = 7;
        let error = parse_dng(&compression.build()).expect_err("compression must be rejected");
        assert!(error.to_string().contains("compression"), "{error}");

        let mut bit_depth = TestDng::new(4, 4, vec![128; 16]);
        bit_depth.bits = 12;
        let error = parse_dng(&bit_depth.build()).expect_err("bit depth must be rejected");
        assert!(error.to_string().contains("bit depth"), "{error}");

        let mut photometric = TestDng::new(4, 4, vec![128; 16]);
        photometric.photometric = 34892;
        let error = parse_dng(&photometric.build()).expect_err("photometric must be rejected");
        assert!(
            error.to_string().contains("photometric interpretation"),
            "{error}"
        );
    }

    #[test]
    fn normalization_maps_black_to_zero_and_white_to_255() {
        let black = render_samples(vec![64; 16]);
        assert!(
            black.data.iter().all(|&value| value == 0),
            "{:?}",
            black.data
        );

        let white = render_samples(vec![1023; 16]);
        assert!(
            white.data.iter().all(|&value| value == 255),
            "{:?}",
            white.data
        );

        let mid = render_samples(vec![544; 16]);
        let expected =
            ((((544.0 - 64.0) / (1023.0 - 64.0)) as f32).powf(1.0 / 2.2) * 255.0).round() as u8;
        assert!(
            mid.data.iter().all(|&value| value.abs_diff(expected) <= 1),
            "expected about {expected}, got {:?}",
            mid.data
        );
    }

    #[test]
    fn gray_world_neutralizes_equal_channel_means() {
        // Red samples are twice as bright as green/blue in the raw data.
        let mut samples = vec![0u16; 16];
        for y in 0..4 {
            for x in 0..4 {
                samples[y * 4 + x] = match CfaPattern::Rggb.color_index(x, y) {
                    0 => 800,
                    _ => 400,
                };
            }
        }

        let document = TestDng::new(4, 4, samples);
        let dng = parse_dng(&document.build()).expect("DNG should parse");
        let gains = gray_world_gains(&dng).expect("gains should compute");

        assert!((gains[0] - 336.0 / 736.0).abs() < 1e-5, "{gains:?}");
        assert!((gains[1] - 1.0).abs() < 1e-6, "{gains:?}");
        assert!((gains[2] - 1.0).abs() < 1e-6, "{gains:?}");

        let rendered = render_rgb(&dng).expect("DNG should render");
        let expected =
            ((((400.0 - 64.0) / (1023.0 - 64.0)) as f32).powf(1.0 / 2.2) * 255.0).round() as u8;

        for index in (0..rendered.data.len()).step_by(3) {
            let pixel = &rendered.data[index..index + 3];
            assert!(
                pixel[0].abs_diff(pixel[1]) <= 2 && pixel[1].abs_diff(pixel[2]) <= 2,
                "pixel is not neutral after gray-world WB: {pixel:?}"
            );
            assert!(
                pixel.iter().all(|&value| value.abs_diff(expected) <= 2),
                "expected about {expected}, got {pixel:?}"
            );
        }
    }

    #[test]
    fn orientations_1_3_6_8_are_applied() {
        // 4x2 RGGB with a bright 2x2 block in the stored top-left corner.
        let mut samples = vec![64u16; 8];
        for y in 0..2 {
            for x in 0..2 {
                samples[y * 4 + x] = 1000;
            }
        }

        let cases: [OrientationCase; 4] = [
            (1, (4, 2), (0, 0), (3, 1)),
            (3, (4, 2), (3, 1), (0, 0)),
            (6, (2, 4), (1, 0), (0, 3)),
            (8, (2, 4), (0, 3), (1, 0)),
        ];

        for (orientation, dimensions, marker, opposite) in cases {
            let mut document = TestDng::new(4, 2, samples.clone());
            document.orientation = orientation;
            let dng = parse_dng(&document.build()).expect("DNG should parse");
            let rendered = render_rgb(&dng).expect("DNG should render");

            assert_eq!(
                (rendered.width, rendered.height),
                dimensions,
                "orientation {orientation}"
            );
            let marker_luma = luma(&rendered, marker.0, marker.1);
            let opposite_luma = luma(&rendered, opposite.0, opposite.1);
            assert!(
                marker_luma > 180,
                "orientation {orientation}: marker luma {marker_luma}"
            );
            assert!(
                opposite_luma < 60,
                "orientation {orientation}: opposite luma {opposite_luma}"
            );
        }
    }

    #[test]
    fn jpeg_has_soi_and_eoi_markers() {
        let document = TestDng::new(4, 4, vec![544; 16]);
        let dng = parse_dng(&document.build()).expect("DNG should parse");
        let jpeg = render_dng_to_jpeg(&dng).expect("JPEG should encode");

        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
        assert_eq!(&jpeg[jpeg.len() - 2..], &[0xFF, 0xD9]);
    }
}
