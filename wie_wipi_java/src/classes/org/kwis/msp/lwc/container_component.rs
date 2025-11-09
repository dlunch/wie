use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lwc::Component;

// class org.kwis.msp.lwc.ContainerComponent
pub struct ContainerComponent;

impl ContainerComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ContainerComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("addComponent", "(Lorg/kwis/msp/lwc/Component;)I", Self::add_component, Default::default()),
                JavaMethodProto::new("removeComponent", "(I)V", Self::remove_component_index, Default::default()),
                JavaMethodProto::new(
                    "removeComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::remove_component,
                    Default::default(),
                ),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ContainerComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn add_component(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, component: ClassInstanceRef<Component>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::addComponent({this:?}, {component:?})");

        Ok(0)
    }

    async fn remove_component_index(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::removeComponent({this:?}, {index})");

        Ok(())
    }

    async fn remove_component(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, component: ClassInstanceRef<Component>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::removeComponent({this:?}, {component:?})");

        Ok(())
    }
}
