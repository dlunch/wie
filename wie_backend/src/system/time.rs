use core::ops::{Add, Sub};
use std::{cell::RefCell, rc::Rc};

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

pub struct Time {
    platform: Rc<RefCell<Box<dyn Platform>>>,
}

impl Time {
    pub fn new(platform: Rc<RefCell<Box<dyn Platform>>>) -> Self {
        Self { platform }
    }

    pub fn now(&self) -> Instant {
        self.platform.borrow().now()
    }
}
