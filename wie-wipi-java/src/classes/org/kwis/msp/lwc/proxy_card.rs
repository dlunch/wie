use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{
    lcdui::{Display, Graphics},
    lwc::ContainerComponent,
};

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
                    "(Lorg/kwis/msp/lwc/ContainerComponent;)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/ContainerComponent;Z)V",
                    Self::init_transparent,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/ContainerComponent;IIII)V",
                    Self::init_with_bounds,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/ContainerComponent;IIIIZ)V",
                    Self::init_with_bounds_and_transparency,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("pointerNotify", "(III)Z", Self::pointer_notify, MethodAccessFlags::PROTECTED),
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

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<ContainerComponent>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProxyCard::<init>({this:?}, {component:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lwc/ProxyCard",
            "cmp",
            "Lorg/kwis/msp/lwc/ContainerComponent;",
            component,
        )
        .await
    }

    async fn init_transparent(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<ContainerComponent>,
        transparent: bool,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProxyCard::<init>({this:?}, {component:?}, {transparent})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(Z)V", (transparent,))
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

    #[allow(clippy::too_many_arguments)]
    async fn init_with_bounds(
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

    #[allow(clippy::too_many_arguments)]
    async fn init_with_bounds_and_transparency(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<ContainerComponent>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        transparent: bool,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ProxyCard::<init>({this:?}, {component:?}, {x}, {y}, {width}, {height}, {transparent})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;
        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lcdui/Card",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                (display, x, y, width, height, transparent),
            )
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

    async fn pointer_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, x: i32, y: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.ProxyCard::pointerNotify({this:?}, {type}, {x}, {y})");

        Ok(false)
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
