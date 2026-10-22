//! Google Motion Photo XMP Metadata Generator & Atomic File Muxer
//!
//! High-performance Rust engine for OnePlus MasterCam (PJZ110 / Snapdragon 8 Elite).
//! Packages primary JPEG + Google Motion Photo v1/v2 XMP RDF metadata into APP1
//! marker segment + 4K 60fps / 120fps MP4 micro-video stream appended at the exact
//! container offset.
//!
//! # Motion Photo Container Specification:
//! Google Motion Photo packages a still image and an MP4 video clip into a single file:
//! ┌──────────────────────────────────────────────────────────────┐
//! │ Primary JPEG Buffer                                          │
//! │  ├─ SOI (0xFF, 0xD8)                                         │
//! │  ├─ APP0 (JFIF) / APP1 (Exif)                                │
//! │  ├─ APP1 (XMP) with GCamera & Container V2 RDF metadata       │
//! │  ├─ DQT, SOF, DHT, SOS entropy scan data                     │
//! │  └─ EOI (0xFF, 0xD9)                                         │
//! ├──────────────────────────────────────────────────────────────┤
//! │ MP4 Micro-Video Stream (appended at exact container offset)  │
//! │  ├─ ISOBMFF ftyp box (length >= 8)                           │
//! │  └─ moov / mdat audio-video payload                          │
//! └──────────────────────────────────────────────────────────────┘
//!
//! References:
//! - Camera namespace: `http://ns.google.com/photos/1.0/camera/`
//! - Container schema: `http://ns.google.com/photos/1.0/container/`
//! - Item schema: `http://ns.google.com/photos/1.0/container/item/`

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use thiserror::Error;

/// Google Camera namespace URI for Motion Photo attributes
pub const XMP_NAMESPACE_CAMERA: &str = "http://ns.google.com/photos/1.0/camera/";

/// Google Container v2 directory schema URI
pub const XMP_NAMESPACE_CONTAINER: &str = "http://ns.google.com/photos/1.0/container/";

/// Google Container v2 item schema URI
pub const XMP_NAMESPACE_ITEM: &str = "http://ns.google.com/photos/1.0/container/item/";

/// Adobe XMP APP1 segment identifier signature (29 bytes including trailing null)
pub const XMP_HEADER_SIGNATURE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

// JPEG Marker constants
const JPEG_SOI: u8 = 0xD8; // Start of Image
const JPEG_EOI: u8 = 0xD9; // End of Image
const JPEG_SOS: u8 = 0xDA; // Start of Scan
const JPEG_APP0: u8 = 0xE0; // JFIF
const JPEG_APP1: u8 = 0xE1; // Exif / XMP

/// Errors encountered during Motion Photo generation, assembly, or extraction
#[derive(Debug, Error)]
pub enum MotionPhotoError {
    #[error("Invalid JPEG: {0}")]
    InvalidJpeg(String),

    #[error("Invalid MP4 micro-video stream: {0}")]
    InvalidMp4(String),

    #[error("XMP generation error: {0}")]
    XmpGenerationError(String),

    #[error("XMP metadata missing or invalid: {0}")]
    XmpMissingOrInvalid(String),

    #[error("Container offset mismatch: expected offset {expected}, but found {actual}")]
    OffsetMismatch { expected: usize, actual: usize },

    #[error("Container length mismatch: expected video length {expected}, but found {actual}")]
    LengthMismatch { expected: usize, actual: usize },

    #[error("File size {0} bytes is too small for Motion Photo container")]
    FileTooSmall(usize),

    #[error("I/O error during atomic assembly: {0}")]
    Io(#[from] std::io::Error),
}

/// A single item entry in Google Container Directory (`Container:Directory`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerItem {
    /// MIME type of the item (e.g. `image/jpeg` or `video/mp4`)
    pub mime: String,
    /// Semantic role of the item (e.g. `Primary` or `MotionPhoto`)
    pub semantic: String,
    /// Byte length of the item payload in the container (0 for primary item)
    pub length: usize,
    /// Padding bytes after the item payload
    pub padding: usize,
}

/// Parsed Google Motion Photo metadata extracted from XMP APP1 block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotionPhotoMetadata {
    /// Whether the file is marked as a Google Motion Photo (`GCamera:MotionPhoto="1"`)
    pub is_motion_photo: bool,
    /// Motion photo version (`GCamera:MotionPhotoVersion`)
    pub version: u32,
    /// Presentation timestamp of the still image in the micro-video (microseconds)
    pub presentation_timestamp_us: i64,
    /// Extracted byte length of the embedded MP4 micro-video
    pub video_length: usize,
    /// Items found in the Container:Directory sequence
    pub container_items: Vec<ContainerItem>,
}

impl MotionPhotoMetadata {
    /// Returns the primary still image item if present in container
    pub fn primary_item(&self) -> Option<&ContainerItem> {
        self.container_items
            .iter()
            .find(|i| i.semantic == "Primary" || i.mime == "image/jpeg")
    }

    /// Returns the motion photo video item if present in container
    pub fn video_item(&self) -> Option<&ContainerItem> {
        self.container_items
            .iter()
            .find(|i| i.semantic == "MotionPhoto" || i.mime == "video/mp4")
    }
}

/// Extracted Motion Photo components with borrowed slices (zero-copy)
#[derive(Debug, Clone)]
pub struct ExtractedMotionPhoto<'a> {
    /// Sliced reference to the primary JPEG image (including SOI, markers, XMP, EOI)
    pub primary_jpeg: &'a [u8],
    /// Sliced reference to the embedded MP4 micro-video stream
    pub mp4_video: &'a [u8],
    /// Parsed Motion Photo XMP metadata
    pub metadata: MotionPhotoMetadata,
    /// Raw XMP RDF/XML block extracted from the APP1 segment
    pub xmp_xml: String,
    /// Absolute byte offset in the container where the MP4 video begins
    pub video_offset: usize,
}

