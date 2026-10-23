//! Multi-frame exposure fusion for the HDR capture mode.
//!
//! The first implementation is deliberately CPU-side and deterministic:
//! Mertens-style weights (contrast, saturation, well-exposedness) are computed
//! per pixel and blended. It is good enough to prove the bracketing pipeline and
//! is unit-testable without a device. A later revision will add motion
//! alignment and, where available, HAL SHDR/QHDR.

use std::io::Cursor;

use jpeg_encoder::{ColorType, Encoder};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HdrError {
    #[error("invalid image dimensions: {0}x{1}")]
    InvalidDimensions(usize, usize),
    #[error("frame {frame} has {got} bytes, expected {expected}")]
    InvalidFrameSize {
        frame: usize,
        got: usize,
        expected: usize,
    },
    #[error("no frames supplied")]
    NoFrames,
    #[error("JPEG encoding failed: {0}")]
    Jpeg(String),
}

/// Tightly packed RGB8 image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbImage {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl RgbImage {
    pub fn new(width: usize, height: usize, data: Vec<u8>) -> Result<Self, HdrError> {
        if width == 0 || height == 0 {
            return Err(HdrError::InvalidDimensions(width, height));
        }
        let expected = width * height * 3;
        if data.len() != expected {
            return Err(HdrError::InvalidFrameSize {
                frame: 0,
                got: data.len(),
                expected,
            });
        }
        Ok(Self {
            width,
            height,
            data,
        })
    }

    fn luma(&self) -> Vec<u8> {
        let mut out = vec![0u8; self.width * self.height];
        for (i, pixel) in out.iter_mut().enumerate() {
            let r = self.data[i * 3] as f32;
            let g = self.data[i * 3 + 1] as f32;
            let b = self.data[i * 3 + 2] as f32;
            *pixel = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 255.0) as u8;
        }
        out
    }
}

/// One YUV_420_888 frame from `AImageReader` (separate U/V planes).
#[derive(Clone, Debug)]
pub struct YuvFrame {
    pub width: usize,
    pub height: usize,
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
    pub y_stride: usize,
    pub u_stride: usize,
    pub v_stride: usize,
    pub u_pixel_stride: usize,
    pub v_pixel_stride: usize,
}

impl YuvFrame {
    fn sample(&self, plane: &[u8], stride: usize, pixel_stride: usize, x: usize, y: usize) -> u8 {
        let idx = y
            .saturating_mul(stride)
            .saturating_add(x.saturating_mul(pixel_stride.max(1)));
        plane.get(idx).copied().unwrap_or(128)
    }
}

fn worker_count(height: usize) -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8)
        .min(height.max(1))
}

/// Converts a YUV_420_888 frame to RGB8 using BT.601 limited-range coefficients.
pub fn yuv420_to_rgb(frame: &YuvFrame) -> Result<RgbImage, HdrError> {
    if frame.width == 0 || frame.height == 0 {
        return Err(HdrError::InvalidDimensions(frame.width, frame.height));
    }
    let width = frame.width;
    let height = frame.height;
    let row_bytes = width * 3;
    let threads = worker_count(height);
    let rows_per_thread = height.div_ceil(threads);
    let mut rgb = vec![0u8; width * height * 3];

    std::thread::scope(|scope| {
        for (chunk_index, chunk) in rgb.chunks_mut(rows_per_thread * row_bytes).enumerate() {
            scope.spawn(move || {
                let start_y = chunk_index * rows_per_thread;
                for (local_y, row) in chunk.chunks_mut(row_bytes).enumerate() {
                    let y = start_y + local_y;
                    for x in 0..width {
                        let y_val = frame.sample(&frame.y, frame.y_stride, 1, x, y) as f32;
                        let u_val = frame.sample(
                            &frame.u,
                            frame.u_stride,
                            frame.u_pixel_stride,
                            x / 2,
                            y / 2,
                        ) as f32;
                        let v_val = frame.sample(
                            &frame.v,
                            frame.v_stride,
                            frame.v_pixel_stride,
                            x / 2,
                            y / 2,
                        ) as f32;

                        let c = y_val - 16.0;
                        let d = u_val - 128.0;
                        let e = v_val - 128.0;
                        let r = ((298.0 * c + 409.0 * e + 128.0) / 256.0).clamp(0.0, 255.0);
                        let g =
                            ((298.0 * c - 100.0 * d - 208.0 * e + 128.0) / 256.0).clamp(0.0, 255.0);
                        let b = ((298.0 * c + 516.0 * d + 128.0) / 256.0).clamp(0.0, 255.0);

                        let idx = x * 3;
                        row[idx] = r as u8;
                        row[idx + 1] = g as u8;
                        row[idx + 2] = b as u8;
                    }
                }
            });
        }
    });

    RgbImage::new(width, height, rgb)
}

