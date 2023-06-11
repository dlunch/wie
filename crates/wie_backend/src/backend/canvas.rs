use std::{collections::HashMap, io::Cursor};

use image::io::Reader as ImageReader;

pub struct Canvas {
    width: u32,
    height: u32,
    buf: Vec<u32>,
}

impl Canvas {
    fn from_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buf: vec![0; (width * height) as usize],
        }
    }

    fn from_image(image: &[u8]) -> anyhow::Result<Self> {
        let image = ImageReader::new(Cursor::new(image)).with_guessed_format()?.decode()?;
        let rgba = image.into_rgba8();

        let pixels = rgba.pixels().map(|x| u32::from_le_bytes(x.0)).collect::<Vec<_>>();

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

    pub fn draw(&mut self, buf: &[u32]) {
        self.buf.copy_from_slice(buf);
    }

    pub fn copy(&self, target: &mut [u32]) {
        target.copy_from_slice(&self.buf);
    }
}

pub type CanvasHandle = u32;
pub struct Canvases {
    canvases: HashMap<CanvasHandle, Canvas>,
    last_id: u32,
}

impl Canvases {
    pub fn new() -> Self {
        Self {
            canvases: HashMap::new(),
            last_id: 0,
        }
    }

    pub fn new_canvas(&mut self, width: u32, height: u32) -> CanvasHandle {
        let canvas = Canvas::from_size(width, height);

        self.insert_canvas(canvas)
    }

    pub fn new_canvas_from_image(&mut self, image: &[u8]) -> anyhow::Result<CanvasHandle> {
        let canvas = Canvas::from_image(image)?;

        Ok(self.insert_canvas(canvas))
    }

    pub fn destroy(&mut self, handle: CanvasHandle) {
        self.canvases.remove(&handle);
    }

    pub fn canvas(&mut self, handle: CanvasHandle) -> &mut Canvas {
        self.canvases.get_mut(&handle).unwrap()
    }

    fn insert_canvas(&mut self, canvas: Canvas) -> CanvasHandle {
        self.last_id += 1;
        let handle = self.last_id;

        self.canvases.insert(handle, canvas);

        handle
    }
}

impl Default for Canvases {
    fn default() -> Self {
        Self::new()
    }
}
