use alloc::{vec, vec::Vec};
use core::{
    mem::{forget, size_of},
    slice,
};

use wie_backend::CanvasHandle;
use wie_base::util::{read_generic, write_generic};

use crate::{
    base::{CContext, CMemoryId, CMethodBody, CResult},
    method::MethodImpl,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Framebuffer {
    width: u32,
    height: u32,
    bpl: u32,
    bpp: u32,
    buf: CMemoryId,
}

impl Framebuffer {
    pub fn from_compatible_canvas(context: &mut dyn CContext, canvas_handle: CanvasHandle) -> anyhow::Result<Self> {
        let (width, height, bytes_per_pixel) = {
            let mut canvases = context.backend().canvases_mut();
            let canvas = canvases.canvas(canvas_handle);

            (canvas.width(), canvas.height(), canvas.bytes_per_pixel())
        };

        let buf = context.alloc(width * height * bytes_per_pixel)?;

        Ok(Self {
            width,
            height,
            bpl: width * bytes_per_pixel,
            bpp: bytes_per_pixel * 8,
            buf,
        })
    }

    pub fn from_canvas(context: &mut dyn CContext, canvas_handle: CanvasHandle) -> anyhow::Result<Self> {
        let (width, height, bytes_per_pixel, data) = {
            let mut canvases = context.backend().canvases_mut();
            let canvas = canvases.canvas(canvas_handle);

            let mut data = vec![0; (canvas.width() * canvas.height()) as usize];
            canvas.copy(&mut data);

            (canvas.width(), canvas.height(), canvas.bytes_per_pixel(), data)
        };

        let data = unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len() * 4) }; // TODO
        let buf = context.alloc(width * height * bytes_per_pixel)?;
        context.write_bytes(context.data_ptr(buf)?, data)?;

        Ok(Self {
            width,
            height,
            bpl: width * bytes_per_pixel,
            bpp: bytes_per_pixel * 8,
            buf,
        })
    }

    pub fn data(&self, context: &dyn CContext) -> anyhow::Result<Vec<u32>> {
        let mut raw = context.read_bytes(context.data_ptr(self.buf)?, self.width * self.height * self.bpp / 8)?;

        // TODO
        let data = unsafe { Vec::from_raw_parts(raw.as_mut_ptr() as _, raw.len() / 4, raw.capacity() / 4) };
        forget(raw);

        Ok(data)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Image {
    img: Framebuffer,
    mask: Framebuffer,
    loop_count: u32,
    delay: u32,
    animated: u32,
    buf: CMemoryId,
    offset: u32,
    current: u32,
    len: u32,
}

impl Image {
    pub fn new(context: &mut dyn CContext, buf: CMemoryId, offset: u32, len: u32) -> anyhow::Result<Self> {
        let ptr_image_data = context.data_ptr(buf)?;
        let data = context.read_bytes(ptr_image_data + offset, len)?;
        let canvas_handle = context.backend().canvases_mut().new_canvas_from_image(&data)?;

        let img_framebuffer = Framebuffer::from_canvas(context, canvas_handle)?;
        let mask_framebuffer = Framebuffer {
            width: 0,
            height: 0,
            bpl: 0,
            bpp: 0,
            buf: CMemoryId(0),
        };

        Ok(Self {
            img: img_framebuffer,
            mask: mask_framebuffer,
            loop_count: 0,
            delay: 0,
            animated: 0,
            buf,
            offset,
            current: 0,
            len,
        })
    }
}

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move {
        log::warn!("stub graphics{}", id);

        Ok(0)
    };

    body.into_body()
}

async fn get_screen_framebuffer(context: &mut dyn CContext, a0: u32) -> CResult<CMemoryId> {
    log::debug!("MC_grpGetScreenFrameBuffer({:#x})", a0);

    let screen_canvas = context.backend().screen_canvas();
    let framebuffer = Framebuffer::from_compatible_canvas(context, screen_canvas)?;

    let memory = context.alloc(size_of::<Framebuffer>() as u32)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer)?;

    Ok(memory)
}

