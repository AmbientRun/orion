use derive_more::{Add, AddAssign, Sub};
/// Represents an abstract point in time
#[derive(Debug, Clone, Add, AddAssign, Sub, Eq, PartialOrd, Ord)]
pub struct Instant(std::time::Instant);

impl Instant {
    #[inline]
    pub fn now() -> Self {
        Self(Instant::now)
    }

    #[inline]
    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.0.duration_since(earlier)
    }
}
