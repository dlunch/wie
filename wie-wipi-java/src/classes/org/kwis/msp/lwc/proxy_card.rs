use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Graphics, lwc::ContainerComponent};

// class org.kwis.msp.lwc.ProxyCard
pub struct ProxyCard;

impl ProxyCard {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ProxyCard",
            parent_class: Some("org/kwis/msp/lcdui/Card"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/ContainerComponent;IIII)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![JavaFieldProto::new(
                "cmp",
                "Lorg/kwis/msp/lwc/ContainerComponent;",
                FieldAccessFlags::empty(),
            )],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<ContainerComponent>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProxyCard::<init>({this:?}, {component:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(IIII)V", (x, y, width, height))
            .await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lwc/ProxyCard",
            "cmp",
            "Lorg/kwis/msp/lwc/ContainerComponent;",
            component,
        )
        .await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProxyCard::paint({this:?}, {graphics:?})");

        let component: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ProxyCard", "cmp", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        jvm.invoke_virtual(
            &component,
            "org/kwis/msp/lwc/ContainerComponent",
            "paint",
            "(Lorg/kwis/msp/lcdui/Graphics;)V",
            (graphics,),
        )
        .await
    }

    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, chr: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.ProxyCard::keyNotify({this:?}, {type}, {chr})");

        let component: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ProxyCard", "cmp", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        jvm.invoke_virtual(&component, "org/kwis/msp/lwc/Component", "keyNotify", "(II)Z", (r#type, chr))
            .await
    }

    async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProxyCard::showNotify({this:?}, {show})");

        let component: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ProxyCard", "cmp", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        jvm.invoke_virtual(&component, "org/kwis/msp/lwc/Component", "showNotify", "(Z)V", (show,))
            .await
    }
}