impl<'a> ExtractedMotionPhoto<'a> {
    /// Converts borrowed slices into an owned representation
    pub fn to_owned(&self) -> ExtractedMotionPhotoOwned {
        ExtractedMotionPhotoOwned {
            primary_jpeg: self.primary_jpeg.to_vec(),
            mp4_video: self.mp4_video.to_vec(),
            metadata: self.metadata.clone(),
            xmp_xml: self.xmp_xml.clone(),
            video_offset: self.video_offset,
        }
    }
}

/// Owned Motion Photo representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedMotionPhotoOwned {
    pub primary_jpeg: Vec<u8>,
    pub mp4_video: Vec<u8>,
    pub metadata: MotionPhotoMetadata,
    pub xmp_xml: String,
    pub video_offset: usize,
}

/// Builder and Assembler for Google Motion Photos
#[derive(Debug, Clone)]
pub struct MotionPhotoBuilder {
    pub presentation_timestamp_us: i64,
    pub version: u32,
    pub include_v1_compat: bool,
}

impl Default for MotionPhotoBuilder {
    fn default() -> Self {
        Self {
            presentation_timestamp_us: 1_500_000,
            version: 1,
            include_v1_compat: true,
        }
    }
}

impl MotionPhotoBuilder {
    /// Creates a new builder with the specified presentation timestamp in microseconds
    pub fn new(presentation_timestamp_us: i64) -> Self {
        Self {
            presentation_timestamp_us,
            version: 1,
            include_v1_compat: true,
        }
    }

    /// Sets the Motion Photo version (default: 1)
    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Sets whether to include Google Camera v1 compatibility attributes (`GCamera:MicroVideoOffset`)
    pub fn include_v1_compat(mut self, compat: bool) -> Self {
        self.include_v1_compat = compat;
        self
    }

    /// Generates compliant Google Motion Photo v1 & v2 RDF/XML block
    pub fn generate_xmp_xml(&self, video_length_bytes: usize) -> String {
        generate_motion_photo_xmp(
            self.presentation_timestamp_us,
            video_length_bytes,
            self.version,
            self.include_v1_compat,
        )
    }

    /// Assembles an atomic Motion Photo buffer in memory: Primary JPEG + XMP APP1 + Appended MP4
    pub fn assemble_motion_photo(
        &self,
        jpeg_bytes: &[u8],
        mp4_bytes: &[u8],
    ) -> Result<Vec<u8>, MotionPhotoError> {
        self.assemble(jpeg_bytes, mp4_bytes)
    }

    /// Assembles an atomic Motion Photo buffer in memory
    pub fn assemble(
        &self,
        jpeg_bytes: &[u8],
        mp4_bytes: &[u8],
    ) -> Result<Vec<u8>, MotionPhotoError> {
        // 1. Validate MP4 stream
        validate_mp4_stream(mp4_bytes)?;

        // 2. Scan and parse primary JPEG structure
        let (segments, eoi_offset) = scan_jpeg_structure(jpeg_bytes)?;

        // 3. Generate XMP RDF/XML and wrap in APP1 segment
        let xmp_xml = self.generate_xmp_xml(mp4_bytes.len());
        let app1_segment = create_xmp_app1_segment(&xmp_xml)?;

        // 4. Construct JPEG with embedded XMP APP1 segment
        let mut assembled_jpeg = Vec::with_capacity(jpeg_bytes.len() + app1_segment.len());

        let existing_xmp = segments.iter().find(|s| s.is_xmp_app1);
        if let Some(xmp_seg) = existing_xmp {
            // Replace existing XMP segment
            assembled_jpeg.extend_from_slice(&jpeg_bytes[0..xmp_seg.seg_start]);
            assembled_jpeg.extend_from_slice(&app1_segment);
            let after_xmp = xmp_seg.seg_start + xmp_seg.total_len;
            assembled_jpeg.extend_from_slice(&jpeg_bytes[after_xmp..eoi_offset + 2]);
        } else {
            // Determine optimal insertion position:
            // Standard order: SOI -> APP0 (JFIF) -> APP1 (Exif) -> [New XMP APP1] -> DQT / DHT / SOF / SOS ...
            let mut insert_pos = 2; // Right after SOI by default
            for seg in &segments {
                if seg.marker == JPEG_APP0 || seg.is_exif_app1 {
                    insert_pos = seg.seg_start + seg.total_len;
                } else {
                    break;
                }
            }

            assembled_jpeg.extend_from_slice(&jpeg_bytes[0..insert_pos]);
            assembled_jpeg.extend_from_slice(&app1_segment);
            assembled_jpeg.extend_from_slice(&jpeg_bytes[insert_pos..eoi_offset + 2]);
        }

        // Verify that the JPEG buffer terminates cleanly with EOI (0xFF, 0xD9)
        let jpeg_len = assembled_jpeg.len();
        if jpeg_len < 2
            || assembled_jpeg[jpeg_len - 2] != 0xFF
            || assembled_jpeg[jpeg_len - 1] != JPEG_EOI
        {
            return Err(MotionPhotoError::InvalidJpeg(
                "Assembled JPEG does not end with EOI (0xFFD9) marker".into(),
            ));
        }

        // 5. Append MP4 micro-video track at the exact container offset
        let mut assembled_file = assembled_jpeg;
        assembled_file.extend_from_slice(mp4_bytes);

        Ok(assembled_file)
    }

