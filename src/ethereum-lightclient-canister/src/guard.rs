use crate::state::mutate_state;

#[must_use]
pub struct TimerLogicGuard();

impl TimerLogicGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            let running = s.is_timer_running;
            if running {
                return None;
            }
            s.is_timer_running = true;
            Some(TimerLogicGuard())
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_state(|s| s.is_timer_running = false);
    }
}
