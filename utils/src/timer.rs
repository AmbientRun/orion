/// Represents a timer which will be invoked at a point in time
pub struct Timer {
    /// When does this timer expire
    deadline: std::time::Instant,
}

pub struct TimerWheel {}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    #[test]
    fn timers() {
        let now = ::time::Instant::now();
    }
}
