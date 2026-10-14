use alloc::{boxed::Box, format, vec::Vec};

use bytemuck::{Pod, Zeroable, pod_read_unaligned};

use wie_util::{Result, WieError};

use crate::canvas::{ArgbPixel, Color, Image, PixelType, Rgb332Pixel, Rgb565Pixel, VecImageBuffer};

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
    let invalid = || WieError::FatalError("Invalid LBMP image".into());
    let header: LbmpHeader = pod_read_unaligned(data.get(..24).ok_or_else(invalid)?);
    let data = &data[24..];
    let kind = u32::from_le(header.r#type);
    let width = u32::from_le(header.width);
    let height = u32::from_le(header.height);
    let size = u32::from_le(header.size);
    let mask = u32::from_le(header.mask);

    let pixel_count = width.checked_mul(height).ok_or_else(invalid)?;
    let plane_size = height.div_ceil(8).checked_mul(width).ok_or_else(invalid)?;
    let expected_size = match kind {
        2 => plane_size,
        8 => pixel_count,
        16 => pixel_count.checked_mul(2).ok_or_else(invalid)?,
        _ => return Err(WieError::Unimplemented(format!("Unsupported LBMP type {kind}"))),
    };
    if width == 0 || height == 0 || size != expected_size || mask > 1 {
        return Err(invalid());
    }
    let image_size = size.checked_mul(if kind == 2 { 2 } else { 1 }).ok_or_else(invalid)? as usize;
    let pixels = data.get(..image_size).ok_or_else(invalid)?;
    let transparency = if mask == 1 {
        data.get(image_size..)
            .and_then(|data| data.get(..plane_size as usize))
            .ok_or_else(invalid)?
    } else {
        &[]
    };

    let mut colors = Vec::with_capacity(pixel_count as usize);
    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) as usize;
            // Bitplanes store eight vertical pixels per byte, top pixel first.
            let plane_offset = ((y / 8) * width + x) as usize;
            let bit = y % 8;
            let mut color = match kind {
                8 => Rgb332Pixel::to_color(pixels[offset]),
                16 => Rgb565Pixel::to_color(u16::from_le_bytes([pixels[offset * 2], pixels[offset * 2 + 1]])),
                _ => {
                    let high = (pixels[plane_offset] >> bit) & 1;
                    let low = (pixels[plane_offset + size as usize] >> bit) & 1;
                    let gray = 255 - (high * 2 + low) * 85;
                    Color {
                        a: 255,
                        r: gray,
                        g: gray,
                        b: gray,
                    }
                }
            };
            if mask == 1 && transparency[plane_offset] & (1 << bit) != 0 {
                color.a = 0;
            }
            colors.push(ArgbPixel::from_color(color));
        }
    }

    Ok(Box::new(VecImageBuffer::<ArgbPixel>::from_raw(width, height, colors)))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::canvas::decode_image;

    #[test]
    fn truncated_lbmp_returns_an_error() {
        for (kind, size, pixel_bytes) in [(2u32, 4u32, 8), (8, 18, 18), (16, 36, 36)] {
            let mut data = vec![0];
            data.extend_from_slice(b"LBMP");
            for value in [kind, 2, 9, size, 1] {
                data.extend_from_slice(&value.to_le_bytes());
            }
            data.resize(data.len() + pixel_bytes + 4, 0);
            for end in 1..data.len() {
                assert!(decode_image(&data[1..end]).is_err(), "type {kind}, length {}", end - 1);
            }
            assert!(decode_image(&data[1..]).is_ok());
            data.extend_from_slice(b"next image");
            assert!(decode_image(&data[1..]).is_ok());
        }
    }
}
