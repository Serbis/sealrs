//! Scheduler for needs of actor system. Based on the Timers crate and as fact, is an wrapper above
//! him.

use std::time::Duration;


/// Represents task in scheduler. Task is exists while this guard does not dropped.
pub struct TaskGuard {
    inner: timer::Guard
}

pub struct Scheduler {
    timer: timer::Timer
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            timer: timer::Timer::new()
        }
    }

    /// Plans task for once execution
    pub fn schedule_once<F>(&self, duration: Duration, f: F) -> TaskGuard
        where F: 'static + FnMut() + Send
    {
        let g = self.timer.schedule_with_delay(chrono::Duration::from_std(duration).ok().unwrap(), f);
        TaskGuard { inner: g }
    }

    /// Plans task for periodic execution
    pub fn schedule_periodic<F>(&self, interval: Duration, f: F) -> TaskGuard
        where F: 'static + FnMut() + Send
    {
        let g = self.timer.schedule_repeating(chrono::Duration::from_std(interval).ok().unwrap(), f);
        TaskGuard { inner: g }
    }
}