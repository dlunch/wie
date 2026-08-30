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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::decode_res;

    #[test]
    fn decodes_static_icon_after_animation() {
        let mut resource = vec![0; 15];
        resource[..4].copy_from_slice(b"\xce\xfa\xad\xde");
        resource[6..8].copy_from_slice(&15u16.to_le_bytes());
        resource.extend_from_slice(&[3, 1, 1, 8, 0, 0, 0, 0, 2, 0, 0]);
        resource.extend_from_slice(&[2, 2, 1, 8, 0, 0, 0, 0, 0xe0, 0x03]);

        let image = decode_res(&resource).unwrap();

        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 1);
        let red = image.get_pixel(0, 0);
        let blue = image.get_pixel(1, 0);
        assert_eq!((red.r, red.g, red.b, red.a), (252, 0, 0, 255));
        assert_eq!((blue.r, blue.g, blue.b, blue.a), (0, 0, 255, 255));
    }
}
