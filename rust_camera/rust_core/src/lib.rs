//! OpOpenCam native camera core.
//!
//! This crate is the Rust engine behind the Compose shell. It owns the Camera2
//! NDK session, the still-capture reader and the reusable media helpers.
//!
//! The Android-only modules are gated so the pure media/math modules can be
//! tested on a desktop host; the full engine is built for
//! `aarch64-linux-android`.

#[cfg(target_os = "android")]
pub mod bridge;
#[cfg(target_os = "android")]
pub mod camera_engine;

pub mod capture_modes;
pub mod circular_ring_buffer;
pub mod dng_writer;
pub mod hdr;
pub mod motion_photo_xmp;
pub mod raw_render;
pub mod sensors;
