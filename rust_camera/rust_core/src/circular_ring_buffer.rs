//! 6.0-Second Circular RAM Buffer for Extreme Live Photo (4K 60 FPS) on OnePlus 13
//!
//! Stores up to 360 frames (3.0s pre-shutter + 3.0s post-shutter) in LPDDR5X RAM with zero
//! dynamic allocations in the hot path.

use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

/// Metadata for a single frame captured from the camera sensor
#[derive(Clone, Debug)]
pub struct FrameMetadata {
    pub sequence_id: u64,
    pub timestamp_ns: u64,
    pub width: u32,
    pub height: u32,
    pub stride_y: u32,
    pub stride_uv: u32,
    pub exposure_time_ns: i64,
    pub iso: i32,
}

/// A pre-allocated slot in the circular ring buffer holding YUV420 data
pub struct RingBufferSlot {
    pub metadata: FrameMetadata,
    /// Pre-allocated buffer: for 4K YUV420 (3840x2160), Y = 3840*2160, UV = 3840*2160/2. Total ~ 12.44 MB
    pub data: Vec<u8>,
    pub is_valid: bool,
}

impl RingBufferSlot {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            metadata: FrameMetadata {
                sequence_id: 0,
                timestamp_ns: 0,
                width: 0,
                height: 0,
                stride_y: 0,
                stride_uv: 0,
                exposure_time_ns: 0,
                iso: 0,
            },
            data: if capacity_bytes > 0 {
                vec![0u8; capacity_bytes]
            } else {
                Vec::new()
            },
            is_valid: false,
        }
    }
}

/// State of the Live Photo latch trigger
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LatchState {
    /// Actively recording pre-shutter frames into the ring buffer
    Rolling,
    /// Shutter pressed: recording post-shutter frames until count reaches target
    Triggered {
        trigger_timestamp_ns: u64,
        shutter_sequence_id: u64,
        remaining_post_frames: usize,
    },
    /// 360-frame capture complete; ready for background HEVC/XMP encoding
    Completed { trigger_timestamp_ns: u64 },
}

/// 6.0-Second Live Photo Ring Buffer
pub struct LivePhotoRingBuffer {
    pub capacity: usize,
    slots: Vec<Mutex<RingBufferSlot>>,
    write_head: AtomicUsize,
    total_frames_written: AtomicU64,
    latch_state: Mutex<LatchState>,
    is_active: AtomicBool,
}

impl LivePhotoRingBuffer {
    /// Creates a circular buffer for `capacity` frames (default 360 for 6 seconds @ 60 FPS).
    /// If `slot_bytes` is 0, slots allocate on-demand on first frame push (zero startup overhead).
    pub fn new(capacity: usize, slot_bytes: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            slots.push(Mutex::new(RingBufferSlot::new(slot_bytes)));
        }

