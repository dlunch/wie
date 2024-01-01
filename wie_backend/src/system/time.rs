use core::ops::{Add, Sub};

use time::OffsetDateTime;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    value: u64,
}

impl Instant {
    pub fn from_systemtime(now: OffsetDateTime) -> Self {
        let epoch = now.unix_timestamp_nanos() / 1000000; // to millis
        Self { value: epoch as u64 }
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
        let now = OffsetDateTime::now_utc();

        Instant::from_systemtime(now)
    }
}
