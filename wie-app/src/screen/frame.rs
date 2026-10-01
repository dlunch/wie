pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// Opaque, row-major pixels in softbuffer's 0x00RRGGBB format.
    pub pixels: Vec<u32>,
}

#[cfg(any(not(target_os = "linux"), test))]
impl Frame {
    pub(super) fn draw_into(&self, pixels: &mut [u32], width: u32, height: u32) {
        pixels.fill(0);
        let (scaled_width, scaled_height) = if u64::from(width) * u64::from(self.height) <= u64::from(height) * u64::from(self.width) {
            (width, (u64::from(self.height) * u64::from(width) / u64::from(self.width)).max(1) as u32)
        } else {
            ((u64::from(self.width) * u64::from(height) / u64::from(self.height)).max(1) as u32, height)
        };
        let left = (width - scaled_width) / 2;
        let top = (height - scaled_height) / 2;
        for y in 0..scaled_height {
            let source_y = u64::from(y) * u64::from(self.height) / u64::from(scaled_height);
            let row = &mut pixels[((top + y) * width + left) as usize..][..scaled_width as usize];
            for (x, pixel) in row.iter_mut().enumerate() {
                let source_x = x as u64 * u64::from(self.width) / u64::from(scaled_width);
                *pixel = self.pixels[(source_y * u64::from(self.width) + source_x) as usize];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Frame;

    #[test]
    fn scales_pixels_without_blending_and_centers_with_black_borders() {
        let frame = Frame {
            width: 2,
            height: 1,
            pixels: vec![0x00ff0000, 0x000000ff],
        };
        let mut pixels = vec![u32::MAX; 16];
        frame.draw_into(&mut pixels, 4, 4);
        assert_eq!(
            pixels,
            [0, 0, 0, 0, 0xff0000, 0xff0000, 0xff, 0xff, 0xff0000, 0xff0000, 0xff, 0xff, 0, 0, 0, 0]
        );

        let mut pixel = [0];
        frame.draw_into(&mut pixel, 1, 1);
        assert_eq!(pixel, [0xff0000]);
    }
}
