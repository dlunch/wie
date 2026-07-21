//! LGT (LG Telecom) WIPI platform support.
//!
//! Two kinds of app run here. **Clets** are C apps compiled to standard ARM ELF.
//! **Java apps** are AOT-compiled (ez-i / Xceed toolchain): each Java class is emitted
//! as native ARM code with a `.data` descriptor record, rather than JVM bytecode. The
//! AOT app boots through wie's shared lcdui `Main.main` path and runs its real native
//! methods, which reach the platform through the `0x64` "java-interface" import module
//! and per-method trampolines that bridge into wie's JVM. The platform tables the
//! native code dispatches through — a global vtable with a reserved slot 0, per-class
//! overrides for hardcoded `java/lang` slots, and an inheritance-aware instance field
//! layout — are reconstructed in `runtime::java::native_jvm`. The Java-app PoC is
//! kept LGT-specific (`LgtJvmShared`) and does not modify shared `wie_midp` /
//! `wie_wipi_java` classes.
//!
//! See `docs/lgt_abi.md` for the consolidated, reverse-engineered ABI and
//! `docs/lgt_native_classes.md` for the descriptor byte layout.
#![no_std]
extern crate alloc;

mod emulator;
mod runtime;

pub use emulator::LgtEmulator;
