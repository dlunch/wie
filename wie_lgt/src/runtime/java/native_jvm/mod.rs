//! LGT native-backed JVM (PoC): the AOT-compiled app's classes are registered as
//! JVM classes whose instances are guest(ARM)-memory-backed and whose methods
//! dispatch to ARM code; and the platform classes the app imports are exposed to
//! the native code through `java_load_classes`' fixed-offset function-pointer
//! tables (native -> platform trampolines).
//!
//! Object bridge:
//!  - App instance: a guest object block; `this+0x08` -> zeroed field array (the
//!    layout the AOT code expects: `r1=[this,#8]; str rX,[r1, idx<<2]`).
//!  - `instances` maps `guest_ptr -> ClassInstance` so a guest pointer flowing
//!    through native code (a `this`, an arg, a return) round-trips to its JVM
//!    object. Platform objects returned to native get a small proxy block.
//!
//! Dispatch directions:
//!  - JVM -> native: `LgtMethod::run` marshals args into `r0..r3`,
//!    `run_function(code_ptr)`, marshals the return.
//!  - native -> platform: `java_load_classes` writes a trampoline pointer into
//!    each requested method slot (`static/virtual_method_offsets[idx*4]`); calling
//!    it re-enters the JVM and invokes the matching `wie_wipi_java`/`wie_midp`
//!    method by name+descriptor.
//!
//! See `docs/lgt_abi.md` (consolidated ABI) and `docs/lgt_native_classes.md`
//! (descriptor byte layout).

mod class_model;
mod dispatch;
mod registration;
mod render;
mod shared;

pub use dispatch::{install_platform_tables, register_java_trampoline_handler};
pub use registration::register_app_classes;
pub use shared::LgtJvmShared;

use alloc::string::String;
use alloc::vec::Vec;

const OBJ_HEADER_SIZE: u32 = 0x0c;
const OBJ_PTR_FIELDS_OFFSET: u32 = 0x08;
const FIELD_ARRAY_WORDS: u32 = 256;

/// Per app-class instance-field layout: `(class_name, [(field_name, field_type,
/// object_slot)])`. See `LgtJvmShared::app_field_layouts` (`shared`).
type AppFieldLayouts = Vec<(String, Vec<(String, String, u32)>)>;
