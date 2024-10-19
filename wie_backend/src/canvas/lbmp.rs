use bytemuck::{cast_vec, from_bytes, Pod, Zeroable};

use wie_util::{Result, WieError};

use crate::canvas::{Image, Rgb332Pixel, Rgb565Pixel, VecImageBuffer};

// lcd bitmap file format for skvm

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LbmpHeader {
    descriptor: u32,
    r#type: u32,
    width: u32,
    height: u32,
    size: u32,
    mask: u32,
}

pub fn decode_lbmp(data: &[u8]) -> Result<Box<dyn Image>> {
    let header: &LbmpHeader = from_bytes(&data[0..24]);
    let data = &data[24..];

    if header.r#type == 2 || header.r#type == 3 {
        // unsupported grayscale
        return Err(WieError::Unimplemented(format!("Unsupported grayscale type {}", header.r#type)));
    }

    Ok(if header.r#type == 8 {
        Box::new(VecImageBuffer::<Rgb332Pixel>::from_raw(header.width, header.height, data.to_vec()))
    } else if header.r#type == 16 {
        Box::new(VecImageBuffer::<Rgb565Pixel>::from_raw(
            header.width,
            header.height,
            cast_vec(data.to_vec()),
        ))
    } else {
        return Err(WieError::Unimplemented(format!("Unsupported type {}", header.r#type)));
    })
}