fn downsample_luma(image: &RgbImage, factor: usize) -> (Vec<u8>, usize, usize) {
    let factor = factor.max(1);
    let luma = image.luma();
    let width = image.width / factor;
    let height = image.height / factor;
    let mut out = vec![0u8; width * height];
    for by in 0..height {
        for bx in 0..width {
            let mut sum = 0u32;
            for y in 0..factor {
                for x in 0..factor {
                    let src = (by * factor + y) * image.width + bx * factor + x;
                    sum += luma[src] as u32;
                }
            }
            out[by * width + bx] = (sum / (factor * factor) as u32) as u8;
        }
    }
    (out, width, height)
}

/// Normalised cross-correlation for a candidate shift applied to `target`.
///
/// NCC is invariant to affine brightness changes, which is essential when the
/// bracket frames have different exposures. Returns `None` for flat regions.
fn ncc_at_shift(
    reference: &[u8],
    width: usize,
    height: usize,
    target: &[u8],
    dx: i32,
    dy: i32,
) -> Option<f64> {
    let mut count = 0u64;
    let mut sum_r = 0.0f64;
    let mut sum_t = 0.0f64;
    let mut sum_rr = 0.0f64;
    let mut sum_tt = 0.0f64;
    let mut sum_rt = 0.0f64;

    for y in 0..height {
        for x in 0..width {
            let tx = x as i32 - dx;
            let ty = y as i32 - dy;
            if tx < 0 || ty < 0 || tx >= width as i32 || ty >= height as i32 {
                continue;
            }
            let r = reference[y * width + x] as f64;
            let t = target[ty as usize * width + tx as usize] as f64;
            sum_r += r;
            sum_t += t;
            sum_rr += r * r;
            sum_tt += t * t;
            sum_rt += r * t;
            count += 1;
        }
    }

    if count < 16 {
        return None;
    }
    let n = count as f64;
    let mean_r = sum_r / n;
    let mean_t = sum_t / n;
    let var_r = (sum_rr / n - mean_r * mean_r).max(0.0);
    let var_t = (sum_tt / n - mean_t * mean_t).max(0.0);
    if var_r < 1.0 || var_t < 1.0 {
        return None;
    }
    let covariance = sum_rt / n - mean_r * mean_t;
    Some(covariance / (var_r.sqrt() * var_t.sqrt()))
}