    /// Assembles and writes the Motion Photo to disk using atomic temporary file + rename.
    /// Guarantees that Android MediaScanner or background readers never see an incomplete file.
    pub fn assemble_to_file<P: AsRef<Path>>(
        &self,
        destination_path: P,
        jpeg_bytes: &[u8],
        mp4_bytes: &[u8],
    ) -> Result<(), MotionPhotoError> {
        let dest = destination_path.as_ref();
        let parent_dir = dest.parent().unwrap_or_else(|| Path::new("."));

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }

        let assembled_data = self.assemble(jpeg_bytes, mp4_bytes)?;

        // Unique temporary filename in the same directory to allow atomic fs::rename
        let pid = std::process::id();
        let rand_suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dest_filename = dest
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("motion_photo.jpg");
        let tmp_path = parent_dir.join(format!(".{}.tmp.{}.{}", dest_filename, pid, rand_suffix));

        // Write buffer and sync to disk
        let write_op = (|| -> Result<(), std::io::Error> {
            let file = File::create(&tmp_path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&assembled_data)?;
            writer.flush()?;
            let file = writer.into_inner()?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_op {
            let _ = fs::remove_file(&tmp_path);
            return Err(MotionPhotoError::Io(e));
        }

        // Atomic rename
        if let Err(e) = fs::rename(&tmp_path, dest) {
            let _ = fs::remove_file(&tmp_path);
            return Err(MotionPhotoError::Io(e));
        }

        Ok(())
    }

    /// Validates an assembled Motion Photo and extracts the embedded MP4 video slice by length
    pub fn extract_embedded_video<'a>(
        &self,
        assembled_file: &'a [u8],
        video_len: usize,
    ) -> Result<&'a [u8], MotionPhotoError> {
        if assembled_file.len() < video_len {
            return Err(MotionPhotoError::FileTooSmall(assembled_file.len()));
        }
        let start_offset = assembled_file.len() - video_len;
        Ok(&assembled_file[start_offset..])
    }
}

/// Assembles a Google Motion Photo in memory from primary JPEG and MP4 buffers
pub fn assemble_motion_photo(
    primary_jpeg: &[u8],
    mp4_video: &[u8],
    presentation_timestamp_us: i64,
) -> Result<Vec<u8>, MotionPhotoError> {
    MotionPhotoBuilder::new(presentation_timestamp_us).assemble(primary_jpeg, mp4_video)
}

/// Atomically assembles and writes a Google Motion Photo to disk
pub fn assemble_motion_photo_to_file<P: AsRef<Path>>(
    destination_path: P,
    primary_jpeg: &[u8],
    mp4_video: &[u8],
    presentation_timestamp_us: i64,
) -> Result<(), MotionPhotoError> {
    MotionPhotoBuilder::new(presentation_timestamp_us).assemble_to_file(
        destination_path,
        primary_jpeg,
        mp4_video,
    )
}

/// Generates compliant Google Motion Photo v1 & v2 RDF/XML string
pub fn generate_motion_photo_xmp(
    presentation_timestamp_us: i64,
    video_length_bytes: usize,
    version: u32,
    include_v1_compat: bool,
) -> String {
    let v1_compat_attrs = if include_v1_compat {
        format!(
            "\n        GCamera:MicroVideo=\"1\"\
             \n        GCamera:MicroVideoVersion=\"1\"\
             \n        GCamera:MicroVideoOffset=\"{}\"\
             \n        GCamera:MicroVideoPresentationTimestampUs=\"{}\"",
            video_length_bytes, presentation_timestamp_us
        )
    } else {
        String::new()
    };

    format!(
        "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.1.0-jc003\">\
\n  <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\
\n    <rdf:Description rdf:about=\"\"\
\n        xmlns:GCamera=\"http://ns.google.com/photos/1.0/camera/\"\
\n        xmlns:Container=\"http://ns.google.com/photos/1.0/container/\"\
\n        xmlns:Item=\"http://ns.google.com/photos/1.0/container/item/\"\
\n        GCamera:MotionPhoto=\"1\"\
\n        GCamera:MotionPhotoVersion=\"{}\"\
\n        GCamera:MotionPhotoPresentationTimestampUs=\"{}\"{}>\
\n      <Container:Directory>\
\n        <rdf:Seq>\
\n          <rdf:li rdf:parseType=\"Resource\">\
\n            <Container:Item\
\n                Item:Mime=\"image/jpeg\"\
\n                Item:Semantic=\"Primary\"\
\n                Item:Length=\"0\"\
\n                Item:Padding=\"0\"/>\
\n          </rdf:li>\
\n          <rdf:li rdf:parseType=\"Resource\">\
\n            <Container:Item\
\n                Item:Mime=\"video/mp4\"\
\n                Item:Semantic=\"MotionPhoto\"\
\n                Item:Length=\"{}\"\
\n                Item:Padding=\"0\"/>\
\n          </rdf:li>\
\n        </rdf:Seq>\
\n      </Container:Directory>\
\n    </rdf:Description>\
\n  </rdf:RDF>\
\n</x:xmpmeta>",
        version, presentation_timestamp_us, v1_compat_attrs, video_length_bytes
    )
}

