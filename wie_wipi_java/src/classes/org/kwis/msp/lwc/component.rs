use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.Component
pub struct Component;

impl Component {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/Component",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, Default::default()),
                JavaMethodProto::new("focusNotify", "(Z)V", Self::focus_notify, Default::default()),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, Default::default()),
                JavaMethodProto::new("configure", "(IIIII)V", Self::configure, Default::default()),
                JavaMethodProto::new("setFocus", "()V", Self::set_focus, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.Component::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, chr: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::keyNotify({:?}, {:?}, {:?})", &this, r#type, chr);

        Ok(true)
    }

    async fn focus_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::focusNotify({:?}, {:?})", &this, focus);

        Ok(())
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::showNotify({:?}, {:?})", &this, show);

        Ok(())
    }

    async fn configure(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, w: i32, h: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::configure({this:?}, {x}, {y}, {w}, {h})",);

        Ok(())
    }

    async fn set_focus(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setFocus({:?})", &this,);

        Ok(())
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getHeight({:?})", &this,);

        Ok(0)
    }
}
