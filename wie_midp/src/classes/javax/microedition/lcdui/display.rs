use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::{
    lcdui::{Displayable, Graphics},
    midlet::MIDlet,
};

// class javax.microedition.lcdui.Display
pub struct Display;

impl Display {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Display",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    Self::set_current,
                    Default::default(),
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("screenImage", "Ljavax/microedition/lcdui/Image;", Default::default()),
                JavaFieldProto::new("screenGraphics", "Ljavax/microedition/lcdui/Graphics;", Default::default()),
                JavaFieldProto::new("width", "I", Default::default()),
                JavaFieldProto::new("height", "I", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let (width, height) = {
            let mut platform = context.system().platform();
            let screen = platform.screen();
            (screen.width() as i32, screen.height() as i32)
        };

        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;

        let screen_image = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;
        let screen_graphics: ClassInstanceRef<Graphics> = jvm
            .invoke_virtual(&screen_image, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
            .await?;

        jvm.put_field(&mut this, "screenImage", "Ljavax/microedition/lcdui/Image;", screen_image)
            .await?;
        jvm.put_field(&mut this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;", screen_graphics)
            .await?;

        Ok(())
    }

    async fn get_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Display::getWidth({:?})", &this);

        let width = jvm.get_field(&this, "width", "I").await?;

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Display::getHeight({:?})", &this);

        let height = jvm.get_field(&this, "height", "I").await?;

        Ok(height)
    }

    async fn set_current(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Display::setCurrent({:?}, {:?})", &this, displayable);

        Ok(())
    }

    async fn get_display(jvm: &Jvm, _context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub javax.microedition.lcdui.Display::getDisplay({:?})", midlet);

        let instance = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?;

        Ok(instance.into())
    }
}
