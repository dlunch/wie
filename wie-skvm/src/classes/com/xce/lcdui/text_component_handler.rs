use alloc::{vec, vec::Vec};

use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::net::wie::MIDPKeyCode;

use super::TextComponent;

// class com.xce.lcdui.TextComponentHandler
pub struct TextComponentHandler;

impl TextComponentHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/TextComponentHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getTextComponentHandler",
                    "()Lcom/xce/lcdui/TextComponentHandler;",
                    Self::get_handler,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setTextComponent",
                    "(Lcom/xce/lcdui/TextComponent;)V",
                    Self::set_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("clear", "()V", Self::clear, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)Z", Self::key_pressed, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyReleased", "(I)Z", Self::key_released, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "instance",
                    "Lcom/xce/lcdui/TextComponentHandler;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
                JavaFieldProto::new("component", "Lcom/xce/lcdui/TextComponent;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::<init>({this:?})");
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn get_handler(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::getTextComponentHandler()");
        let instance: ClassInstanceRef<Self> = jvm
            .get_static_field("com/xce/lcdui/TextComponentHandler", "instance", "Lcom/xce/lcdui/TextComponentHandler;")
            .await?;
        if !instance.is_null() {
            return Ok(instance);
        }
        let instance = jvm.new_class("net/wie/WieTextComponentHandler", "()V", ()).await?;
        jvm.put_static_field(
            "com/xce/lcdui/TextComponentHandler",
            "instance",
            "Lcom/xce/lcdui/TextComponentHandler;",
            instance.clone(),
        )
        .await?;
        Ok(instance.into())
    }

    async fn set_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<TextComponent>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::setTextComponent({this:?}, {component:?})");
        jvm.put_field(
            &mut this,
            "com/xce/lcdui/TextComponentHandler",
            "component",
            "Lcom/xce/lcdui/TextComponent;",
            component,
        )
        .await
    }

    async fn clear(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.TextComponentHandler::clear({this:?})");
        Ok(())
    }

    async fn key_pressed(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::keyPressed({this:?}, {key})");

        let component: ClassInstanceRef<TextComponent> = jvm
            .get_field(&this, "com/xce/lcdui/TextComponentHandler", "component", "Lcom/xce/lcdui/TextComponent;")
            .await?;
        if component.is_null() {
            return Ok(false);
        }
        let constraints: i32 = jvm
            .invoke_virtual(&component, "com/xce/lcdui/TextComponent", "getConstraints", "()I", ())
            .await?;
        let digit = (MIDPKeyCode::KEY_NUM0 as i32..=MIDPKeyCode::KEY_NUM9 as i32).contains(&key);
        if constraints & 0x20000 != 0 {
            return Ok(key == MIDPKeyCode::CLEAR as i32 || digit);
        }

        let size: i32 = jvm.invoke_virtual(&component, "com/xce/lcdui/TextComponent", "size", "()I", ()).await?;
        if key == MIDPKeyCode::CLEAR as i32 {
            if size > 0 {
                let _: () = jvm.invoke_virtual(&component, "com/xce/lcdui/TextComponent", "delete", "()V", ()).await?;
                let _: () = jvm
                    .invoke_virtual(&component, "com/xce/lcdui/TextComponent", "repaint", "()V", ())
                    .await?;
            }
            return Ok(true);
        }
        let max_size: i32 = jvm
            .invoke_virtual(&component, "com/xce/lcdui/TextComponent", "getMaxSize", "()I", ())
            .await?;
        // MIDP NUMERIC, PHONENUMBER, and DECIMAL fields take digits directly and leave other keys to the caller.
        let numeric = matches!(constraints & 0xffff, 2 | 3 | 5);
        if size >= max_size || (numeric && !digit) {
            return Ok(digit);
        }
        let text: Vec<JavaChar> = if numeric {
            vec![key as JavaChar]
        } else {
            // TextComponent cannot be read back, so the host input starts empty and its result is inserted.
            context
                .system()
                .platform()
                .text_input("")
                .await
                .unwrap_or_default()
                .encode_utf16()
                .collect()
        };
        if text.is_empty() {
            return Ok(digit);
        }
        for character in text.into_iter().take((max_size - size) as usize) {
            let _: () = jvm
                .invoke_virtual(&component, "com/xce/lcdui/TextComponent", "insert", "(C)V", (character,))
                .await?;
        }
        let _: () = jvm
            .invoke_virtual(&component, "com/xce/lcdui/TextComponent", "repaint", "()V", ())
            .await?;
        Ok(true)
    }

    async fn key_released(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::warn!("stub com.xce.lcdui.TextComponentHandler::keyReleased({this:?}, {key})");
        Ok(false)
    }
}