/// Estimates the integer shift to apply to `target` so it aligns with `reference`.
///
/// A coarse luma search at 1/8 (or 1/2 for small images) is refined at 1/4. The
/// result is the shift for [`shift_image`], i.e. `target(x - dx, y - dy)`.
pub fn estimate_translation(
    reference: &RgbImage,
    target: &RgbImage,
    max_shift: i32,
) -> Result<(i32, i32), HdrError> {
    if reference.width != target.width || reference.height != target.height {
        return Err(HdrError::InvalidFrameSize {
            frame: 1,
            got: target.data.len(),
            expected: reference.width * reference.height * 3,
        });
    }

    let min_dim = reference.width.min(reference.height);
    let coarse_factor = if min_dim >= 512 { 8 } else { 2 };
    let (ref_coarse, cw, ch) = downsample_luma(reference, coarse_factor);
    let (tgt_coarse, _, _) = downsample_luma(target, coarse_factor);
    let coarse_limit = (max_shift / coarse_factor as i32).max(2);

    let mut best = (0, 0);
    let mut best_score = f64::MIN;
    for dy in -coarse_limit..=coarse_limit {
        for dx in -coarse_limit..=coarse_limit {
            if let Some(ncc) = ncc_at_shift(&ref_coarse, cw, ch, &tgt_coarse, dx, dy) {
                let penalty = 0.0005 * ((dx * dx + dy * dy) as f64);
                let score = ncc - penalty;
                if score > best_score {
                    best_score = score;
                    best = (dx, dy);
                }
            }
        }
    }

    let coarse_full = (best.0 * coarse_factor as i32, best.1 * coarse_factor as i32);
    let refine_factor = if min_dim >= 512 { 4 } else { 1 };
    let (ref_fine, fw, fh) = downsample_luma(reference, refine_factor);
    let (tgt_fine, _, _) = downsample_luma(target, refine_factor);
    let center = (
        coarse_full.0 / refine_factor as i32,
        coarse_full.1 / refine_factor as i32,
    );
    let refine_limit = 2;

    let mut best_fine = center;
    let mut best_fine_score = f64::MIN;
    for dy in center.1 - refine_limit..=center.1 + refine_limit {
        for dx in center.0 - refine_limit..=center.0 + refine_limit {
            if let Some(ncc) = ncc_at_shift(&ref_fine, fw, fh, &tgt_fine, dx, dy) {
                let penalty = 0.0005 * ((dx * dx + dy * dy) as f64);
                let score = ncc - penalty;
                if score > best_fine_score {
                    best_fine_score = score;
                    best_fine = (dx, dy);
                }
            }
        }
    }

    Ok((
        best_fine.0 * refine_factor as i32,
        best_fine.1 * refine_factor as i32,
    ))
}

/// Shifts an RGB image by `(dx, dy)`; out-of-bounds samples clamp to the edge.
pub fn shift_image(image: &RgbImage, dx: i32, dy: i32) -> RgbImage {
    let mut out = vec![0u8; image.width * image.height * 3];
    for y in 0..image.height {
        for x in 0..image.width {
            let sx = (x as i32 - dx).clamp(0, image.width as i32 - 1) as usize;
            let sy = (y as i32 - dy).clamp(0, image.height as i32 - 1) as usize;
            let src = (sy * image.width + sx) * 3;
            let dst = (y * image.width + x) * 3;
            out[dst] = image.data[src];
            out[dst + 1] = image.data[src + 1];
            out[dst + 2] = image.data[src + 2];
        }
    }
    RgbImage {
        width: image.width,
        height: image.height,
        data: out,
    }
}

/// Aligns all frames to the middle frame, then fuses them.
pub fn exposure_fusion_aligned(frames: &[RgbImage]) -> Result<RgbImage, HdrError> {
    if frames.len() < 2 {
        return exposure_fusion(frames);
    }
    let reference_index = frames.len() / 2;
    let reference = &frames[reference_index];
    let mut aligned = Vec::with_capacity(frames.len());
    for (index, frame) in frames.iter().enumerate() {
        if index == reference_index {
            aligned.push(frame.clone());
            continue;
        }
        let (dx, dy) = estimate_translation(reference, frame, 64)?;
        aligned.push(shift_image(frame, dx, dy));
    }
    exposure_fusion(&aligned)
}

