use std::{cmp::Reverse, collections::BinaryHeap};

use slotmap::SlotMap;

use crate::time::Instant;

/// Represents a timer which will be invoked at a point in time
pub struct Timer {
    /// When does this timer expire
    deadline: Instant,
}

slotmap::new_key_type! {
    pub struct TimerKey;
}

pub struct TimerWheel {
    timers: SlotMap<TimerKey, Timer>,
    /// Get the timers closest in time
    heap: BinaryHeap<(Reverse<Instant>, TimerKey)>,
}

#[cfg(test)]
mod test {

    use super::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    #[test]
    fn timers() {
        let now = Instant::now();
        eprintln!("Now: {now:?}");
    }
}
