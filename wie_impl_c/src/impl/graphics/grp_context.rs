use core::mem;

use bytemuck::{Pod, Zeroable};

use crate::{method::TypeConverter, CContext, CWord};

/// _MC_GrpContext
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICGraphicsContext {
    pub mask: CWord,
    /// top-left x, y, bottom-right x, y
    pub clip: [CWord; 4],
    pub fgpxl: CWord,
    pub bgpxl: CWord,
    pub transpxl: CWord,
    pub alpha: CWord,
    /// x, y
    pub offset: [CWord; 2],
    pub pixel_op_func_ptr: CWord, // MC_GrpPixelOpProc
    pub param1: CWord,
    pub reserved: CWord,
    pub font: CWord,
    pub style: CWord,
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
    fn to_rust(_context: &mut dyn CContext, raw: CWord) -> WIPICGraphicsContextIdx {
        if raw >= (Self::ClipIdx as CWord) && raw <= (Self::OutlineIdx as CWord) {
            // SAFETY: WIPICGraphicsContextIdx has CWord repr and is unit only.
            let x: Self = unsafe { mem::transmute(raw) };
            x
        } else {
            Self::Invalid
        }
    }

    fn from_rust(_context: &mut dyn CContext, rust: WIPICGraphicsContextIdx) -> CWord {
        rust as CWord
    }
}
