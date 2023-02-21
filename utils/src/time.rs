use crate::platform;

pub use crate::timer::{sleep, sleep_until, Sleep, TimerWheel};
pub use platform::time::{schedule_wakeup, Instant};
