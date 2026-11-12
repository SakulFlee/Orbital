//! CPU-side bookkeeping for asynchronous GPU timestamp readback.
//!
//! The renderer records one set of timestamp queries per frame, copies the
//! resolved values into a staging buffer, and maps that buffer asynchronously.
//! This module tracks which staging slots are still waiting for their map and
//! which completed generation is the newest, without touching `wgpu` objects.

/// Number of timestamp queries recorded per frame.
pub(crate) const TS_COUNT: u32 = 14;

/// Number of staging buffers available for in-flight readbacks.
pub(crate) const STAGING_SLOTS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotState {
    Free,
    Pending,
}

#[derive(Debug, Clone, Copy)]
struct Slot {
    state: SlotState,
    generation: u64,
}

/// Tracks which staging slots are waiting for a completed map.
///
/// A slot is only handed out again after its previous readback was consumed
/// or released, so a pending map is never unmapped or overwritten.
#[derive(Debug)]
pub(crate) struct TimestampReadbackRing {
    slots: [Slot; STAGING_SLOTS],
    next_generation: u64,
}

impl TimestampReadbackRing {
    pub(crate) fn new() -> Self {
        Self {
            slots: [Slot {
                state: SlotState::Free,
                generation: 0,
            }; STAGING_SLOTS],
            next_generation: 1,
        }
    }

    /// Reserves a free slot for this frame and returns its index and
    /// generation.
    pub(crate) fn acquire(&mut self) -> Option<(usize, u64)> {
        let index = self
            .slots
            .iter()
            .position(|slot| slot.state == SlotState::Free)?;
        let generation = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1);
        self.slots[index] = Slot {
            state: SlotState::Pending,
            generation,
        };
        Some((index, generation))
    }

    /// Generation of a pending slot, if it has not completed yet.
    pub(crate) fn pending_generation(&self, index: usize) -> Option<u64> {
        let slot = self.slots.get(index)?;
        (slot.state == SlotState::Pending).then_some(slot.generation)
    }

    /// Frees a slot after its readback completed or failed.
    pub(crate) fn release(&mut self, index: usize) {
        if let Some(slot) = self.slots.get_mut(index) {
            slot.state = SlotState::Free;
        }
    }
}

/// Most recently accepted GPU timestamp sample, in nanoseconds.
///
/// Samples from older generations and samples containing only zeroes are
/// ignored, so a stalled readback keeps the last known value instead of
/// replacing it with an empty one.
#[derive(Debug)]
pub(crate) struct GpuTimestampHistory {
    generation: u64,
    ns: [f64; TS_COUNT as usize],
}

impl GpuTimestampHistory {
    pub(crate) fn new() -> Self {
        Self {
            generation: 0,
            ns: [0.0; TS_COUNT as usize],
        }
    }

    /// Generation of the most recently accepted sample.
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    /// Timestamp values of the most recently accepted sample, in nanoseconds.
    pub(crate) fn ns(&self) -> [f64; TS_COUNT as usize] {
        self.ns
    }

    /// Accepts a completed sample if it is newer than the current one and
    /// contains at least one non-zero timestamp.
    ///
    /// Returns whether the sample was accepted.
    pub(crate) fn accept(
        &mut self,
        generation: u64,
        raw: [f64; TS_COUNT as usize],
        period_ns: f32,
    ) -> bool {
        if generation <= self.generation || raw.iter().all(|value| *value == 0.0) {
            return false;
        }

        self.generation = generation;
        self.ns = raw.map(|value| value * period_ns as f64);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(frame: u64) -> [f64; TS_COUNT as usize] {
        std::array::from_fn(|index| (frame * 100 + index as u64) as f64)
    }

    #[test]
    fn ring_never_reuses_a_pending_slot() {
        let mut ring = TimestampReadbackRing::new();
        let mut slots = Vec::new();
        for _ in 0..STAGING_SLOTS {
            let (slot, _) = ring.acquire().expect("a free slot is available");
            slots.push(slot);
        }

        assert_eq!(slots, (0..STAGING_SLOTS).collect::<Vec<_>>());
        assert!(ring.acquire().is_none(), "every slot is still pending");

        ring.release(slots[0]);
        assert_eq!(ring.acquire().map(|(slot, _)| slot), Some(slots[0]));
    }

    #[test]
    fn history_keeps_the_newest_generation() {
        let mut history = GpuTimestampHistory::new();

        assert!(history.accept(2, sample(2), 1.0));
        assert!(!history.accept(1, sample(1), 1.0));
        assert_eq!(history.generation(), 2);
        assert_eq!(history.ns(), sample(2));

        assert!(history.accept(4, sample(4), 1.0));
        assert_eq!(history.generation(), 4);
        assert_eq!(history.ns(), sample(4));
    }

    #[test]
    fn history_ignores_empty_samples() {
        let mut history = GpuTimestampHistory::new();

        assert!(!history.accept(1, [0.0; TS_COUNT as usize], 1.0));
        assert_eq!(history.generation(), 0);

        assert!(history.accept(2, sample(2), 1.0));
        assert!(!history.accept(3, [0.0; TS_COUNT as usize], 1.0));
        assert_eq!(history.generation(), 2);
        assert_eq!(history.ns(), sample(2));
    }

    #[test]
    fn history_scales_ticks_by_the_queue_period() {
        let mut history = GpuTimestampHistory::new();
        let raw = sample(1);
        assert!(history.accept(1, raw, 2.0));

        let expected = raw.map(|value| value * 2.0);
        assert_eq!(history.ns(), expected);
    }

    #[test]
    fn delayed_gpu_samples_advance_when_readbacks_complete() {
        let mut ring = TimestampReadbackRing::new();
        let mut history = GpuTimestampHistory::new();
        let mut in_flight: Vec<(usize, u64, [f64; TS_COUNT as usize])> = Vec::new();
        let mut previous = history.ns();

        // Record frames faster than the GPU can finish them: a sample only
        // completes two frames after it was submitted.
        for frame in 1..=12 {
            let (slot, generation) = ring.acquire().expect("a free slot is available");
            in_flight.push((slot, generation, sample(frame)));

            if frame <= 2 {
                assert_eq!(history.generation(), 0);
                assert_eq!(history.ns(), previous);
            }

            if in_flight.len() > 2 {
                let (slot, generation, raw) = in_flight.remove(0);
                ring.release(slot);

                assert!(history.accept(generation, raw, 1.0));
                assert_eq!(history.generation(), generation);
                assert_eq!(history.ns(), raw);
                assert_ne!(
                    history.ns(),
                    previous,
                    "a completed frame must change the sample"
                );
                previous = history.ns();
            }
        }

        // Drain the delay so every recorded frame is accounted for.
        for (slot, generation, raw) in in_flight {
            ring.release(slot);
            assert!(history.accept(generation, raw, 1.0));
            assert_ne!(
                history.ns(),
                previous,
                "a completed frame must change the sample"
            );
            previous = history.ns();
        }

        assert_eq!(history.generation(), 12);
        assert_eq!(history.ns(), sample(12));
    }
}