/// Wraps an XMP XML string into a standard JPEG APP1 segment
pub fn create_xmp_app1_segment(xmp_xml: &str) -> Result<Vec<u8>, MotionPhotoError> {
    let xml_bytes = xmp_xml.as_bytes();
    let header_len = XMP_HEADER_SIGNATURE.len(); // 29 bytes
    let payload_len = header_len + xml_bytes.len();
    let total_seg_len = 2 + payload_len; // Includes the 2 length bytes

    if total_seg_len > 0xFFFF {
        return Err(MotionPhotoError::XmpGenerationError(format!(
            "XMP payload size {} exceeds standard JPEG APP1 64KB limit",
            total_seg_len
        )));
    }

    let mut segment = Vec::with_capacity(2 + total_seg_len);
    segment.push(0xFF);
    segment.push(JPEG_APP1);
    segment.extend_from_slice(&(total_seg_len as u16).to_be_bytes());
    segment.extend_from_slice(XMP_HEADER_SIGNATURE);
    segment.extend_from_slice(xml_bytes);

    Ok(segment)
}

/// Internal segment descriptor from JPEG stream parsing
#[derive(Debug)]
struct JpegSegmentInfo {
    marker: u8,
    seg_start: usize,
    total_len: usize,
    is_xmp_app1: bool,
    is_exif_app1: bool,
}

/// Scans JPEG segment markers and locates the EOI offset
fn scan_jpeg_structure(data: &[u8]) -> Result<(Vec<JpegSegmentInfo>, usize), MotionPhotoError> {
    if data.len() < 4 {
        return Err(MotionPhotoError::InvalidJpeg(
            "JPEG buffer is too short".into(),
        ));
    }
    if data[0] != 0xFF || data[1] != JPEG_SOI {
        return Err(MotionPhotoError::InvalidJpeg(format!(
            "Missing SOI (0xFFD8) marker: found 0x{:02X}{:02X}",
            data[0], data[1]
        )));
    }

    let mut segments = Vec::new();
    let mut offset = 2;
    let mut eoi_offset = None;

    while offset < data.len() {
        if data[offset] != 0xFF {
            break;
        }

        // Skip continuous fill bytes 0xFF
        let mut marker_pos = offset;
        while marker_pos < data.len() && data[marker_pos] == 0xFF {
            marker_pos += 1;
        }
        if marker_pos >= data.len() {
            break;
        }

        let marker = data[marker_pos];
        let seg_start = offset;
        offset = marker_pos + 1;

        if marker == 0x00 {
            continue;
        }

        if marker == JPEG_EOI {
            eoi_offset = Some(seg_start);
            break;
        }

        // Standalone markers without length
        if marker == JPEG_SOI || (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            segments.push(JpegSegmentInfo {
                marker,
                seg_start,
                total_len: offset - seg_start,
                is_xmp_app1: false,
                is_exif_app1: false,
            });
            continue;
        }

        // Marker with 2-byte big-endian length
        if offset + 2 > data.len() {
            return Err(MotionPhotoError::InvalidJpeg(
                "Truncated marker length field in JPEG".into(),
            ));
        }
        let seg_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        if seg_len < 2 || offset + seg_len > data.len() {
            return Err(MotionPhotoError::InvalidJpeg(format!(
                "Invalid segment length {} for marker 0x{:02X}",
                seg_len, marker
            )));
        }

        let is_xmp = if marker == JPEG_APP1 && seg_len >= 2 + XMP_HEADER_SIGNATURE.len() {
            let payload_start = offset + 2;
            &data[payload_start..payload_start + XMP_HEADER_SIGNATURE.len()] == XMP_HEADER_SIGNATURE
        } else {
            false
        };

        let is_exif = if marker == JPEG_APP1 && seg_len >= 2 + 6 {
            let payload_start = offset + 2;
            &data[payload_start..payload_start + 6] == b"Exif\0\0"
        } else {
            false
        };

        let total_len = (offset + seg_len) - seg_start;
        segments.push(JpegSegmentInfo {
            marker,
            seg_start,
            total_len,
            is_xmp_app1: is_xmp,
            is_exif_app1: is_exif,
        });

        offset += seg_len;

        if marker == JPEG_SOS {
            // SOS header followed by entropy-coded scan data. Scan for EOI (0xFF, 0xD9)
            while offset + 1 < data.len() {
                if data[offset] == 0xFF {
                    let next = data[offset + 1];
                    if next == 0x00 || (0xD0..=0xD7).contains(&next) {
                        offset += 2;
                    } else if next == 0xFF {
                        offset += 1;
                    } else if next == JPEG_EOI {
                        eoi_offset = Some(offset);
                        break;
                    } else {
                        offset += 1;
                    }
                } else {
                    offset += 1;
                }
            }
            break;
        }
    }

    let final_eoi = match eoi_offset {
        Some(pos) => pos,
        None => {
            // Fallback search backwards for 0xFF, 0xD9
            let mut found = None;
            for i in (1..data.len()).rev() {
                if data[i - 1] == 0xFF && data[i] == JPEG_EOI {
                    found = Some(i - 1);
                    break;
                }
            }
            match found {
                Some(pos) => pos,
                None => {
                    return Err(MotionPhotoError::InvalidJpeg(
                        "EOI (0xFFD9) marker not found in JPEG stream".into(),
                    ));
                }
            }
        }
    };

    Ok((segments, final_eoi))
}

