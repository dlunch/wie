use crate::method::TypeConverter;
use crate::CContext;
use bytemuck::{Pod, Zeroable};
use core::mem;

/// _MC_GrpContext
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICGraphicsContext {
    pub mask: u32,
    /// top-left x, y, bottom-right x, y
    pub clip: [u32; 4],
    pub fgpxl: u32,
    pub bgpxl: u32,
    pub transpxl: u32,
    pub alpha: u32,
    /// x, y
    pub offset: [u32; 2],
    pub pixel_op_func_ptr: u32, // MC_GrpPixelOpProc
    pub param1: u32,
    pub reserved: u32,
    pub font: u32,
    pub style: u32,
}

#[repr(u32)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum WIPICGraphicsContextIdx {
    ClipIdx = 0,
    FgPixelIdx = 1,
    BgPixelIdx = 2,
    TransPixelIdx = 3,
    AlphaIdx = 4,
    PixelopIdx = 5,
    PixelParam1Idx = 6,
    FontIdx = 7,
    StyleIdx = 8,
    XorModeIdx = 9,
    OffsetIdx = 10,
    OutlineIdx = 11,

    /// Unknown values are mapped to this enum value.
    /// Note that this field doesn't exist in WIPI and the choice of this ordinal is arbitrary.
    Invalid = 0xff,
}

impl TypeConverter<WIPICGraphicsContextIdx> for WIPICGraphicsContextIdx {
    fn to_rust(_context: &mut dyn CContext, raw: u32) -> WIPICGraphicsContextIdx {
        if raw >= (Self::ClipIdx as u32) && raw <= (Self::OutlineIdx as u32) {
            // SAFETY: WIPICGraphicsContextIdx has u32 repr and is unit only.
            let x: Self = unsafe { mem::transmute(raw) };
            x
        } else {
            Self::Invalid
        }
    }

    fn from_rust(_context: &mut dyn CContext, rust: WIPICGraphicsContextIdx) -> u32 {
        rust as u32
    }
}
