//! OnePlus 13 camera topology and Qualcomm/OPlus vendor tag constants.
//!
//! The values here are the verified hardware IDs from the device's Camera2
//! static metadata (`dumpsys media.camera`, 2026-10-22):
//!
//! | ID | Role | Sensor | Active array |
//! |----|------|--------|--------------|
//! | 0  | logical SAT (multi-camera) | routes to 3 / 2 / 4 | 4096x3072 |
//! | 1  | front | Sony IMX615 | 3280x2464 |
//! | 2  | physical main | Sony LYT-808 | 4096x3072 |
//! | 3  | physical ultra-wide | Samsung JN5 | 4096x3072 |
//! | 4  | physical 3x tele | Sony LYT-600 | 4096x3072 |
//!
//! All three rear sensors advertise 8192x6144 RAW sizes and 4096x3072 output at
//! 30 fps; 3840x2160 is available at 60 fps. The logical camera exposes a
//! 0.6x-20x zoom range and physical IDs `[3, 2, 4]`.

/// Qualcomm vendor tag: enable the IdealRAW pipeline (bypasses some ISP tone mapping).
pub const VENDOR_TAG_ENABLE_IDEAL_RAW: u32 = 0x8026_002d;

/// OPlus vendor tag: calling package name used by the Chi HAL whitelist.
pub const VENDOR_TAG_OPLUS_PACKAGE_NAME: u32 = 0x8117_002e;

/// OPlus vendor tag: 50 MP ultra-resolution mode (0 = binned, 1 = remosaic).
pub const VENDOR_TAG_ULTRA_RESOLUTION_MODE: u32 = 0x8117_0048;

/// OPlus vendor tag: edge-enhancement level (0 = no over-sharpening).
pub const VENDOR_TAG_CONTROL_EDGE_LEVEL: u32 = 0x8117_0094;

/// OPlus vendor tag: clarity level (0 = no clarity boost).
pub const VENDOR_TAG_CONTROL_CLARITY_LEVEL: u32 = 0x8117_0096;

/// Package name expected by the OnePlus Chi HAL privileged path.
pub const PRIVILEGED_PACKAGE_NAME: &str = "com.oneplus.camera";

/// Hardware camera sensors on the OnePlus 13 (PJZ110).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OnePlus13Sensor {
    /// Logical SAT camera, usually the default Camera2 ID 0.
    LogicalSat = 0,
    /// Front Sony IMX615.
    Front = 1,
    /// Physical main Sony LYT-808.
    PhysicalMain = 2,
    /// Physical ultra-wide Samsung JN5.
    PhysicalUltraWide = 3,
    /// Physical 3x periscope tele Sony LYT-600.
    PhysicalTelephoto3x = 4,
}

impl OnePlus13Sensor {
    /// Maps an integer camera ID to the sensor enum, falling back to the main sensor.
    pub fn from_id(id: i32) -> Self {
        match id {
            0 => Self::LogicalSat,
            1 => Self::Front,
            2 => Self::PhysicalMain,
            3 => Self::PhysicalUltraWide,
            4 => Self::PhysicalTelephoto3x,
            _ => Self::PhysicalMain,
        }
    }

    /// Integer Camera2 ID.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// Camera2 ID as the string expected by the NDK API.
    pub fn camera_id_str(&self) -> &'static str {
        match self {
            Self::LogicalSat => "0",
            Self::Front => "1",
            Self::PhysicalMain => "2",
            Self::PhysicalUltraWide => "3",
            Self::PhysicalTelephoto3x => "4",
        }
    }

    /// Human-readable sensor name used in logs and UI.
    pub fn sensor_name(&self) -> &'static str {
        match self {
            Self::LogicalSat => "Qualcomm SAT Multi-Camera",
            Self::Front => "Sony IMX615 (32MP)",
            Self::PhysicalMain => "Sony LYT-808 (50MP)",
            Self::PhysicalUltraWide => "Samsung JN5 (50MP)",
            Self::PhysicalTelephoto3x => "Sony LYT-600 (50MP)",
        }
    }

    /// Full-resolution dimensions advertised by the sensor.
    pub fn full_resolution_dimensions(&self) -> (u32, u32) {
        match self {
            Self::Front => (6560, 4928),
            _ => (8192, 6144),
        }
    }

    /// Standard 4:1 pixel-binned output dimensions.
    pub fn binned_resolution_dimensions(&self) -> (u32, u32) {
        match self {
            Self::Front => (3280, 2464),
            _ => (4096, 3072),
        }
    }

    /// 35 mm equivalent focal length.
    pub fn focal_length_35mm_equiv(&self) -> f32 {
        match self {
            Self::LogicalSat | Self::PhysicalMain => 24.0,
            Self::Front => 21.0,
            Self::PhysicalUltraWide => 15.0,
            Self::PhysicalTelephoto3x => 73.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_ids_match_hardware_topology() {
        assert_eq!(OnePlus13Sensor::LogicalSat.camera_id_str(), "0");
        assert_eq!(OnePlus13Sensor::PhysicalMain.camera_id_str(), "2");
        assert_eq!(OnePlus13Sensor::PhysicalUltraWide.camera_id_str(), "3");
        assert_eq!(OnePlus13Sensor::PhysicalTelephoto3x.camera_id_str(), "4");
    }

    #[test]
    fn unknown_ids_fall_back_to_main() {
        assert_eq!(OnePlus13Sensor::from_id(999), OnePlus13Sensor::PhysicalMain);
    }

    #[test]
    fn rear_sensors_advertise_50mp() {
        assert_eq!(
            OnePlus13Sensor::PhysicalMain.full_resolution_dimensions(),
            (8192, 6144)
        );
        assert_eq!(
            OnePlus13Sensor::PhysicalTelephoto3x.full_resolution_dimensions(),
            (8192, 6144)
        );
    }
}
