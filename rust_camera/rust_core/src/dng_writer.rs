//! Adobe DNG 1.4 CFA writer.
//!
//! Colour tags are taken from the sensor and the capture result instead of a
//! generic sRGB guess: `ColorMatrix1/2`, `ForwardMatrix1/2`,
//! `CameraCalibration1/2`, `CalibrationIlluminant1/2`, `AsShotNeutral`, a
//! per-CFA `BlackLevel` and `ActiveArea`/`DefaultCrop`. Writing invented values
//! is what made converters fall back to their own white balance and render
//! green/cyan casts.

#![allow(clippy::vec_init_then_push, clippy::manual_is_multiple_of)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::dng_color::DngColor;

/// Writes a 16-bit Bayer RAW image to an Adobe DNG 1.4 file.
#[allow(clippy::too_many_arguments)]
pub fn write_dng_16bit<P: AsRef<Path>>(
    path: P,
    width: u32,
    height: u32,
    raw_pixels_16bit: &[u16],
    model_name: &str,
    orientation: u16,
    color: &DngColor,
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 1. TIFF Little-Endian Header
    writer.write_all(b"II")?; // 'II' Little-Endian
    writer.write_all(&42u16.to_le_bytes())?; // TIFF Magic 42
    writer.write_all(&8u32.to_le_bytes())?; // Offset to IFD0 (8 bytes from start)

    // 2. Prepare tag values and strings
    let make_str = b"OnePlus\0";
    let model_str = format!("{}\0", model_name);
    let camera_unique = format!("OnePlus 13 ({})\0", model_name);
    let software_str = b"MasterCam Pure Rust Engine v1.0\0";

    // Data sizes
    let pixel_byte_count = width * height * 2;

    // IFD Tag Entry Struct
    struct IfdEntry {
        tag: u16,
        field_type: u16, // 1=BYTE, 2=ASCII, 3=SHORT, 4=LONG, 5=RATIONAL, 10=SRATIONAL, 12=DOUBLE
        count: u32,
        value_or_offset: u32,
        extra_data: Vec<u8>,
    }

    /// Encodes an `f64` as a rational with a fixed denominator. 1e6 keeps ~6
    /// decimal digits, which is far more than the HAL's own precision.
    fn rational_pair(value: f64) -> (i32, i32) {
        const DEN: i64 = 1_000_000;
        let scaled = (value * DEN as f64).round() as i64;
        let clamped = scaled.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        (clamped, DEN as i32)
    }

    fn signed_rationals(tag: u16, values: &[f64]) -> IfdEntry {
        let mut bytes = Vec::with_capacity(values.len() * 8);
        for value in values {
            let (num, den) = rational_pair(*value);
            bytes.extend_from_slice(&num.to_le_bytes());
            bytes.extend_from_slice(&den.to_le_bytes());
        }
        IfdEntry {
            tag,
            field_type: 10, // SRATIONAL
            count: values.len() as u32,
            value_or_offset: 0,
            extra_data: bytes,
        }
    }

    fn unsigned_rationals(tag: u16, values: &[f64]) -> IfdEntry {
        let mut bytes = Vec::with_capacity(values.len() * 8);
        for value in values {
            let (num, den) = rational_pair(*value);
            bytes.extend_from_slice(&num.max(0).to_le_bytes());
            bytes.extend_from_slice(&den.to_le_bytes());
        }
        IfdEntry {
            tag,
            field_type: 5, // RATIONAL
            count: values.len() as u32,
            value_or_offset: 0,
            extra_data: bytes,
        }
    }

    fn matrix_values(m: &[[f64; 3]; 3]) -> [f64; 9] {
        [
            m[0][0], m[0][1], m[0][2], m[1][0], m[1][1], m[1][2], m[2][0], m[2][1], m[2][2],
        ]
    }

    let mut entries: Vec<IfdEntry> = Vec::new();

    // Tag: NewSubfileType (0x00FE) = 0
    entries.push(IfdEntry {
        tag: 0x00FE,
        field_type: 4,
        count: 1,
        value_or_offset: 0,
        extra_data: vec![],
    });
    // Tag: ImageWidth (0x0100)
    entries.push(IfdEntry {
        tag: 0x0100,
        field_type: 4,
        count: 1,
        value_or_offset: width,
        extra_data: vec![],
    });
    // Tag: ImageLength (0x0101)
    entries.push(IfdEntry {
        tag: 0x0101,
        field_type: 4,
        count: 1,
        value_or_offset: height,
        extra_data: vec![],
    });
    // Tag: BitsPerSample (0x0102) = 16
    entries.push(IfdEntry {
        tag: 0x0102,
        field_type: 3,
        count: 1,
        value_or_offset: 16,
        extra_data: vec![],
    });
    // Tag: Compression (0x0103) = 1 (Uncompressed)
    entries.push(IfdEntry {
        tag: 0x0103,
        field_type: 3,
        count: 1,
        value_or_offset: 1,
        extra_data: vec![],
    });
    // Tag: PhotometricInterpretation (0x0106) = 32803 (CFA Bayer)
    entries.push(IfdEntry {
        tag: 0x0106,
        field_type: 3,
        count: 1,
        value_or_offset: 32803,
        extra_data: vec![],
    });
    // Tag: Make (0x010F)
    entries.push(IfdEntry {
        tag: 0x010F,
        field_type: 2,
        count: make_str.len() as u32,
        value_or_offset: 0,
        extra_data: make_str.to_vec(),
    });
    // Tag: Model (0x0110)
    entries.push(IfdEntry {
        tag: 0x0110,
        field_type: 2,
        count: model_str.len() as u32,
        value_or_offset: 0,
        extra_data: model_str.as_bytes().to_vec(),
    });
    // Tag: StripOffsets (0x0111) - placeholder, calculated below
    entries.push(IfdEntry {
        tag: 0x0111,
        field_type: 4,
        count: 1,
        value_or_offset: 0,
        extra_data: vec![],
    });
    // Tag: Orientation (0x0112): 6 = 90° CW, 8 = 90° CCW.
    entries.push(IfdEntry {
        tag: 0x0112,
        field_type: 3,
        count: 1,
        value_or_offset: orientation as u32,
        extra_data: vec![],
    });
    // Tag: SamplesPerPixel (0x0115) = 1 (Raw Bayer channel)
    entries.push(IfdEntry {
        tag: 0x0115,
        field_type: 3,
        count: 1,
        value_or_offset: 1,
        extra_data: vec![],
    });
    // Tag: RowsPerStrip (0x0116) = height
    entries.push(IfdEntry {
        tag: 0x0116,
        field_type: 4,
        count: 1,
        value_or_offset: height,
        extra_data: vec![],
    });
    // Tag: StripByteCounts (0x0117) = byte_count
    entries.push(IfdEntry {
        tag: 0x0117,
        field_type: 4,
        count: 1,
        value_or_offset: pixel_byte_count,
        extra_data: vec![],
    });
    // Tag: Software (0x0131)
    entries.push(IfdEntry {
        tag: 0x0131,
        field_type: 2,
        count: software_str.len() as u32,
        value_or_offset: 0,
        extra_data: software_str.to_vec(),
    });
    // Tag: PlanarConfiguration (0x011C) = 1 (chunky)
    entries.push(IfdEntry {
        tag: 0x011C,
        field_type: 3,
        count: 1,
        value_or_offset: 1,
        extra_data: vec![],
    });
    // Tags: XResolution/YResolution (0x011A/0x011B) = 72/1, ResolutionUnit
    // (0x0128) = 2 (inch). AOSP's DngCreator writes the same.
    entries.push(unsigned_rationals(0x011A, &[72.0]));
    entries.push(unsigned_rationals(0x011B, &[72.0]));
    entries.push(IfdEntry {
        tag: 0x0128,
        field_type: 3,
        count: 1,
        value_or_offset: 2,
        extra_data: vec![],
    });
    // Tag: CFAPlaneColor (0xC616) = [R, G, B]
    entries.push(IfdEntry {
        tag: 0xC616,
        field_type: 1,
        count: 3,
        value_or_offset: 0x0002_0100,
        extra_data: vec![],
    });
    // Tag: CFALayout (0xC617) = 1 (rectangular)
    entries.push(IfdEntry {
        tag: 0xC617,
        field_type: 3,
        count: 1,
        value_or_offset: 1,
        extra_data: vec![],
    });
    // Tag: CFARepeatPatternDim (0x828D) = [2, 2]
    entries.push(IfdEntry {
        tag: 0x828D,
        field_type: 3,
        count: 2,
        value_or_offset: (2u32 << 16) | 2u32,
        extra_data: vec![],
    });
    // Tag: CFAPattern (0x828E), little-endian packed [b0, b1, b2, b3].
    entries.push(IfdEntry {
        tag: 0x828E,
        field_type: 1,
        count: 4,
        value_or_offset: u32::from_le_bytes(color.cfa_pattern),
        extra_data: vec![],
    });
    // Tag: DNGVersion (0xC612) = [1, 4, 0, 0]
    entries.push(IfdEntry {
        tag: 0xC612,
        field_type: 1,
        count: 4,
        value_or_offset: 0x00000401,
        extra_data: vec![],
    });
    // Tag: DNGBackwardVersion (0xC613) = [1, 1, 0, 0]
    entries.push(IfdEntry {
        tag: 0xC613,
        field_type: 1,
        count: 4,
        value_or_offset: 0x00000101,
        extra_data: vec![],
    });
    // Tag: UniqueCameraModel (0xC614)
    entries.push(IfdEntry {
        tag: 0xC614,
        field_type: 2,
        count: camera_unique.len() as u32,
        value_or_offset: 0,
        extra_data: camera_unique.as_bytes().to_vec(),
    });
    // Tag: BlackLevelRepeatDim (0xC619) = [2, 2]
    entries.push(IfdEntry {
        tag: 0xC619,
        field_type: 3,
        count: 2,
        value_or_offset: (2u32 << 16) | 2u32,
        extra_data: vec![],
    });
    // Tag: BlackLevel (0xC61A), one rational per CFA phase.
    entries.push(unsigned_rationals(0xC61A, &color.black_level));
    // Tag: WhiteLevel (0xC61D)
    entries.push(IfdEntry {
        tag: 0xC61D,
        field_type: 4,
        count: 1,
        value_or_offset: color.white_level,
        extra_data: vec![],
    });
    // Tag: DefaultScale (0xC61E) = [1/1, 1/1]
    entries.push(unsigned_rationals(0xC61E, &[1.0, 1.0]));
    // Tag: DefaultCropOrigin (0xC61F) and DefaultCropSize (0xC620)
    entries.push(IfdEntry {
        tag: 0xC61F,
        field_type: 4,
        count: 2,
        value_or_offset: 0,
        extra_data: color
            .crop_origin
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect(),
    });
    entries.push(IfdEntry {
        tag: 0xC620,
        field_type: 4,
        count: 2,
        value_or_offset: 0,
        extra_data: color
            .crop_size
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect(),
    });
    // Tag: ColorMatrix1/2 (0xC621/0xC622): XYZ -> camera native, as reported by
    // the sensor. This is exactly what AOSP's DngCreator writes.
    entries.push(signed_rationals(
        0xC621,
        &matrix_values(&color.color_matrix1),
    ));
    if let Some(m) = color.color_matrix2 {
        entries.push(signed_rationals(0xC622, &matrix_values(&m)));
    }
    // Tag: CameraCalibration1/2 (0xC623/0xC624)
    if let Some(m) = color.calibration1 {
        entries.push(signed_rationals(0xC623, &matrix_values(&m)));
    }
    if let Some(m) = color.calibration2 {
        entries.push(signed_rationals(0xC624, &matrix_values(&m)));
    }
    // Tag: AsShotNeutral (0xC628): camera-native RGB of a neutral subject.
    if let Some(neutral) = color.as_shot_neutral {
        entries.push(unsigned_rationals(0xC628, &neutral));
    }
    // Tag: NoiseProfile (0xC62F), S/O pairs per colour plane.
    if let Some(noise) = color.noise_profile {
        let mut bytes = Vec::with_capacity(48);
        for value in noise {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        entries.push(IfdEntry {
            tag: 0xC62F,
            field_type: 12, // DOUBLE
            count: 6,
            value_or_offset: 0,
            extra_data: bytes,
        });
    }
    // Tag: ForwardMatrix1/2 (0xC714/0xC715)
    if let Some(m) = color.forward_matrix1 {
        entries.push(signed_rationals(0xC714, &matrix_values(&m)));
    }
    if let Some(m) = color.forward_matrix2 {
        entries.push(signed_rationals(0xC715, &matrix_values(&m)));
    }
    // Tag: CalibrationIlluminant1/2 (0xC65A/0xC65B)
    entries.push(IfdEntry {
        tag: 0xC65A,
        field_type: 3,
        count: 1,
        value_or_offset: color.illuminant1 as u32,
        extra_data: vec![],
    });
    if let Some(illuminant2) = color.illuminant2 {
        entries.push(IfdEntry {
            tag: 0xC65B,
            field_type: 3,
            count: 1,
            value_or_offset: illuminant2 as u32,
            extra_data: vec![],
        });
    }
    // Tag: ActiveArea (0xC68D) = [top, left, bottom, right]
    entries.push(IfdEntry {
        tag: 0xC68D,
        field_type: 4,
        count: 4,
        value_or_offset: 0,
        extra_data: color
            .active_area
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect(),
    });

    // Sort entries by tag ID as required by TIFF / DNG spec
    entries.sort_by_key(|e| e.tag);

    // Calculate layout
    let num_entries = entries.len() as u16;
    let ifd_size = 2 + (entries.len() * 12) + 4; // count + entries + next_ifd_offset
    let mut extra_data_offset = 8 + ifd_size as u32;

    // First pass: assign offsets for extra data
    for entry in &mut entries {
        if !entry.extra_data.is_empty() {
            entry.value_or_offset = extra_data_offset;
            extra_data_offset += entry.extra_data.len() as u32;
            if extra_data_offset % 2 != 0 {
                extra_data_offset += 1; // 2-byte alignment
            }
        }
    }

    // Set pixel strip offset
    let raw_pixel_offset = extra_data_offset;
    for entry in &mut entries {
        if entry.tag == 0x0111 {
            entry.value_or_offset = raw_pixel_offset;
            break;
        }
    }

    // Write IFD count
    writer.write_all(&num_entries.to_le_bytes())?;

    // Write IFD entries
    for entry in &entries {
        writer.write_all(&entry.tag.to_le_bytes())?;
        writer.write_all(&entry.field_type.to_le_bytes())?;
        writer.write_all(&entry.count.to_le_bytes())?;
        writer.write_all(&entry.value_or_offset.to_le_bytes())?;
    }

    // Offset to next IFD: 0 (no more IFDs)
    writer.write_all(&0u32.to_le_bytes())?;

    // Write extra data payloads
    for entry in &entries {
        if !entry.extra_data.is_empty() {
            writer.write_all(&entry.extra_data)?;
            if entry.extra_data.len() % 2 != 0 {
                writer.write_all(&[0])?; // padding
            }
        }
    }

    // 3. Write Raw 16-bit CFA Bayer pixels in chunks. Writing one 2-byte
    // `write_all` per sample made 50 MP DNGs unnecessarily slow.
    let mut sample_buffer: Vec<u8> = Vec::with_capacity(64 * 1024);
    for &sample in raw_pixels_16bit {
        sample_buffer.extend_from_slice(&sample.to_le_bytes());
        if sample_buffer.len() >= 64 * 1024 {
            writer.write_all(&sample_buffer)?;
            sample_buffer.clear();
        }
    }
    if !sample_buffer.is_empty() {
        writer.write_all(&sample_buffer)?;
    }

    // Pad if pixel slice length was smaller than width * height
    let written_samples = raw_pixels_16bit.len();
    let total_samples = (width * height) as usize;
    if written_samples < total_samples {
        let padding_val = color.black_level[0] as u16;
        for _ in written_samples..total_samples {
            writer.write_all(&padding_val.to_le_bytes())?;
        }
    }

    writer.flush()?;
    Ok(())
}
