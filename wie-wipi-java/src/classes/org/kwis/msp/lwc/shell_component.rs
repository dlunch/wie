use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{
    lcdui::{Card, Display},
    lwc::Component,
};

// class org.kwis.msp.lwc.ShellComponent
pub struct ShellComponent;

impl ShellComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ShellComponent",
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(IIII)V", Self::init_with_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setWorkComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::set_work_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hide", "()V", Self::hide, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_card, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_area, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new("cd", "Lorg/kwis/msp/lcdui/Card;", FieldAccessFlags::PROTECTED)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::<init>({this:?})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;
        let width: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", ()).await?;
        let height: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getHeight", "()I", ()).await?;

        jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "(IIII)V", (0, 0, width, height))
            .await
    }

    async fn init_with_size(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::<init>({this:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await?;

        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "x", "I", x).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "y", "I", y).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "w", "I", width).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "h", "I", height).await?;

        let card = jvm
            .new_class(
                "org/kwis/msp/lwc/ProxyCard",
                "(Lorg/kwis/msp/lwc/ContainerComponent;IIII)V",
                (this.clone(), x, y, width, height),
            )
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;", card)
            .await
    }

    async fn set_work_component(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.ShellComponent::setWorkComponent({this:?}, {component:?})");

        Ok(())
    }

    async fn show(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::show({this:?})");

        let card: ClassInstanceRef<Card> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;")
            .await?;
        let display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "getDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
            .await?;
        jvm.invoke_virtual(
            &display,
            "org/kwis/msp/lcdui/Display",
            "pushCard",
            "(Lorg/kwis/msp/lcdui/Card;)V",
            (card,),
        )
        .await
    }

    async fn hide(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::hide({this:?})");

        let card: ClassInstanceRef<Card> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;")
            .await?;
        let display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "getDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
            .await?;
        let _: bool = jvm
            .invoke_virtual(
                &display,
                "org/kwis/msp/lcdui/Display",
                "removeCard",
                "(Lorg/kwis/msp/lcdui/Card;)Z",
                (card,),
            )
            .await?;

        Ok(())
    }

    async fn is_shown(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::isShown({this:?})");

        let card: ClassInstanceRef<Card> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;")
            .await?;
        jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ()).await
    }

    async fn get_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::getCard({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;")
            .await
    }

    async fn repaint_area(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::repaint({this:?}, {x}, {y}, {width}, {height})");

        let card: ClassInstanceRef<Card> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;")
            .await?;
        jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (x, y, width, height))
            .await
    }

    async fn service_repaints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::serviceRepaints({this:?})");

        let card: ClassInstanceRef<Card> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;")
            .await?;
        jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "serviceRepaints", "()V", ()).await
    }

    async fn layout(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.ShellComponent::layout({this:?})");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::{
            net::wie::CardCanvas,
            org::kwis::msp::lcdui::{Display, Graphics},
        },
        get_protos,
    };

    struct TestShell;

    impl TestShell {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TestShell",
                parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![
                    JavaFieldProto::new("paintCount", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("lastKey", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("showCount", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("hideCount", "I", FieldAccessFlags::PRIVATE),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn paint(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _: ClassInstanceRef<Graphics>) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "test/TestShell", "paintCount", "I").await?;
            jvm.put_field(&mut this, "test/TestShell", "paintCount", "I", count + 1).await
        }

        async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
            jvm.put_field(&mut this, "test/TestShell", "lastKey", "I", r#type * 1000 + key).await?;
            Ok(false)
        }

        async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
            let field = if show { "showCount" } else { "hideCount" };
            let count: i32 = jvm.get_field(&this, "test/TestShell", field, "I").await?;
            jvm.put_field(&mut this, "test/TestShell", field, "I", count + 1).await
        }
    }

    #[test]
    fn shell_component_paints_and_receives_keys_through_its_card() -> Result<()> {
        let jlet_proto = WieJavaClassProto {
            name: "test/ShellJlet",
            parent_class: Some("org/kwis/msp/lcdui/Jlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                get_protos().into(),
                Box::new([TestShell::as_proto(), jlet_proto]),
            ]),
            |jvm| async move {
                let _midlet = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await?;
                let jlet = jvm.instantiate_class("test/ShellJlet").await?;
                let _: () = jvm.invoke_special(&jlet, "org/kwis/msp/lcdui/Jlet", "<init>", "()V", ()).await?;
                let display: ClassInstanceRef<Display> = jvm
                    .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
                    .await?;
                let canvas: ClassInstanceRef<CardCanvas> = jvm
                    .get_field(&display, "org/kwis/msp/lcdui/Display", "cardCanvas", "Lnet/wie/CardCanvas;")
                    .await?;

                let shell = jvm.instantiate_class("test/TestShell").await?;
                let _: () = jvm.invoke_special(&shell, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

                // A shell created without bounds covers the whole display.
                for dimension in ["getWidth", "getHeight"] {
                    let expected: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", dimension, "()I", ()).await?;
                    let actual: i32 = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", dimension, "()I", ()).await?;
                    assert_eq!(actual, expected);
                }
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&shell, "org/kwis/msp/lwc/Component", "isShown", "()Z", ())
                        .await?
                );

                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "show", "()V", ()).await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(&shell, "org/kwis/msp/lwc/Component", "isShown", "()Z", ())
                        .await?
                );
                assert_eq!(jvm.get_field::<i32>(&shell, "test/TestShell", "showCount", "I").await?, 1);
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&display, "org/kwis/msp/lcdui/Display", "countCard", "()I", ())
                        .await?,
                    1
                );

                let _: () = jvm
                    .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "serviceRepaints", "()V", ())
                    .await?;
                let paint_count: i32 = jvm.get_field(&shell, "test/TestShell", "paintCount", "I").await?;
                assert!(paint_count > 0);
                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await?;
                let _: () = jvm
                    .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "serviceRepaints", "()V", ())
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&shell, "test/TestShell", "paintCount", "I").await?, paint_count + 1);

                let _: () = jvm.invoke_virtual(&canvas, "net/wie/CardCanvas", "keyPressed", "(I)V", (42,)).await?;
                assert_eq!(jvm.get_field::<i32>(&shell, "test/TestShell", "lastKey", "I").await?, 1042);

                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "hide", "()V", ()).await?;
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&shell, "org/kwis/msp/lwc/Component", "isShown", "()Z", ())
                        .await?
                );
                assert_eq!(jvm.get_field::<i32>(&shell, "test/TestShell", "hideCount", "I").await?, 1);
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&display, "org/kwis/msp/lcdui/Display", "countCard", "()I", ())
                        .await?,
                    0
                );

                Ok(())
            },
        )
    }
}
