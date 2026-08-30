use core::mem;

use wipi_types::wipic::WIPICWord;

use crate::{WIPICContext, method::ParamConverter};

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
