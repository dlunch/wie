use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::Image;

pub const FLIP_HORIZONTAL: i32 = 0x2000;
pub const FLIP_VERTICAL: i32 = 0x4000;
pub const ROTATE_90: i32 = 90;
pub const ROTATE_180: i32 = 180;
pub const ROTATE_270: i32 = 270;
pub const TYPE_BYTE_1_GRAY: i32 = 1;
pub const TYPE_BYTE_1_GRAY_VERTICAL: i32 = -1;
pub const TYPE_BYTE_2_GRAY: i32 = 2;
pub const TYPE_BYTE_4_GRAY: i32 = 4;
pub const TYPE_BYTE_8_GRAY: i32 = 8;
pub const TYPE_BYTE_332_RGB: i32 = 332;
pub const TYPE_USHORT_4444_ARGB: i32 = 4444;
pub const TYPE_USHORT_444_RGB: i32 = 444;
pub const TYPE_USHORT_555_RGB: i32 = 555;
pub const TYPE_USHORT_1555_ARGB: i32 = 1555;
pub const TYPE_USHORT_565_RGB: i32 = 565;
pub const TYPE_INT_888_RGB: i32 = 888;
pub const TYPE_INT_8888_ARGB: i32 = 8888;

// interface com.nokia.mid.ui.DirectGraphics
#[java_class(name = "com/nokia/mid/ui/DirectGraphics", public, interface, abstract)]
pub struct DirectGraphics {
    #[field(public, static, final)]
    FLIP_HORIZONTAL: i32,
    #[field(public, static, final)]
    FLIP_VERTICAL: i32,
    #[field(public, static, final)]
    ROTATE_90: i32,
    #[field(public, static, final)]
    ROTATE_180: i32,
    #[field(public, static, final)]
    ROTATE_270: i32,
    #[field(public, static, final)]
    TYPE_BYTE_1_GRAY: i32,
    #[field(public, static, final)]
    TYPE_BYTE_1_GRAY_VERTICAL: i32,
    #[field(public, static, final)]
    TYPE_BYTE_2_GRAY: i32,
    #[field(public, static, final)]
    TYPE_BYTE_4_GRAY: i32,
    #[field(public, static, final)]
    TYPE_BYTE_8_GRAY: i32,
    #[field(public, static, final)]
    TYPE_BYTE_332_RGB: i32,
    #[field(public, static, final)]
    TYPE_USHORT_4444_ARGB: i32,
    #[field(public, static, final)]
    TYPE_USHORT_444_RGB: i32,
    #[field(public, static, final)]
    TYPE_USHORT_555_RGB: i32,
    #[field(public, static, final)]
    TYPE_USHORT_1555_ARGB: i32,
    #[field(public, static, final)]
    TYPE_USHORT_565_RGB: i32,
    #[field(public, static, final)]
    TYPE_INT_888_RGB: i32,
    #[field(public, static, final)]
    TYPE_INT_8888_ARGB: i32,
}

#[java_methods]
impl DirectGraphics {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphics::<clinit>");

        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "FLIP_HORIZONTAL", "I", FLIP_HORIZONTAL)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "FLIP_VERTICAL", "I", FLIP_VERTICAL)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "ROTATE_90", "I", ROTATE_90)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "ROTATE_180", "I", ROTATE_180)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "ROTATE_270", "I", ROTATE_270)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_BYTE_1_GRAY", "I", TYPE_BYTE_1_GRAY)
            .await?;
        jvm.put_static_field(
            "com/nokia/mid/ui/DirectGraphics",
            "TYPE_BYTE_1_GRAY_VERTICAL",
            "I",
            TYPE_BYTE_1_GRAY_VERTICAL,
        )
        .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_BYTE_2_GRAY", "I", TYPE_BYTE_2_GRAY)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_BYTE_4_GRAY", "I", TYPE_BYTE_4_GRAY)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_BYTE_8_GRAY", "I", TYPE_BYTE_8_GRAY)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_BYTE_332_RGB", "I", TYPE_BYTE_332_RGB)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_USHORT_4444_ARGB", "I", TYPE_USHORT_4444_ARGB)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_USHORT_444_RGB", "I", TYPE_USHORT_444_RGB)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_USHORT_555_RGB", "I", TYPE_USHORT_555_RGB)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_USHORT_1555_ARGB", "I", TYPE_USHORT_1555_ARGB)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_USHORT_565_RGB", "I", TYPE_USHORT_565_RGB)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_INT_888_RGB", "I", TYPE_INT_888_RGB)
            .await?;
        jvm.put_static_field("com/nokia/mid/ui/DirectGraphics", "TYPE_INT_8888_ARGB", "I", TYPE_INT_8888_ARGB)
            .await?;

        Ok(())
    }

    #[method(name = "setARGBColor", public)]
    async fn set_argb_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, argb_color: i32) -> JvmResult<()>;

    #[method(public)]
    async fn draw_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        anchor: i32,
        manipulation: i32,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn draw_triangle(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        argb_color: i32,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn draw_polygon(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x_points: ClassInstanceRef<Array<i32>>,
        x_offset: i32,
        y_points: ClassInstanceRef<Array<i32>>,
        y_offset: i32,
        n_points: i32,
        argb_color: i32,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn fill_triangle(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        argb_color: i32,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn fill_polygon(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x_points: ClassInstanceRef<Array<i32>>,
        x_offset: i32,
        y_points: ClassInstanceRef<Array<i32>>,
        y_offset: i32,
        n_points: i32,
        argb_color: i32,
    ) -> JvmResult<()>;

    #[method(name = "drawPixels", public)]
    async fn draw_pixels_ints(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i32>>,
        transparency: bool,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) -> JvmResult<()>;

    #[method(name = "drawPixels", public)]
    async fn draw_pixels_bytes(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i8>>,
        transparency_mask: ClassInstanceRef<Array<i8>>,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) -> JvmResult<()>;

    #[method(name = "drawPixels", public)]
    async fn draw_pixels_shorts(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i16>>,
        transparency: bool,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) -> JvmResult<()>;

    #[method(name = "getPixels", public)]
    async fn get_pixels_ints(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i32>>,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) -> JvmResult<()>;

    #[method(name = "getPixels", public)]
    async fn get_pixels_bytes(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i8>>,
        transparency_mask: ClassInstanceRef<Array<i8>>,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) -> JvmResult<()>;

    #[method(name = "getPixels", public)]
    async fn get_pixels_shorts(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i16>>,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn get_native_pixel_format(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn get_alpha_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;
}
