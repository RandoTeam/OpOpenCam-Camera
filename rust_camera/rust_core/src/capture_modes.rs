//! Capture modes exposed by the Rust engine and the Compose UI.
//!
//! Each mode selects a different still output. Mode changes restart the Camera2
//! session (preview surface is reused), which keeps the HAL stream configuration
//! valid and avoids exceeding the device's simultaneous-output limit.

/// Still capture strategy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    /// 4096x3072 JPEG (12.5 MP binned).
    Jpeg12 = 0,
    /// 8192x6144 JPEG (50 MP remosaic; may require the vendor 50 MP mode).
    Jpeg50 = 1,
    /// 4096x3072 RAW10/RAW16, written as DNG.
    Raw12 = 2,
    /// 8192x6144 RAW, written as DNG.
    Raw50 = 3,
    /// Three YUV frames at -2/0/+2 EV, merged and encoded as JPEG.
    Hdr3 = 4,
    /// Hidden probe: 5888x4416 RAW_SENSOR via the vendor resolution session.
    Raw25 = 5,
}

impl CaptureMode {
    pub const ALL: [CaptureMode; 6] = [
        CaptureMode::Jpeg12,
        CaptureMode::Jpeg50,
        CaptureMode::Raw12,
        CaptureMode::Raw50,
        CaptureMode::Hdr3,
        CaptureMode::Raw25,
    ];

    /// Maps the integer passed from Kotlin. Unknown values fall back to 12 MP JPEG.
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Jpeg50,
            2 => Self::Raw12,
            3 => Self::Raw50,
            4 => Self::Hdr3,
            5 => Self::Raw25,
            _ => Self::Jpeg12,
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Jpeg12 => "12 MP",
            Self::Jpeg50 => "50 MP",
            Self::Raw12 => "RAW 12",
            Self::Raw50 => "RAW 50",
            Self::Hdr3 => "HDR 3×",
            Self::Raw25 => "RAW 25 probe",
        }
    }

    /// Output size of the still reader.
    pub fn still_size(self) -> (i32, i32) {
        match self {
            Self::Raw25 => (5888, 4416),
            Self::Jpeg50 | Self::Raw50 => (8192, 6144),
            _ => (4096, 3072),
        }
    }

    /// `ImageFormat` value for the `AImageReader`.
    ///
    /// 256 = JPEG, 32 = RAW_SENSOR (16-bit), 35 = YUV_420_888.
    pub fn image_format(self) -> i32 {
        match self {
            Self::Jpeg12 | Self::Jpeg50 => 256,
            Self::Raw12 | Self::Raw25 | Self::Raw50 => 32,
            Self::Hdr3 => 35,
        }
    }

    pub fn is_jpeg(self) -> bool {
        matches!(self, Self::Jpeg12 | Self::Jpeg50)
    }

    pub fn is_raw(self) -> bool {
        matches!(self, Self::Raw12 | Self::Raw25 | Self::Raw50)
    }

    pub fn is_hdr(self) -> bool {
        matches!(self, Self::Hdr3)
    }

    pub fn is_50mp(self) -> bool {
        matches!(self, Self::Jpeg50 | Self::Raw50)
    }

    pub fn is_25mp(self) -> bool {
        matches!(self, Self::Raw25)
    }

    /// File extension for the Compose layer.
    pub fn file_extension(self) -> &'static str {
        if self.is_raw() {
            "dng"
        } else {
            "jpg"
        }
    }

    /// Number of still requests.
    pub fn exposure_count(self) -> usize {
        if self.is_hdr() {
            3
        } else {
            1
        }
    }

    /// AE compensation steps (1/3 EV units) for the HDR sequence.
    ///
    /// `-6, 0, +6` is -2 EV / 0 / +2 EV. Keeping the range conservative avoids
    /// hitting the HAL's AE compensation limits on scenes with little headroom.
    pub fn bracket_ev_steps(self) -> &'static [i32] {
        if self.is_hdr() {
            &[-6, 0, 6]
        } else {
            &[0]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_ids_roundtrip() {
        for mode in CaptureMode::ALL {
            assert_eq!(CaptureMode::from_i32(mode.as_i32()), mode);
        }
    }

    #[test]
    fn unknown_mode_falls_back_to_12mp_jpeg() {
        assert_eq!(CaptureMode::from_i32(99), CaptureMode::Jpeg12);
    }

    #[test]
    fn sizes_and_formats_match_the_hal_baseline() {
        assert_eq!(CaptureMode::Jpeg12.still_size(), (4096, 3072));
        assert_eq!(CaptureMode::Jpeg50.still_size(), (8192, 6144));
        assert_eq!(CaptureMode::Raw25.still_size(), (5888, 4416));
        assert_eq!(CaptureMode::Raw25.image_format(), 32);
        assert_eq!(CaptureMode::Raw12.image_format(), 32);
        assert_eq!(CaptureMode::Hdr3.image_format(), 35);
        assert_eq!(CaptureMode::Jpeg50.image_format(), 256);
    }

    #[test]
    fn hdr_plan_is_symmetric() {
        let plan = CaptureMode::Hdr3.bracket_ev_steps();
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0], -plan[2]);
        assert_eq!(plan[1], 0);
    }

    #[test]
    fn extensions_are_user_facing() {
        assert_eq!(CaptureMode::Jpeg12.file_extension(), "jpg");
        assert_eq!(CaptureMode::Raw12.file_extension(), "dng");
        assert_eq!(CaptureMode::Hdr3.file_extension(), "jpg");
    }
}
