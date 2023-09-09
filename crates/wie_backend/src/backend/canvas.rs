use std::io::Cursor;

use image::io::Reader as ImageReader;

pub struct Canvas {
    width: u32,
    height: u32,
    buf: Vec<u32>,
}

impl Canvas {
    pub fn from_raw(width: u32, height: u32, buf: Vec<u32>) -> Self {
        Self { width, height, buf }
    }

    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buf: vec![0; (width * height) as usize],
        }
    }

    pub fn from_image(image: &[u8]) -> anyhow::Result<Self> {
        let image = ImageReader::new(Cursor::new(image)).with_guessed_format()?.decode()?;
        let rgba = image.into_rgba8();

        let pixels = rgba.pixels().map(|x| u32::from_be_bytes(x.0)).collect::<Vec<_>>();

        Ok(Self {
            width: rgba.width(),
            height: rgba.height(),
            buf: pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        4
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buf
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, dx: u32, dy: u32, w: u32, h: u32, buf: &[u32], sx: u32, sy: u32, line_size: u32) {
        for j in dy..(dy + h) {
            for i in dx..(dx + w) {
                self.buf[(i + j * self.width) as usize] = buf[((i - dx + sx) + (j - dy + sy) * line_size) as usize];
            }
        }
    }
}
