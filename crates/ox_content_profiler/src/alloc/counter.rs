use super::snapshot::{AllocDelta, AllocSnapshot};

/// Convenience guard: takes a baseline snapshot on construction, exposes the
/// running delta until dropped. Useful for ad-hoc scope measurements outside
/// of [`crate::Recorder`].
pub struct AllocCounter {
    baseline: AllocSnapshot,
}

impl AllocCounter {
    #[must_use]
    pub fn start() -> Self {
        Self { baseline: AllocSnapshot::capture() }
    }

    #[must_use]
    pub fn delta(&self) -> AllocDelta {
        AllocSnapshot::capture().delta_from(&self.baseline)
    }
}
