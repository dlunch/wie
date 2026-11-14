#[allow(clippy::module_inception)]
mod midlet;
mod midlet_state_change_exception;

pub use self::{midlet::MIDlet, midlet_state_change_exception::MIDletStateChangeException};
