use std::time::Instant;

#[cfg(test)]
use std::time::Duration;

use chrono::{DateTime, Utc};

pub trait Clock {
    fn now_utc(&self) -> DateTime<Utc>;
    fn now_instant(&self) -> Instant;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn now_instant(&self) -> Instant {
        Instant::now()
    }
}

#[cfg(test)]
#[derive(Clone)]
pub struct MockClock {
    state: std::sync::Arc<std::sync::Mutex<MockClockState>>,
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct MockClockState {
    base_utc: DateTime<Utc>,
    base_instant: Instant,
    offset: Duration,
}

#[cfg(test)]
impl MockClock {
    pub fn new(base_utc: DateTime<Utc>) -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(MockClockState {
                base_utc,
                base_instant: Instant::now(),
                offset: Duration::from_secs(0),
            })),
        }
    }

    pub fn advance(&self, delta: Duration) {
        let mut state = self.state.lock().expect("mock clock lock");
        state.offset = state.offset.saturating_add(delta);
    }
}

#[cfg(test)]
impl Clock for MockClock {
    fn now_utc(&self) -> DateTime<Utc> {
        let state = self.state.lock().expect("mock clock lock");
        let offset =
            chrono::Duration::from_std(state.offset).expect("offset should fit chrono::Duration");
        state.base_utc + offset
    }

    fn now_instant(&self) -> Instant {
        let state = self.state.lock().expect("mock clock lock");
        state.base_instant + state.offset
    }
}