/// Validates that an MP4 stream starts with a valid ISOBMFF atom/box
pub fn validate_mp4_stream(mp4_data: &[u8]) -> Result<(), MotionPhotoError> {
    if mp4_data.len() < 8 {
        return Err(MotionPhotoError::InvalidMp4(format!(
            "MP4 stream is too short: {} bytes (minimum 8 bytes required)",
            mp4_data.len()
        )));
    }

    let box_size =
        u32::from_be_bytes([mp4_data[0], mp4_data[1], mp4_data[2], mp4_data[3]]) as usize;

    let box_type = &mp4_data[4..8];
    let valid_boxes: &[&[u8; 4]] = &[b"ftyp", b"moov", b"mdat", b"free", b"skip", b"wide"];

    if !valid_boxes.iter().any(|&b| b == box_type) {
        let tag_str = String::from_utf8_lossy(box_type);
        return Err(MotionPhotoError::InvalidMp4(format!(
            "MP4 stream does not begin with a valid ISOBMFF box (found '{}')",
            tag_str
        )));
    }

    if box_size > 1 && box_size > mp4_data.len() {
        return Err(MotionPhotoError::InvalidMp4(format!(
            "First MP4 box length ({}) exceeds total stream length ({})",
            box_size,
            mp4_data.len()
        )));
    }

    Ok(())
}

/// Parses Google Motion Photo metadata from an XMP RDF/XML text
pub fn parse_motion_photo_xmp(xml: &str) -> Result<MotionPhotoMetadata, MotionPhotoError> {
    // 1. Verify Google Camera MotionPhoto flag
    let motion_photo_flag = extract_attr_or_tag(xml, "MotionPhoto").ok_or_else(|| {
        MotionPhotoError::XmpMissingOrInvalid("Missing GCamera:MotionPhoto tag".into())
    })?;

    let is_motion_photo =
        motion_photo_flag == "1" || motion_photo_flag.eq_ignore_ascii_case("true");
    if !is_motion_photo {
        return Err(MotionPhotoError::XmpMissingOrInvalid(
            "GCamera:MotionPhoto is not enabled ('1')".into(),
        ));
    }

    // 2. Motion photo version
    let version = extract_attr_or_tag(xml, "MotionPhotoVersion")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1);

    // 3. Presentation timestamp (microseconds)
    let presentation_timestamp_us = extract_attr_or_tag(xml, "MotionPhotoPresentationTimestampUs")
        .or_else(|| extract_attr_or_tag(xml, "MicroVideoPresentationTimestampUs"))
        .and_then(|ts| ts.parse::<i64>().ok())
        .unwrap_or(0);

    // 4. Parse Container:Directory items (V2)
    let container_items = parse_container_items(xml);

    // 5. Determine video length: prefer Container:Item (V2), fallback to MicroVideoOffset (V1)
    let video_len_from_container = container_items
        .iter()
        .find(|i| i.semantic == "MotionPhoto" || i.mime == "video/mp4")
        .map(|i| i.length);

    let video_len_from_v1 =
        extract_attr_or_tag(xml, "MicroVideoOffset").and_then(|s| s.parse::<usize>().ok());

    let video_length = match (video_len_from_container, video_len_from_v1) {
        (Some(len), _) if len > 0 => len,
        (_, Some(len)) if len > 0 => len,
        _ => {
            return Err(MotionPhotoError::XmpMissingOrInvalid(
                "Neither Container:Item video/mp4 Length nor GCamera:MicroVideoOffset specified"
                    .into(),
            ));
        }
    };

    Ok(MotionPhotoMetadata {
        is_motion_photo,
        version,
        presentation_timestamp_us,
        video_length,
        container_items,
    })
}

/// Extracts attribute or XML element text by tag suffix (e.g. `MotionPhoto` matches `GCamera:MotionPhoto`)
fn extract_attr_or_tag(xml: &str, tag_suffix: &str) -> Option<String> {
    extract_attr_value(xml, tag_suffix).or_else(|| extract_element_text(xml, tag_suffix))
}

/// Extracts attribute value by attribute name suffix (e.g. `foo:bar="val"` -> `"val"`)
fn extract_attr_value(xml: &str, attr_suffix: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(pos) = xml[search_from..].find(attr_suffix) {
        let abs_pos = search_from + pos;
        let prefix_ok = abs_pos == 0 || {
            let prev = xml.as_bytes()[abs_pos - 1];
            prev == b':' || prev == b' ' || prev == b'\t' || prev == b'\n' || prev == b'\r'
        };
        let after_idx = abs_pos + attr_suffix.len();
        if prefix_ok && after_idx < xml.len() {
            let rest = &xml[after_idx..];
            let trimmed = rest.trim_start();
            if let Some(after_eq) = trimmed.strip_prefix('=') {
                let val_start = after_eq.trim_start();
                if let Some(quote) = val_start.chars().next() {
                    if quote == '"' || quote == '\'' {
                        let inner = &val_start[quote.len_utf8()..];
                        if let Some(quote_end) = inner.find(quote) {
                            return Some(inner[..quote_end].to_string());
                        }
                    }
                }
            }
        }
        search_from = abs_pos + attr_suffix.len();
    }
    None
}

/// Extracts element inner text (e.g. `<Item:Mime>video/mp4</Item:Mime>` -> `"video/mp4"`)
fn extract_element_text(xml: &str, tag_suffix: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(pos) = xml[search_from..].find(tag_suffix) {
        let abs_pos = search_from + pos;
        let mut angle_pos = None;
        for i in (0..abs_pos).rev() {
            let b = xml.as_bytes()[i];
            if b == b'<' {
                angle_pos = Some(i);
                break;
            }
            if b == b'>' || b == b' ' || b == b'\t' || b == b'\n' {
                break;
            }
        }
        if angle_pos.is_some() {
            let after_tag = abs_pos + tag_suffix.len();
            if after_tag < xml.len() && xml.as_bytes()[after_tag] == b'>' {
                let content_start = after_tag + 1;
                let close_pattern = "</";
                if let Some(close_pos) = xml[content_start..].find(close_pattern) {
                    let content = &xml[content_start..content_start + close_pos];
                    return Some(content.trim().to_string());
                }
            }
        }
        search_from = abs_pos + tag_suffix.len();
    }
    None
}

