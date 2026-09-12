#![no_std]
extern crate alloc;

use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::{future::Future, pin::Pin};

use bytemuck::{Pod, Zeroable};

pub mod ir;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegionKey {
    pub pc: u32,
    pub thumb: bool,
    pub cpu_mode: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodePageStamp {
    pub page: u32,
    pub version: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CompiledHandle {
    pub slot: u32,
}

#[derive(Clone, Debug)]
pub struct CodeImage {
    pub address: u32,
    pub bytes: Vec<u8>,
    pub source: Vec<CodePageStamp>,
}

pub struct CompileRegion {
    pub ir: ir::RegionIr,
    pub source: Vec<CodePageStamp>,
    pub source_bytes: Vec<(u32, Vec<u8>)>,
}

pub struct CompileRequest {
    pub images: Arc<[CodeImage]>,
    /// Each step emits one region or makes bounded decoder progress without retaining IR.
    pub regions: Box<dyn Iterator<Item = Option<CompileRegion>> + Send>,
}
pub type PreparationFuture = Pin<Box<dyn Future<Output = Result<CompiledArtifact, String>> + Send>>;

#[derive(Clone, Debug)]
pub struct ManifestRegion {
    pub entry: RegionKey,
    pub instruction_pcs: Vec<u32>,
    pub source: Vec<CodePageStamp>,
    pub source_bytes: Vec<(u32, Vec<u8>)>,
}

pub struct CompiledRegion {
    pub manifest: ManifestRegion,
    pub handle: CompiledHandle,
}

pub struct CompiledArtifact {
    pub regions: Vec<CompiledRegion>,
    pub encoded_size: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationState {
    Loading,
    Preparing,
    Ready,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CompiledExit {
    Dispatch = 0,
    Sample = 1,
    Budget = 2,
    End = 3,
    InterpretOne = 4,
    GuestFault = 6,
}

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct RunFrame {
    pub regs: [u32; 16],
    pub cpsr: u32,
    pub end: u32,
    pub budget_remaining: u32,
    pub sample_remaining: u32,
    pub executed: u32,
    pub fault_address: u32,
    pub scratch: u32,
}

pub trait ExecutionAccess {
    /// Looks up current code for a dispatcher transfer without retiring stale handles.
    fn resolve(&self, pc: u32, cpsr: u32) -> Option<CompiledHandle>;
    /// Borrows the mapped 64 KiB guest page containing `address`, without reading or publishing code.
    /// Generated code may reuse the page until its next access method call or synchronous return.
    /// Writes through the page are guest stores and do not publish code.
    fn page(&mut self, address: u32) -> Option<&mut [u8; 0x10000]>;
    /// Borrows an aligned, fully mapped range of `words` (1..=16), wrapping guest addresses at 32 bits.
    /// Returns `None` for unaligned or unmapped ranges. The guest-backed slices contain only
    /// requested bytes, split at a backing-memory boundary into a nonempty prefix and optional remainder;
    /// their lengths are multiples of four and total `words * 4` bytes.
    /// Acquisition does not read or write data, publish code, or change sampling state. Writes
    /// through the slices are guest stores and do not publish code either.
    /// The exclusive borrow is for one synchronous guest instruction, without remapping memory
    /// or suspending execution while the slices are in use.
    fn word_range(&mut self, address: u32, words: u32) -> Option<(&mut [u8], &mut [u8])>;
    fn sample_prepare(&mut self, pc: u32, r7: u32);
}

pub trait CompiledExecutor: Send {
    fn prepare(&mut self, request: CompileRequest, deadline_ms: f64) -> PreparationFuture;
    /// Both `Ok` and `Err` leave the completed instruction prefix in `frame`, including its next PC and counters.
    /// On `Err`, discard this executor and resume in the interpreter without replaying completed writes.
    /// Fallible host calls must fail before guest side effects; arbitrary code or memory corruption is not resumable.
    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit, String>;
    fn retire(&mut self, handles: &[CompiledHandle]);
}

#[cfg(test)]
mod tests {
    use core::mem::{align_of, offset_of, size_of};

    use super::RunFrame;

    #[test]
    fn generated_code_frame_has_a_fixed_plain_data_layout() {
        assert_eq!(size_of::<RunFrame>(), 92);
        assert_eq!(align_of::<RunFrame>(), 4);
        assert_eq!(offset_of!(RunFrame, regs), 0);
        assert_eq!(offset_of!(RunFrame, cpsr), 64);
        assert_eq!(offset_of!(RunFrame, end), 68);
        assert_eq!(offset_of!(RunFrame, budget_remaining), 72);
        assert_eq!(offset_of!(RunFrame, sample_remaining), 76);
        assert_eq!(offset_of!(RunFrame, executed), 80);
        assert_eq!(offset_of!(RunFrame, fault_address), 84);
        assert_eq!(offset_of!(RunFrame, scratch), 88);
        assert_eq!(bytemuck::bytes_of(&RunFrame::default()), &[0; 92]);
    }
}
