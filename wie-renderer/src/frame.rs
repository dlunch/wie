use wie_backend::canvas::Image;

pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// Opaque, row-major, sRGB-encoded RGBA bytes.
    pub rgba: Vec<u8>,
}

impl Frame {
    pub fn from_image(image: &dyn Image) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
            rgba: image.colors().into_iter().flat_map(|color| [color.r, color.g, color.b, 255]).collect(),
        }
    }
}
