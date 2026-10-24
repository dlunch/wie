//! Shared source-resolution targets and GPU framebuffer presentation.

mod context;
mod frame;
mod presenter;
mod target;

pub use context::GpuContext;
pub use frame::Frame;
pub use presenter::{Presentation, Presenter};
pub use target::RenderTarget;
