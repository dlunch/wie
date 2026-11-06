use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::Image;

// class org.kwis.msp.lwc.LabelComponent
pub struct LabelComponent;

impl LabelComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/LabelComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_label, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/Image;)V",
                    Self::init_with_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    Self::init_with_image_resource,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }

    async fn init_with_label(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?}, {label:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }

    async fn init_with_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?}, {label:?}, {image:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }

    async fn init_with_image_resource(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        resource: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?}, {label:?}, {resource:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }
}
