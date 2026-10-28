//! Adobe DNG (Digital Negative) 1.4 Linear RAW File Generator
//! Produces 100% compliant Adobe DNG 1.4 files for Sony LYT-808 / LYT-600 / JN5 sensors.
//! Compatible with Adobe Lightroom, Photoshop Camera Raw, RawTherapee, and Snapseed.

#![allow(clippy::vec_init_then_push, clippy::manual_is_multiple_of)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Writes a 16-bit Bayer RAW image to an authentic Adobe DNG 1.4 file
#[allow(clippy::too_many_arguments)]
pub fn write_dng_16bit<P: AsRef<Path>>(
    path: P,
    width: u32,
    height: u32,
    raw_pixels_16bit: &[u16],
    model_name: &str,
    black_level: u32,
    white_level: u32,
    orientation: u16,
    // 2x2 CFA pattern as top-left, top-right, bottom-left, bottom-right.
    // 0 = R, 1 = G, 2 = B. Must match the delivered buffer, not the sensor
    // datasheet: the binned 12 MP stream is RGGB, the 50 MP remosaic is BGGR.
    cfa_pattern: [u8; 4],
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

    // Generic sRGB D65 fallback. DNG ColorMatrix1 maps XYZ -> camera native;
    // this is the inverse of the public sRGB->XYZ matrix, not a per-sensor
    // calibration. Accurate color still requires a user-supplied ColorMatrix.
    let color_matrix_rationals: [(i32, i32); 9] = [
        (32_404_542, 10_000_000),
        (-15_371_385, 10_000_000),
        (-4_985_314, 10_000_000),
        (-9_692_660, 10_000_000),
        (18_760_108, 10_000_000),
        (415_560, 10_000_000),
        (556_434, 10_000_000),
        (-2_040_259, 10_000_000),
        (10_572_252, 10_000_000),
    ];

    // Data sizes
    let pixel_byte_count = width * height * 2;

    // IFD Tag Entry Struct
    struct IfdEntry {
        tag: u16,
        field_type: u16, // 1=BYTE, 2=ASCII, 3=SHORT, 4=LONG, 5=RATIONAL
        count: u32,
        value_or_offset: u32,
        extra_data: Vec<u8>,
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
        value_or_offset: u32::from_le_bytes(cfa_pattern),
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
    // Tag: BlackLevel (0xC61A)
    entries.push(IfdEntry {
        tag: 0xC61A,
        field_type: 4,
        count: 1,
        value_or_offset: black_level,
        extra_data: vec![],
    });
    // Tag: WhiteLevel (0xC61D)
    entries.push(IfdEntry {
        tag: 0xC61D,
        field_type: 4,
        count: 1,
        value_or_offset: white_level,
        extra_data: vec![],
    });
    // Tag: CalibrationIlluminant1 (0xC65A) = 21 (D65)
    entries.push(IfdEntry {
        tag: 0xC65A,
        field_type: 3,
        count: 1,
        value_or_offset: 21,
        extra_data: vec![],
    });

    // Tag: ColorMatrix1 (0xC621) = 9 RATIONAL values
    let mut matrix_bytes = Vec::with_capacity(72);
    for (num, den) in color_matrix_rationals {
        matrix_bytes.extend_from_slice(&num.to_le_bytes());
        matrix_bytes.extend_from_slice(&den.to_le_bytes());
    }
    entries.push(IfdEntry {
        tag: 0xC621,
        field_type: 10,
        count: 9,
        value_or_offset: 0,
        extra_data: matrix_bytes,
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
        let padding_val = black_level as u16;
        for _ in written_samples..total_samples {
            writer.write_all(&padding_val.to_le_bytes())?;
        }
    }

    writer.flush()?;
    Ok(())
}
