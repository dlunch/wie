use core::ops::{Add, Sub};
use std::time::SystemTime;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    value: u64,
}

impl Instant {
    pub fn from_systemtime(now: SystemTime) -> Self {
        let epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        Self {
            value: epoch.as_millis() as u64,
        }
    }

    pub fn raw(&self) -> u64 {
        self.value
    }
}

impl Add<u64> for Instant {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self { value: self.value + rhs }
    }
}

impl Sub for Instant {
    type Output = u64;

    fn sub(self, rhs: Instant) -> Self::Output {
        self.value - rhs.value
    }
}

#[derive(Default)]
pub struct Time {}

impl Time {
    pub fn new() -> Self {
        Self {}
    }

    pub fn now(&self) -> Instant {
        let now = SystemTime::now();

        Instant::from_systemtime(now)
    }
}
