use std::time::Duration;

use derive_more::{Add, AddAssign, Sub};
use ordered_float::NotNan;

/// Represents an abstract point in time
#[derive(Debug, Clone, Add, AddAssign, Sub, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(NotNan<f64>);

impl Instant {
    #[cfg(not(test))]
    pub fn now() -> Self {
        let perf = web_sys::window().unwrap().performance().unwrap();
        Self(NotNan::new(perf.now()).unwrap())
    }

    #[cfg(test)]
    pub fn now() -> Self {
        Self(Default::default())
    }

    #[inline]
    pub fn duration_since(&self, earlier: Self) -> Duration {
        Duration::from_nanos(((*self.0 - *earlier.0).min(0.0) * 1e6) as _)
    }
}