/// Parses Container items from Container:Directory
fn parse_container_items(xml: &str) -> Vec<ContainerItem> {
    let mut items = Vec::new();
    let mut search_from = 0;

    // Search for <Container:Item, <rdf:li, or <Item:
    while let Some(rel_pos) = xml[search_from..]
        .find("<Container:Item")
        .or_else(|| xml[search_from..].find("<rdf:li"))
    {
        let item_start = search_from + rel_pos;
        let close_tag = if xml[item_start..].starts_with("<Container:Item") {
            ">"
        } else {
            "</rdf:li>"
        };

        if let Some(end_offset) = xml[item_start..].find(close_tag) {
            let block = &xml[item_start..item_start + end_offset + close_tag.len()];
            search_from = item_start + end_offset + close_tag.len();

            let mime = extract_attr_or_tag(block, "Mime");
            let semantic = extract_attr_or_tag(block, "Semantic");
            let length = extract_attr_or_tag(block, "Length")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let padding = extract_attr_or_tag(block, "Padding")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            if let Some(mime_str) = mime {
                items.push(ContainerItem {
                    mime: mime_str,
                    semantic: semantic.unwrap_or_default(),
                    length,
                    padding,
                });
            }
        } else {
            break;
        }
    }

    items
}

/// Extractor and Validator: reads an assembled file and verifies the JPEG,
/// the XMP header, and the embedded MP4 stream length.
pub fn extract_motion_photo<'a>(
    assembled_data: &'a [u8],
) -> Result<ExtractedMotionPhoto<'a>, MotionPhotoError> {
    if assembled_data.len() < 32 {
        return Err(MotionPhotoError::FileTooSmall(assembled_data.len()));
    }

    // 1. Scan JPEG structure & locate XMP APP1 segment
    let (segments, eoi_offset) = scan_jpeg_structure(assembled_data)?;

    let xmp_segment = segments.iter().find(|s| s.is_xmp_app1).ok_or_else(|| {
        MotionPhotoError::XmpMissingOrInvalid("No XMP APP1 metadata segment found in JPEG".into())
    })?;

    // 2. Extract XMP XML string from APP1 payload
    let header_len = XMP_HEADER_SIGNATURE.len();
    // Segment layout: [0xFF, 0xE1, len_hi, len_lo, signature (29 bytes), XML text...]
    let xml_start = xmp_segment.seg_start + 4 + header_len;
    let xml_end = xmp_segment.seg_start + xmp_segment.total_len;
    if xml_start >= xml_end || xml_end > assembled_data.len() {
        return Err(MotionPhotoError::XmpMissingOrInvalid(
            "Malformed XMP APP1 segment boundaries".into(),
        ));
    }

    let xmp_xml_str = std::str::from_utf8(&assembled_data[xml_start..xml_end]).map_err(|e| {
        MotionPhotoError::XmpMissingOrInvalid(format!("Invalid UTF-8 in XMP: {}", e))
    })?;

    // 3. Parse and validate XMP metadata
    let metadata = parse_motion_photo_xmp(xmp_xml_str)?;

    // 4. Validate embedded MP4 stream length and container offset
    let video_len = metadata.video_length;
    let total_len = assembled_data.len();

    if total_len <= video_len {
        return Err(MotionPhotoError::FileTooSmall(total_len));
    }

    let expected_video_offset = total_len - video_len;

    // Verify container boundary: the JPEG EOI (0xFF, 0xD9) must be located right before video_offset
    let jpeg_eoi_end = eoi_offset + 2;
    if jpeg_eoi_end != expected_video_offset {
        return Err(MotionPhotoError::OffsetMismatch {
            expected: expected_video_offset,
            actual: jpeg_eoi_end,
        });
    }

    // 5. Slice JPEG and MP4 buffers
    let primary_jpeg = &assembled_data[..expected_video_offset];
    let mp4_video = &assembled_data[expected_video_offset..];

    if mp4_video.len() != video_len {
        return Err(MotionPhotoError::LengthMismatch {
            expected: video_len,
            actual: mp4_video.len(),
        });
    }

    // 6. Validate MP4 stream structure (ISOBMFF box)
    validate_mp4_stream(mp4_video)?;

    Ok(ExtractedMotionPhoto {
        primary_jpeg,
        mp4_video,
        metadata,
        xmp_xml: xmp_xml_str.to_string(),
        video_offset: expected_video_offset,
    })
}

/// Owned version of `extract_motion_photo`
pub fn extract_motion_photo_owned(
    assembled_data: &[u8],
) -> Result<ExtractedMotionPhotoOwned, MotionPhotoError> {
    extract_motion_photo(assembled_data).map(|e| e.to_owned())
}

/// Validates an assembled Motion Photo file and returns its parsed metadata
pub fn validate_motion_photo(
    assembled_data: &[u8],
) -> Result<MotionPhotoMetadata, MotionPhotoError> {
    extract_motion_photo(assembled_data).map(|e| e.metadata)
}

