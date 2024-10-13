use core::mem;

use bytemuck::{Pod, Zeroable};

use crate::{method::ParamConverter, WIPICContext, WIPICWord};

/// _MC_GrpContext
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICGraphicsContext {
    pub mask: WIPICWord,
    /// top-left x, y, bottom-right x, y
    pub clip: [WIPICWord; 4],
    pub fgpxl: WIPICWord,
    pub bgpxl: WIPICWord,
    pub transpxl: WIPICWord,
    pub alpha: WIPICWord,
    /// x, y
    pub offset: [WIPICWord; 2],
    pub pixel_op_func_ptr: WIPICWord, // MC_GrpPixelOpProc
    pub param1: WIPICWord,
    pub reserved: WIPICWord,
    pub font: WIPICWord,
    pub style: WIPICWord,
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

impl ParamConverter<WIPICGraphicsContextIdx> for WIPICGraphicsContextIdx {
    fn convert(_context: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICGraphicsContextIdx {
        if raw >= (Self::ClipIdx as WIPICWord) && raw <= (Self::OutlineIdx as WIPICWord) {
            // SAFETY: WIPICGraphicsContextIdx has CWord repr and is unit only.
            let x: Self = unsafe { mem::transmute(raw) };
            x
        } else {
            Self::Invalid
        }
    }
}
