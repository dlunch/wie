use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Canvas, Graphics};

// class com.xce.lcdui.XTextField
pub struct XTextField;

impl XTextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/XTextField",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                    Self::init,
                    Default::default(),
                ),
                JavaMethodProto::new("setFocus", "(Z)V", Self::set_focus, Default::default()),
                JavaMethodProto::new("setBounds", "(IIII)V", Self::set_bounds, Default::default()),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, Default::default()),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, Default::default()),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, Default::default()),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, Default::default()),
                JavaMethodProto::new("getText", "()Ljava/lang/String;", Self::get_text, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("text", "Ljava/lang/String;", Default::default())],
            access_flags: Default::default(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        text2: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
        canvas: ClassInstanceRef<Canvas>,
    ) -> JvmResult<()> {
        tracing::debug!(
            "com.xce.lcdui.XTextField::<init>({:?}, {:?}, {}, {}, {:?})",
            &this,
            &text2,
            max_size,
            constraints,
            &canvas
        );

        // Call the parent constructor
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text2).await?;

        Ok(())
    }

    async fn set_focus(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::setFocus({this:?}, {focus})");

        Ok(())
    }

    async fn set_bounds(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::setBounds({this:?}, {x}, {y}, {width}, {height})");

        Ok(())
    }

    async fn key_pressed(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::keyPressed({this:?}, {key_code})");

        Ok(())
    }

    async fn key_repeated(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::keyRepeated({this:?}, {key_code})");

        Ok(())
    }

    async fn key_released(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::keyReleased({this:?}, {key_code})");

        Ok(())
    }

    async fn paint(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::paint({this:?}, {graphics:?})");

        Ok(())
    }

    async fn get_text(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("com.xce.lcdui.XTextField::getText({this:?})");

        let text = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        Ok(text)
    }
}
