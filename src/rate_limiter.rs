use log::*;
use std::cell::Cell;
use std::{thread, time};

pub struct RateLimiter {
    limit: time::Duration,
    last_action_start: Cell<Option<time::Instant>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            limit: time::Duration::from_secs(2),
            last_action_start: Cell::new(None),
        }
    }

    pub fn wait(&self) {
        if let Some(start) = self.last_action_start.get() {
            let duration = self
                .limit
                .checked_sub(start.elapsed())
                .unwrap_or(time::Duration::from_secs(0));

            debug!("Sleeping {} ms", duration.as_millis());
            thread::sleep(duration);
        }

        self.last_action_start.set(Some(time::Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter() {
        let limiter = RateLimiter::new();

        let start = time::Instant::now();

        limiter.wait();
        assert!(
            start.elapsed() < time::Duration::from_millis(10),
            "should be instantneous"
        );

        limiter.wait();
        assert!(
            start.elapsed() > time::Duration::from_secs(1),
            "should for wait the the preconfigured duration"
        );
    }
}
