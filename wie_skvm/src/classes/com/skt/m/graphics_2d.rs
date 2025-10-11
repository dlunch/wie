use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_backend::canvas::Clip;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

// class com.skt.m.Graphics2D
pub struct Graphics2D;

impl Graphics2D {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Graphics2D",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Graphics;)V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "getGraphics2D",
                    "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                    Self::get_graphics2d,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "captureLCD",
                    "(IIII)Ljavax/microedition/lcdui/Image;",
                    Self::capture_lcd,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "drawImage",
                    "(IILjavax/microedition/lcdui/Image;IIIII)V",
                    Self::draw_image,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "createMaskableImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    Self::create_maskable_image,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![JavaFieldProto::new("graphics", "Ljavax/microedition/lcdui/Graphics;", Default::default())],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::<init>({:?}, {:?})", &this, graphics);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "graphics", "Ljavax/microedition/lcdui/Graphics;", graphics)
            .await?;

        Ok(())
    }

    async fn get_graphics2d(jvm: &Jvm, _context: &mut WieJvmContext, graphics: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("com.skt.m.Graphics2D::getGraphics2D({:?})", graphics);

        let instance = jvm
            .new_class("com/skt/m/Graphics2D", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics,))
            .await?;

        Ok(instance.into())
    }

    async fn capture_lcd(jvm: &Jvm, _context: &mut WieJvmContext, x: i32, y: i32, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.skt.m.Graphics2D::captureLCD({}, {}, {}, {})", x, y, width, height);

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        Ok(image)
    }

    #[allow(clippy::too_many_arguments)]
    async fn draw_image(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        tx: i32,
        ty: i32,
        src: ClassInstanceRef<Image>,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::drawImage({this:?}, {tx}, {ty}, {src:?}, {sx}, {sy}, {sw}, {sh}, {mode})");

        if src.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        let src_image = Image::image(jvm, &src).await?;

        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        canvas.draw(
            tx as _,
            ty as _,
            sw as _,
            sh as _,
            &*src_image,
            sx,
            sy,
            Clip {
                x: tx,
                y: ty,
                width: sw as _,
                height: sh as _,
            },
        );

        Ok(())
    }

    async fn create_maskable_image(jvm: &Jvm, _context: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.skt.m.Graphics2D::createMaskableImage({}, {})", width, height);

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        Ok(image)
    }
}