async fn create_image(context: &mut dyn CContext, ptr_image: u32, image_data: CMemoryId, offset: u32, len: u32) -> CResult<u32> {
    log::debug!("MC_grpCreateImage({:#x}, {:#x}, {:#x}, {:#x})", ptr_image, image_data.0, offset, len);

    let image = Image::new(context, image_data, offset, len)?;

    let memory = context.alloc(size_of::<Image>() as u32)?;
    write_generic(context, ptr_image, memory)?;
    write_generic(context, context.data_ptr(memory)?, image)?;

    Ok(0)
}

#[allow(clippy::too_many_arguments)]
async fn draw_image(
    context: &mut dyn CContext,
    framebuffer: CMemoryId,
    a1: u32,
    a2: u32,
    a3: u32,
    a4: u32,
    image: CMemoryId,
    a6: u32,
    a7: u32,
    a8: u32,
) -> CResult<u32> {
    log::warn!(
        "stub MC_grpDrawImage({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
        framebuffer.0,
        a1,
        a2,
        a3,
        a4,
        image.0,
        a6,
        a7,
        a8
    );

    let _framebuffer: Framebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;
    let _image: Image = read_generic(context, context.data_ptr(image)?)?;

    Ok(0)
}

async fn flush(context: &mut dyn CContext, a0: u32, framebuffer: CMemoryId, a2: u32, a3: u32, a4: u32, a5: u32) -> CResult<u32> {
    log::warn!(
        "stub MC_grpFlushLcd({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
        a0,
        framebuffer.0,
        a2,
        a3,
        a4,
        a5
    );

    let framebuffer: Framebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    let data = framebuffer.data(context)?;

    let backend = context.backend();
    let mut canvases = backend.canvases_mut();
    let screen_canvas = canvases.canvas(backend.screen_canvas());

    screen_canvas.draw(&data);

    Ok(0)
}

pub fn get_graphics_method_table() -> Vec<CMethodBody> {
    vec![
        gen_stub(0),
        gen_stub(1),
        get_screen_framebuffer.into_body(),
        gen_stub(3),
        gen_stub(4),
        gen_stub(5),
        gen_stub(6),
        gen_stub(7),
        gen_stub(8),
        gen_stub(9),
        gen_stub(10),
        gen_stub(11),
        gen_stub(12),
        draw_image.into_body(),
        gen_stub(14),
        gen_stub(15),
        gen_stub(16),
        gen_stub(17),
        gen_stub(18),
        gen_stub(19),
        gen_stub(20),
        flush.into_body(),
        gen_stub(22),
        gen_stub(23),
        gen_stub(24),
        gen_stub(25),
        gen_stub(26),
        gen_stub(27),
        gen_stub(28),
        gen_stub(29),
        gen_stub(30),
        gen_stub(31),
        create_image.into_body(),
        gen_stub(33),
        gen_stub(34),
        gen_stub(35),
        gen_stub(36),
        gen_stub(37),
        gen_stub(38),
        gen_stub(39),
        gen_stub(40),
        gen_stub(41),
        gen_stub(42),
        gen_stub(43),
        gen_stub(44),
        gen_stub(45),
        gen_stub(46),
        gen_stub(47),
        gen_stub(48),
        gen_stub(49),
        gen_stub(50),
        gen_stub(51),
        gen_stub(52),
        gen_stub(53),
        gen_stub(54),
        gen_stub(55),
        gen_stub(56),
        gen_stub(57),
        gen_stub(58),
        gen_stub(59),
        gen_stub(60),
        gen_stub(61),
        gen_stub(62),
        gen_stub(63),
        gen_stub(64),
        gen_stub(65),
        gen_stub(66),
        gen_stub(67),
        gen_stub(68),
        gen_stub(69),
        gen_stub(70),
        gen_stub(71),
        gen_stub(72),
        gen_stub(73),
        gen_stub(74),
        gen_stub(75),
        gen_stub(76),
        gen_stub(77),
        gen_stub(78),
        gen_stub(79),
    ]
}
