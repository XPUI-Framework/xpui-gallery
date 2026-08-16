//! A stand-in for the firmware's heap figures.
//!
//! The screen this came from read real numbers from the device it ran on.
//! There is no device here, so this makes some up — but it makes up *moving*
//! ones, which matters more than it sounds: a screen showing a constant would
//! let a repaint test pass without anything having been redrawn, and that is
//! the exact class of bug the tests around it exist to catch.

/// One reading, in bytes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub total: i32,
    pub free: i32,
    /// The biggest single allocation that would still fit.
    ///
    /// Always below `free`, because a heap that has been used is a heap that
    /// has been fragmented. That relationship is the interesting one on a
    /// device: plenty free and nowhere to put anything is a real failure.
    pub largest_block: i32,
    /// The lowest `free` has ever been. A high-water mark in reverse, and the
    /// only figure here that says whether the device nearly ran out.
    pub min_free: i32,
}

/// Successive readings from a heap that is being used.
#[derive(Clone, Debug)]
pub struct Heap {
    total: i32,
    /// Counts readings rather than time. Nothing here needs a clock, and one
    /// would make the tests depend on how fast they ran.
    taken: core::cell::Cell<u32>,
}

impl Default for Heap {
    fn default() -> Self {
        // A figure in the range a small microcontroller actually has, so the
        // formatted numbers look like the ones a person would see on hardware.
        Heap {
            total: 380 * 1024,
            taken: core::cell::Cell::new(0),
        }
    }
}

impl Heap {
    /// Takes a reading, and moves on.
    ///
    /// The walk is deliberate rather than random: a test can predict it, and a
    /// screenshot golden stays stable because the sequence restarts with every
    /// fresh `Heap`.
    pub fn reading(&self) -> Reading {
        let step = self.taken.get();
        self.taken.set(step.wrapping_add(1));

        // Sawtooth: allocations accumulate and are periodically released, which
        // is roughly what a screen stack does.
        let used = 40 * 1024 + (step as i32 % 16) * 3_500;
        let free = self.total - used;

        Reading {
            total: self.total,
            free,
            largest_block: free / 2 - (step as i32 % 4) * 128,
            min_free: self.total - (40 * 1024 + 15 * 3_500),
        }
    }
}
