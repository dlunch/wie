use std::collections::HashMap;

pub struct Canvas {
    buf: Vec<u32>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            buf: vec![0; (width * height) as usize],
        }
    }

    pub(crate) fn buffer(&self) -> &[u32] {
        &self.buf
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
        let canvas = Canvas::new(width, height);

        self.last_id += 1;
        let handle = self.last_id;

        self.canvases.insert(handle, canvas);

        handle
    }

    pub fn destroy(&mut self, handle: CanvasHandle) {
        self.canvases.remove(&handle);
    }

    pub fn canvas(&mut self, handle: CanvasHandle) -> &mut Canvas {
        self.canvases.get_mut(&handle).unwrap()
    }
}

impl Default for Canvases {
    fn default() -> Self {
        Self::new()
    }
}