        Self {
            capacity,
            slots,
            write_head: AtomicUsize::new(0),
            total_frames_written: AtomicU64::new(0),
            latch_state: Mutex::new(LatchState::Rolling),
            is_active: AtomicBool::new(true),
        }
    }

    /// Pushes a new frame into the circular buffer without truncating payload.
    pub fn push_frame(&self, metadata: FrameMetadata, source_data: &[u8]) -> Option<LatchState> {
        if !self.is_active.load(Ordering::Acquire) {
            return None;
        }

        let head = self.write_head.fetch_add(1, Ordering::Relaxed) % self.capacity;
        self.total_frames_written.fetch_add(1, Ordering::Relaxed);

        // Copy frame data into slot (resizing if needed to prevent truncation)
        {
            let mut slot = self.slots[head].lock();
            slot.metadata = metadata;
            if slot.data.len() != source_data.len() {
                slot.data.resize(source_data.len(), 0);
            }
            slot.data.copy_from_slice(source_data);
            slot.is_valid = true;
        }

        // Check and update latch state
        let mut state = self.latch_state.lock();
        match *state {
            LatchState::Rolling => None,
            LatchState::Triggered {
                trigger_timestamp_ns,
                shutter_sequence_id,
                remaining_post_frames,
            } => {
                if remaining_post_frames <= 1 {
                    *state = LatchState::Completed {
                        trigger_timestamp_ns,
                    };
                    Some(*state)
                } else {
                    *state = LatchState::Triggered {
                        trigger_timestamp_ns,
                        shutter_sequence_id,
                        remaining_post_frames: remaining_post_frames - 1,
                    };
                    None
                }
            }
            LatchState::Completed { .. } => None,
        }
    }

    /// Triggered by shutter press: begins recording post-shutter frames.
    /// If already triggered, preserves the active trigger timestamp to avoid clobbering.
    pub fn trigger_shutter(&self, trigger_timestamp_ns: u64, post_frames: usize) {
        let mut state = self.latch_state.lock();
        if let LatchState::Triggered { .. } = *state {
            return;
        }
        let total = self.total_frames_written.load(Ordering::Relaxed);
        *state = LatchState::Triggered {
            trigger_timestamp_ns,
            shutter_sequence_id: total,
            remaining_post_frames: post_frames,
        };
    }

    /// Checks if shutter latch has been triggered
    pub fn is_latch_triggered(&self) -> bool {
        let state = self.latch_state.lock();
        matches!(
            *state,
            LatchState::Triggered { .. } | LatchState::Completed { .. }
        )
    }

    /// Resets the buffer back to rolling state after Live Photo export is queued
    pub fn reset_latch(&self) {
        let mut state = self.latch_state.lock();
        *state = LatchState::Rolling;
    }

    /// Retrieves chronological sequence of valid frame metadata for video encoding
    pub fn get_chronological_metadata(&self) -> Vec<FrameMetadata> {
        let current_head = self.write_head.load(Ordering::Relaxed) % self.capacity;
        let mut result = Vec::with_capacity(self.capacity);

        // Read from oldest to newest: from current_head to capacity-1, then 0 to current_head-1
        for i in 0..self.capacity {
            let idx = (current_head + i) % self.capacity;
            let slot = self.slots[idx].lock();
            if slot.is_valid {
                result.push(slot.metadata.clone());
            }
        }
        result
    }

    /// Retrieves chronological sequence of all valid frames (metadata + byte data)
    pub fn get_chronological_frames(&self) -> Vec<(FrameMetadata, Vec<u8>)> {
        let current_head = self.write_head.load(Ordering::Relaxed) % self.capacity;
        let mut result = Vec::with_capacity(self.capacity);

        for i in 0..self.capacity {
            let idx = (current_head + i) % self.capacity;
            let slot = self.slots[idx].lock();
            if slot.is_valid {
                result.push((slot.metadata.clone(), slot.data.clone()));
            }
        }
        result
    }

    /// Retrieves the most recently written valid frame
    pub fn latest_frame(&self) -> Option<(FrameMetadata, Vec<u8>)> {
        let total = self.total_frames_written.load(Ordering::Relaxed);
        if total == 0 {
            return None;
        }
        let head = self.write_head.load(Ordering::Relaxed);
        let latest_idx = (head + self.capacity - 1) % self.capacity;
        let slot = self.slots[latest_idx].lock();
        if slot.is_valid {
            Some((slot.metadata.clone(), slot.data.clone()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_rolling_and_trigger() {
        let buffer = LivePhotoRingBuffer::new(10, 1024);
        let dummy_data = vec![0xAB; 1024];

        // Push 15 frames (wraps around)
        for i in 0..15 {
            let meta = FrameMetadata {
                sequence_id: i,
                timestamp_ns: i * 16_666_666,
                width: 1920,
                height: 1080,
                stride_y: 1920,
                stride_uv: 1920,
                exposure_time_ns: 5_000_000,
                iso: 100,
            };
            buffer.push_frame(meta, &dummy_data);
        }

        let meta_list = buffer.get_chronological_metadata();
        assert_eq!(meta_list.len(), 10);
        // Oldest should be sequence_id 5, newest should be sequence_id 14
        assert_eq!(meta_list[0].sequence_id, 5);
        assert_eq!(meta_list[9].sequence_id, 14);

        // Trigger shutter with 3 post-frames
        buffer.trigger_shutter(15 * 16_666_666, 3);

        // Push frame 15
        let res1 = buffer.push_frame(
            FrameMetadata {
                sequence_id: 15,
                timestamp_ns: 15 * 16_666_666,
                width: 1920,
                height: 1080,
                stride_y: 1920,
                stride_uv: 1920,
                exposure_time_ns: 5_000_000,
                iso: 100,
            },
            &dummy_data,
        );
        assert!(res1.is_none());

        // Push frame 16
        let res2 = buffer.push_frame(
            FrameMetadata {
                sequence_id: 16,
                timestamp_ns: 16 * 16_666_666,
                width: 1920,
                height: 1080,
                stride_y: 1920,
                stride_uv: 1920,
                exposure_time_ns: 5_000_000,
                iso: 100,
            },
            &dummy_data,
        );
        assert!(res2.is_none());

        // Push frame 17 (3rd post-frame: triggers Completed)
        let res3 = buffer.push_frame(
            FrameMetadata {
                sequence_id: 17,
                timestamp_ns: 17 * 16_666_666,
                width: 1920,
                height: 1080,
                stride_y: 1920,
                stride_uv: 1920,
                exposure_time_ns: 5_000_000,
                iso: 100,
            },
            &dummy_data,
        );
        assert!(matches!(res3, Some(LatchState::Completed { .. })));

        // Verify full frame data retrieval
        let frames = buffer.get_chronological_frames();
        assert_eq!(frames.len(), 10);
        assert_eq!(frames[0].0.sequence_id, 8);
        assert_eq!(frames[9].0.sequence_id, 17);
        assert_eq!(frames[9].1.len(), 1024);
        assert_eq!(frames[9].1[0], 0xAB);

        // Verify latest_frame
        let latest = buffer.latest_frame().unwrap();
        assert_eq!(latest.0.sequence_id, 17);
        assert_eq!(latest.1, dummy_data);
    }

    #[test]
    fn test_lazy_allocation_and_no_truncation() {
        // Capacity 5, initial slot size 0 (lazy)
        let buffer = LivePhotoRingBuffer::new(5, 0);

        // Push frame with 2048 bytes
        let data = vec![0x42u8; 2048];
        let meta = FrameMetadata {
            sequence_id: 1,
            timestamp_ns: 1_000_000,
            width: 1920,
            height: 1080,
            stride_y: 1920,
            stride_uv: 1920,
            exposure_time_ns: 10_000_000,
            iso: 200,
        };

        buffer.push_frame(meta, &data);

        let latest = buffer.latest_frame().unwrap();
        assert_eq!(latest.0.sequence_id, 1);
        assert_eq!(latest.1.len(), 2048); // NOT truncated!
        assert_eq!(latest.1, data);
    }

    #[test]
    fn test_trigger_timestamp_preservation() {
        let buffer = LivePhotoRingBuffer::new(5, 128);
        assert!(!buffer.is_latch_triggered());

        buffer.trigger_shutter(100_000_000, 3);
        assert!(buffer.is_latch_triggered());

        // Second trigger attempt with different timestamp should be ignored (preserves first trigger)
        buffer.trigger_shutter(999_999_999, 10);
        let state = buffer.latch_state.lock();
        if let LatchState::Triggered {
            trigger_timestamp_ns,
            remaining_post_frames,
            ..
        } = *state
        {
            assert_eq!(trigger_timestamp_ns, 100_000_000);
            assert_eq!(remaining_post_frames, 3);
        } else {
            panic!("Expected Triggered state");
        }
    }
}