/// Checks whether a byte buffer contains a valid Google Motion Photo
pub fn is_motion_photo(data: &[u8]) -> bool {
    validate_motion_photo(data).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a minimal valid dummy JPEG (SOI, DQT, SOF0, DHT, SOS, scan bytes, EOI)
    fn create_dummy_jpeg() -> Vec<u8> {
        let mut jpeg = Vec::new();
        // SOI
        jpeg.extend_from_slice(&[0xFF, JPEG_SOI]);
        // DQT marker (0xFF, 0xDB, length=67)
        jpeg.extend_from_slice(&[0xFF, 0xDB, 0x00, 0x43, 0x00]);
        jpeg.extend_from_slice(&[16u8; 64]);
        // SOF0 marker (0xFF, 0xC0, length=11)
        jpeg.extend_from_slice(&[
            0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x10, 0x00, 0x10, 0x01, 0x11,
        ]);
        // SOS marker (0xFF, 0xDA, length=8)
        jpeg.extend_from_slice(&[0xFF, JPEG_SOS, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00]);
        // Entropy scan data (0x00 0x12 0x34)
        jpeg.extend_from_slice(&[0x00, 0x12, 0x34, 0x56]);
        // EOI
        jpeg.extend_from_slice(&[0xFF, JPEG_EOI]);
        jpeg
    }

    /// Helper to create a minimal valid dummy MP4 with ftyp and moov boxes
    fn create_dummy_mp4(total_len: usize) -> Vec<u8> {
        assert!(
            total_len >= 24,
            "Dummy MP4 length must be at least 24 bytes"
        );
        let mut mp4 = Vec::with_capacity(total_len);
        // ftyp box (16 bytes)
        mp4.extend_from_slice(&16u32.to_be_bytes()); // box size = 16
        mp4.extend_from_slice(b"ftyp"); // box type
        mp4.extend_from_slice(b"mp42"); // major brand
        mp4.extend_from_slice(&0u32.to_be_bytes()); // minor version

        // remainder filled with mdat box
        let mdat_len = total_len - 16;
        mp4.extend_from_slice(&(mdat_len as u32).to_be_bytes());
        mp4.extend_from_slice(b"mdat");
        mp4.resize(total_len, 0xAA);
        mp4
    }

    #[test]
    fn test_xmp_xml_generation() {
        let xmp = generate_motion_photo_xmp(1_500_000, 4_194_304, 1, true);

        // Verify XML namespaces
        assert!(xmp.contains(XMP_NAMESPACE_CAMERA));
        assert!(xmp.contains(XMP_NAMESPACE_CONTAINER));
        assert!(xmp.contains(XMP_NAMESPACE_ITEM));

        // Verify Attributes
        assert!(xmp.contains("GCamera:MotionPhoto=\"1\""));
        assert!(xmp.contains("GCamera:MotionPhotoVersion=\"1\""));
        assert!(xmp.contains("GCamera:MotionPhotoPresentationTimestampUs=\"1500000\""));

        // Verify V1 compatibility
        assert!(xmp.contains("GCamera:MicroVideoOffset=\"4194304\""));

        // Verify V2 Container items
        assert!(xmp.contains("Item:Mime=\"image/jpeg\""));
        assert!(xmp.contains("Item:Semantic=\"Primary\""));
        assert!(xmp.contains("Item:Length=\"0\""));
        assert!(xmp.contains("Item:Mime=\"video/mp4\""));
        assert!(xmp.contains("Item:Semantic=\"MotionPhoto\""));
        assert!(xmp.contains("Item:Length=\"4194304\""));
    }

    #[test]
    fn test_app1_segment_encoding() {
        let xml = "<x:xmpmeta>test</x:xmpmeta>";
        let seg = create_xmp_app1_segment(xml).unwrap();

        assert_eq!(seg[0], 0xFF);
        assert_eq!(seg[1], JPEG_APP1);
        let seg_len = u16::from_be_bytes([seg[2], seg[3]]) as usize;
        assert_eq!(seg_len, 2 + XMP_HEADER_SIGNATURE.len() + xml.len());
        assert_eq!(
            &seg[4..4 + XMP_HEADER_SIGNATURE.len()],
            XMP_HEADER_SIGNATURE
        );
        assert_eq!(&seg[4 + XMP_HEADER_SIGNATURE.len()..], xml.as_bytes());
    }

    #[test]
    fn test_assemble_and_extract_roundtrip() {
        let dummy_jpeg = create_dummy_jpeg();
        let dummy_mp4 = create_dummy_mp4(256);
        let timestamp_us = 2_000_000;

        let builder = MotionPhotoBuilder::new(timestamp_us);
        let assembled = builder.assemble(&dummy_jpeg, &dummy_mp4).unwrap();

        // Ensure assembled file is larger than the sum (due to added XMP APP1)
        assert!(assembled.len() > dummy_jpeg.len() + dummy_mp4.len());

        // Extract and validate
        let extracted = extract_motion_photo(&assembled).unwrap();
        assert!(extracted.metadata.is_motion_photo);
        assert_eq!(extracted.metadata.version, 1);
        assert_eq!(extracted.metadata.presentation_timestamp_us, timestamp_us);
        assert_eq!(extracted.metadata.video_length, dummy_mp4.len());

        // Verify exact container offset
        assert_eq!(extracted.video_offset, extracted.primary_jpeg.len());
        assert_eq!(extracted.video_offset + dummy_mp4.len(), assembled.len());

        // Verify exact byte streams
        assert_eq!(extracted.mp4_video, dummy_mp4.as_slice());
        assert!(extracted.primary_jpeg.starts_with(&[0xFF, JPEG_SOI]));
        assert!(extracted.primary_jpeg.ends_with(&[0xFF, JPEG_EOI]));

        // Verify container items
        let primary_item = extracted.metadata.primary_item().unwrap();
        assert_eq!(primary_item.mime, "image/jpeg");
        let video_item = extracted.metadata.video_item().unwrap();
        assert_eq!(video_item.mime, "video/mp4");
        assert_eq!(video_item.length, dummy_mp4.len());
    }

    #[test]
    fn test_existing_xmp_replacement() {
        let dummy_jpeg = create_dummy_jpeg();
        let dummy_mp4_first = create_dummy_mp4(128);

        // Assemble first time
        let first_assembled =
            assemble_motion_photo(&dummy_jpeg, &dummy_mp4_first, 1_000_000).unwrap();
        let first_extracted = extract_motion_photo(&first_assembled).unwrap();

        // Now take the first assembled JPEG (which already contains XMP APP1) and re-assemble with a different video
        let dummy_mp4_second = create_dummy_mp4(512);
        let second_assembled =
            assemble_motion_photo(first_extracted.primary_jpeg, &dummy_mp4_second, 3_500_000)
                .unwrap();

        let second_extracted = extract_motion_photo(&second_assembled).unwrap();
        assert_eq!(
            second_extracted.metadata.presentation_timestamp_us,
            3_500_000
        );
        assert_eq!(second_extracted.metadata.video_length, 512);
        assert_eq!(second_extracted.mp4_video, dummy_mp4_second.as_slice());

        // Ensure there is only one XMP APP1 segment, not two duplicates
        let (segments, _) = scan_jpeg_structure(second_extracted.primary_jpeg).unwrap();
        let xmp_count = segments.iter().filter(|s| s.is_xmp_app1).count();
        assert_eq!(
            xmp_count, 1,
            "Old XMP must be replaced rather than duplicated"
        );
    }

    #[test]
    fn test_atomic_file_assembly() {
        let dummy_jpeg = create_dummy_jpeg();
        let dummy_mp4 = create_dummy_mp4(100);

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("test_motion_photo_{}.jpg", std::process::id()));

        // Clean up before test
        let _ = fs::remove_file(&test_file);

        // Assemble atomically to disk
        assemble_motion_photo_to_file(&test_file, &dummy_jpeg, &dummy_mp4, 500_000).unwrap();
        assert!(test_file.exists());

        // Read back and validate
        let disk_data = fs::read(&test_file).unwrap();
        let metadata = validate_motion_photo(&disk_data).unwrap();
        assert!(metadata.is_motion_photo);
        assert_eq!(metadata.presentation_timestamp_us, 500_000);
        assert_eq!(metadata.video_length, 100);

        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_invalid_jpeg_rejection() {
        let not_a_jpeg = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature
        let dummy_mp4 = create_dummy_mp4(64);

        let res = assemble_motion_photo(&not_a_jpeg, &dummy_mp4, 1_000_000);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), MotionPhotoError::InvalidJpeg(_)));
    }

    #[test]
    fn test_invalid_mp4_rejection() {
        let dummy_jpeg = create_dummy_jpeg();
        let corrupted_mp4 = vec![0x00, 0x00, 0x00, 0x10, 0x58, 0x59, 0x5A, 0x57]; // Unknown box

        let res = assemble_motion_photo(&dummy_jpeg, &corrupted_mp4, 1_000_000);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), MotionPhotoError::InvalidMp4(_)));
    }

    #[test]
    fn test_container_offset_mismatch_detection() {
        let dummy_jpeg = create_dummy_jpeg();
        let dummy_mp4 = create_dummy_mp4(128);

        let mut assembled = assemble_motion_photo(&dummy_jpeg, &dummy_mp4, 1_000_000).unwrap();

        // Tamper with file by appending 10 garbage bytes at the end
        assembled.extend_from_slice(&[0xFF; 10]);

        let res = extract_motion_photo(&assembled);
        assert!(res.is_err());
        // Since XMP specifies 128 bytes, extracting at total - 128 will not match JPEG EOI
        assert!(matches!(
            res.unwrap_err(),
            MotionPhotoError::OffsetMismatch { .. }
        ));
    }

    #[test]
    fn test_backward_compat_method() {
        let builder = MotionPhotoBuilder::new(1_500_000);
        let dummy_jpeg = create_dummy_jpeg();
        let dummy_mp4 = create_dummy_mp4(64);

        let result = builder
            .assemble_motion_photo(&dummy_jpeg, &dummy_mp4)
            .unwrap();
        let extracted_video = builder
            .extract_embedded_video(&result, dummy_mp4.len())
            .unwrap();
        assert_eq!(extracted_video, &dummy_mp4[..]);
    }

    #[test]
    fn test_real_lmc83_jpeg_assembly_and_extraction() {
        let sample_path = Path::new("../lmc8.3_analysis/PXL_20261013_154150.jpg");
        if sample_path.exists() {
            let real_jpeg = fs::read(sample_path).expect("Failed to read sample JPEG");
            let dummy_mp4 = create_dummy_mp4(1024);
            let timestamp_us = 1_541_500_000;

            let assembled = assemble_motion_photo(&real_jpeg, &dummy_mp4, timestamp_us)
                .expect("Failed to assemble Motion Photo with real JPEG");

            assert!(assembled.len() > real_jpeg.len() + dummy_mp4.len());

            let extracted = extract_motion_photo(&assembled)
                .expect("Failed to extract Motion Photo with real JPEG");

            assert!(extracted.metadata.is_motion_photo);
            assert_eq!(extracted.metadata.presentation_timestamp_us, timestamp_us);
            assert_eq!(extracted.metadata.video_length, dummy_mp4.len());
            assert_eq!(extracted.mp4_video, dummy_mp4.as_slice());
            assert!(extracted.primary_jpeg.starts_with(&[0xFF, JPEG_SOI]));
            assert!(extracted.primary_jpeg.ends_with(&[0xFF, JPEG_EOI]));
        }
    }
}
