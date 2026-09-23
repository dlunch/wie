use alloc::{boxed::Box, format};

use wie_util::{Result, WieError};

use crate::canvas::{Image, Rgb332Pixel, VecImageBuffer};

pub fn decode_res(data: &[u8]) -> Result<Box<dyn Image>> {
    if !data.starts_with(b"\xce\xfa\xad\xde") {
        return Err(WieError::FatalError("Invalid RES image".into()));
    }

    let Some(header_size) = data.get(6..8) else {
        return Err(WieError::FatalError("Invalid RES image".into()));
    };
    let mut offset = u16::from_le_bytes([header_size[0], header_size[1]]) as usize;

    loop {
        let Some(header_end) = offset.checked_add(8) else {
            return Err(WieError::FatalError("Invalid RES image".into()));
        };
        let Some(header) = data.get(offset..header_end) else {
            break;
        };

        let resource_type = header[0];
        let width = header[1] as usize;
        let height = header[2] as usize;
        let bits_per_pixel = header[3];
        if bits_per_pixel != 8 {
            return Err(WieError::Unimplemented(format!("Unsupported RES color depth {bits_per_pixel}")));
        }

        let Some(pixel_count) = width.checked_mul(height) else {
            return Err(WieError::FatalError("Invalid RES image".into()));
        };
        offset = header_end;

        match resource_type {
            2 => {
                let Some(pixel_end) = offset.checked_add(pixel_count) else {
                    return Err(WieError::FatalError("Invalid RES image".into()));
                };
                let Some(pixels) = data.get(offset..pixel_end) else {
                    return Err(WieError::FatalError("Invalid RES image".into()));
                };

                return Ok(Box::new(VecImageBuffer::<Rgb332Pixel>::from_raw(
                    width as u32,
                    height as u32,
                    pixels.to_vec(),
                )));
            }
            3 => {
                let Some(&frame_count) = data.get(offset) else {
                    return Err(WieError::FatalError("Invalid RES image".into()));
                };
                let Some(frame_data_size) = pixel_count.checked_mul(frame_count as usize) else {
                    return Err(WieError::FatalError("Invalid RES image".into()));
                };
                let Some(next_offset) = offset.checked_add(1).and_then(|offset| offset.checked_add(frame_data_size)) else {
                    return Err(WieError::FatalError("Invalid RES image".into()));
                };
                offset = next_offset;
            }
            _ => {
                return Err(WieError::Unimplemented(format!("Unsupported RES type {resource_type}")));
            }
        }
    }

    Err(WieError::FatalError("RES does not contain a static icon".into()))
}
