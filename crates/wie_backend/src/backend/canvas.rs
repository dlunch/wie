use std::{io::Cursor, ops::Deref};

use image::{imageops, io::Reader as ImageReader, GenericImageView, RgbaImage};

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
        let src_view = src.image.view(sx, sy, w, h);

        imageops::overlay(&mut self.image.image, src_view.deref(), dx as i64, dy as i64);
    }
}

impl Deref for Canvas {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}
