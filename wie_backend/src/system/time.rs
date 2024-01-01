use core::ops::{Add, Sub};

use crate::Platform;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    value: u64,
}

impl Instant {
    pub fn from_epoch_millis(epoch: u64) -> Self {
        Self { value: epoch }
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

pub struct Time<'a> {
    platform: &'a dyn Platform,
}

impl<'a> Time<'a> {
    pub fn new(platform: &'a dyn Platform) -> Self {
        Self { platform }
    }

    pub fn now(&self) -> Instant {
        self.platform.now()
    }
}
