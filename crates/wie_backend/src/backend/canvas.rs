use std::{io::Cursor, ops::Deref};

use image::{io::Reader as ImageReader, RgbaImage};

pub struct Image {
    image: RgbaImage,
}

impl Image {
    pub fn from_raw(width: u32, height: u32, buf: Vec<u8>) -> Self {
        let image = RgbaImage::from_raw(width, height, buf).unwrap();

        Self { image }
    }

    pub fn from_size(width: u32, height: u32) -> Self {
        let image = RgbaImage::new(width, height);

        Self { image }
    }

    pub fn from_image(image: &[u8]) -> anyhow::Result<Self> {
        let image = ImageReader::new(Cursor::new(image)).with_guessed_format()?.decode()?;

        Ok(Self { image: image.into_rgba8() })
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        4
    }

    pub fn buffer(&self) -> &[u8] {
        self.image.as_raw()
    }
}

pub struct Canvas {
    image: Image,
}

impl Canvas {
    pub fn from_image(image: Image) -> Self {
        Self { image }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, dx: u32, dy: u32, w: u32, h: u32, src: &Image, sx: u32, sy: u32) {
        for j in dy..(dy + h) {
            for i in dx..(dx + w) {
                if i >= self.image.width() || j >= self.image.height() {
                    continue; // TODO remove this
                }
                let pixel = src.image.get_pixel(i - dx + sx, j - dy + sy);
                if pixel[3] == 0 {
                    // TODO we need alpha blending..
                    continue;
                }
                self.image.image.put_pixel(i, j, *pixel);
            }
        }
    }
}

impl Deref for Canvas {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}
