use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.FormComponent
pub struct FormComponent;

impl FormComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/FormComponent",
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Z)V", Self::init_with_orientation, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.FormComponent::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await
    }

    async fn init_with_orientation(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, vertical: bool) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.FormComponent::<init>({this:?}, {vertical})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await
    }
}