/// Mertens-style exposure fusion.
///
/// Weight = (contrast + eps) * (saturation + eps) * (well_exposedness + eps).
/// Use [`exposure_fusion_aligned`] when the frames may be shifted.
pub fn exposure_fusion(frames: &[RgbImage]) -> Result<RgbImage, HdrError> {
    if frames.is_empty() {
        return Err(HdrError::NoFrames);
    }
    let width = frames[0].width;
    let height = frames[0].height;
    for (i, frame) in frames.iter().enumerate() {
        if frame.width != width || frame.height != height {
            return Err(HdrError::InvalidFrameSize {
                frame: i,
                got: frame.data.len(),
                expected: width * height * 3,
            });
        }
    }

    let lumas: Vec<Vec<u8>> = frames.iter().map(RgbImage::luma).collect();
    let row_bytes = width * 3;
    let threads = worker_count(height);
    let rows_per_thread = height.div_ceil(threads);
    let mut out = vec![0u8; width * height * 3];

    std::thread::scope(|scope| {
        for (chunk_index, chunk) in out.chunks_mut(rows_per_thread * row_bytes).enumerate() {
            let frames = &frames;
            let lumas = &lumas;
            scope.spawn(move || {
                let start_y = chunk_index * rows_per_thread;
                for (local_y, row) in chunk.chunks_mut(row_bytes).enumerate() {
                    let y = start_y + local_y;
                    for x in 0..width {
                        let idx = y * width + x;
                        let mut weight_sum = 0.0f32;
                        let mut acc = [0.0f32; 3];

                        for (frame_idx, frame) in frames.iter().enumerate() {
                            let r = frame.data[idx * 3] as f32 / 255.0;
                            let g = frame.data[idx * 3 + 1] as f32 / 255.0;
                            let b = frame.data[idx * 3 + 2] as f32 / 255.0;
                            let l = lumas[frame_idx][idx] as f32 / 255.0;

                            // Local Laplacian magnitude on the luma plane.
                            let left = if x > 0 {
                                lumas[frame_idx][idx - 1] as f32 / 255.0
                            } else {
                                l
                            };
                            let right = if x + 1 < width {
                                lumas[frame_idx][idx + 1] as f32 / 255.0
                            } else {
                                l
                            };
                            let up = if y > 0 {
                                lumas[frame_idx][idx - width] as f32 / 255.0
                            } else {
                                l
                            };
                            let down = if y + 1 < height {
                                lumas[frame_idx][idx + width] as f32 / 255.0
                            } else {
                                l
                            };
                            let contrast = (4.0 * l - left - right - up - down).abs().min(1.0);

                            let max_c = r.max(g).max(b);
                            let min_c = r.min(g).min(b);
                            let saturation = (max_c - min_c).min(1.0);

                            // Gaussian around mid-grey; sigma = 0.2.
                            let exposure = (-((l - 0.5).powi(2)) / (2.0 * 0.2 * 0.2)).exp();

                            let weight =
                                (contrast + 0.01) * (saturation + 0.01) * (exposure + 0.01);
                            weight_sum += weight;
                            acc[0] += r * weight;
                            acc[1] += g * weight;
                            acc[2] += b * weight;
                        }

                        if weight_sum <= f32::EPSILON {
                            row[x * 3] = frames[0].data[idx * 3];
                            row[x * 3 + 1] = frames[0].data[idx * 3 + 1];
                            row[x * 3 + 2] = frames[0].data[idx * 3 + 2];
                        } else {
                            for c in 0..3 {
                                row[x * 3 + c] =
                                    ((acc[c] / weight_sum) * 255.0).clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                }
            });
        }
    });

    RgbImage::new(width, height, out)
}

/// Encodes RGB8 to JPEG.
pub fn encode_jpeg(image: &RgbImage, quality: u8) -> Result<Vec<u8>, HdrError> {
    let mut out = Vec::new();
    {
        let mut cursor = Cursor::new(&mut out);
        let encoder = Encoder::new(&mut cursor, quality);
        encoder
            .encode(
                &image.data,
                image.width as u16,
                image.height as u16,
                ColorType::Rgb,
            )
            .map_err(|e| HdrError::Jpeg(e.to_string()))?;
    }
    Ok(out)
}

/// Rotates an RGB image 90 degrees clockwise (sensor landscape -> portrait UI).
pub fn rotate90_cw(image: &RgbImage) -> RgbImage {
    let mut out = vec![0u8; image.width * image.height * 3];
    let new_width = image.height;
    let new_height = image.width;
    for y in 0..image.height {
        for x in 0..image.width {
            let src = (y * image.width + x) * 3;
            let dst_x = new_width - 1 - y;
            let dst_y = x;
            let dst = (dst_y * new_width + dst_x) * 3;
            out[dst] = image.data[src];
            out[dst + 1] = image.data[src + 1];
            out[dst + 2] = image.data[src + 2];
        }
    }
    RgbImage {
        width: new_width,
        height: new_height,
        data: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yuv(width: usize, height: usize, y_value: u8) -> YuvFrame {
        YuvFrame {
            width,
            height,
            y: vec![y_value; width * height],
            u: vec![128; width * height / 2],
            v: vec![128; width * height / 2],
            y_stride: width,
            u_stride: width / 2,
            v_stride: width / 2,
            u_pixel_stride: 1,
            v_pixel_stride: 1,
        }
    }

    #[test]
    fn yuv_gray_converts_to_gray() {
        let rgb = yuv420_to_rgb(&yuv(4, 4, 128)).unwrap();
        assert_eq!(rgb.data.len(), 4 * 4 * 3);
        assert!((rgb.data[0] as i32 - 130).abs() <= 3);
    }

    #[test]
    fn fusion_of_mid_frame_stays_mid() {
        let dark = RgbImage::new(2, 2, vec![20; 12]).unwrap();
        let mid = RgbImage::new(2, 2, vec![128; 12]).unwrap();
        let bright = RgbImage::new(2, 2, vec![240; 12]).unwrap();
        let fused = exposure_fusion(&[dark, mid, bright]).unwrap();
        assert!(fused.data.iter().all(|v| (100..=160).contains(v)));
    }

    #[test]
    fn fusion_requires_matching_dimensions() {
        let a = RgbImage::new(2, 2, vec![0; 12]).unwrap();
        let b = RgbImage::new(1, 1, vec![0; 3]).unwrap();
        assert!(exposure_fusion(&[a, b]).is_err());
    }

    #[test]
    fn jpeg_encoding_has_soi_and_eoi() {
        let image = RgbImage::new(2, 2, vec![128; 12]).unwrap();
        let jpeg = encode_jpeg(&image, 90).unwrap();
        assert_eq!(&jpeg[0..2], &[0xFF, 0xD8]);
        assert_eq!(&jpeg[jpeg.len() - 2..], &[0xFF, 0xD9]);
    }

    #[test]
    fn rotate_swaps_dimensions() {
        let image = RgbImage::new(2, 3, vec![0; 18]).unwrap();
        let rotated = rotate90_cw(&image);
        assert_eq!((rotated.width, rotated.height), (3, 2));
    }

    fn pattern(width: usize, height: usize) -> RgbImage {
        let mut data = vec![0u8; width * height * 3];
        let mut state = 0x1234_5678u32;
        for y in 0..height {
            for x in 0..width {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let v = (state >> 24) as u8;
                let i = (y * width + x) * 3;
                data[i] = v;
                data[i + 1] = v.wrapping_add(40);
                data[i + 2] = v.wrapping_add(80);
            }
        }
        RgbImage::new(width, height, data).unwrap()
    }

    #[test]
    fn translation_estimation_recovers_shift() {
        let reference = pattern(64, 48);
        let shifted = shift_image(&reference, 3, 2);
        let (dx, dy) = estimate_translation(&reference, &shifted, 32).unwrap();
        assert!((dx + 3).abs() <= 1, "dx={dx}");
        assert!((dy + 2).abs() <= 1, "dy={dy}");
    }

    #[test]
    fn translation_is_exposure_invariant() {
        let reference = pattern(96, 64);
        let dark = RgbImage::new(
            reference.width,
            reference.height,
            reference.data.iter().map(|v| v / 2).collect(),
        )
        .unwrap();
        let bright = RgbImage::new(
            reference.width,
            reference.height,
            reference
                .data
                .iter()
                .map(|v| ((*v as u16) * 3 / 2).min(255) as u8)
                .collect(),
        )
        .unwrap();
        let shifted_bright = shift_image(&bright, -4, 3);
        let (dx, dy) = estimate_translation(&dark, &shifted_bright, 32).unwrap();
        assert!((dx - 4).abs() <= 1, "dx={dx}");
        assert!((dy + 3).abs() <= 1, "dy={dy}");
    }

    #[test]
    fn aligned_fusion_accepts_shifted_frames() {
        let reference = pattern(64, 48);
        let dark = RgbImage::new(
            reference.width,
            reference.height,
            reference.data.iter().map(|v| v / 3).collect(),
        )
        .unwrap();
        let bright = RgbImage::new(
            reference.width,
            reference.height,
            reference
                .data
                .iter()
                .map(|v| (*v as u16 * 2).min(255) as u8)
                .collect(),
        )
        .unwrap();
        let shifted_bright = shift_image(&bright, 2, 1);
        let fused = exposure_fusion_aligned(&[dark, reference, shifted_bright]).unwrap();
        assert_eq!(fused.data.len(), 64 * 48 * 3);
        assert!(fused.data.iter().any(|v| *v > 0));
    }
}
