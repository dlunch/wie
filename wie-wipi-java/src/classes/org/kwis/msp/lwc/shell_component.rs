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
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;)I",
                    Self::add_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("removeComponent", "(I)V", Self::remove_component_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setWorkComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::set_work_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hide", "()V", Self::hide, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_card, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_area, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("cd", "Lorg/kwis/msp/lcdui/Card;", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("cmpWork", "Lorg/kwis/msp/lwc/Component;", FieldAccessFlags::PROTECTED),
            ],
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

    async fn add_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::addComponent({this:?}, {component:?})");
        let index = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/ContainerComponent",
                "addComponent",
                "(Lorg/kwis/msp/lwc/Component;)I",
                (component.clone(),),
            )
            .await?;
        let work: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cmpWork", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if work.is_null() {
            jvm.put_field(
                &mut this,
                "org/kwis/msp/lwc/ShellComponent",
                "cmpWork",
                "Lorg/kwis/msp/lwc/Component;",
                component,
            )
            .await?;
        }
        Ok(index)
    }

    async fn remove_component_index(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::removeComponent({this:?}, {index})");
        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "removeComponent", "(I)V", (index,))
            .await?;
        let work: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cmpWork", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if work.is_null() {
            return Ok(());
        }
        let parent: ClassInstanceRef<Self> = jvm
            .get_field(&work, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        if parent.is_null() {
            jvm.put_field(
                &mut this,
                "org/kwis/msp/lwc/ShellComponent",
                "cmpWork",
                "Lorg/kwis/msp/lwc/Component;",
                ClassInstanceRef::<Component>::from(None),
            )
            .await?;
        }
        Ok(())
    }

    async fn set_work_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::setWorkComponent({this:?}, {component:?})");
        let old: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cmpWork", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if !component.is_null() {
            if !old.is_null() && old.equals(&**component)? {
                return Ok(());
            }
            let parent: ClassInstanceRef<Self> = jvm
                .get_field(
                    &component,
                    "org/kwis/msp/lwc/Component",
                    "parent",
                    "Lorg/kwis/msp/lwc/ContainerComponent;",
                )
                .await?;
            if !parent.is_null() {
                return Err(jvm.exception("java/lang/IllegalArgumentException", "child already has a parent").await);
            }
        }
        if !old.is_null() {
            let _: () = jvm
                .invoke_virtual(
                    &this,
                    "org/kwis/msp/lwc/ContainerComponent",
                    "removeComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    (old,),
                )
                .await?;
        }
        if component.is_null() {
            return Ok(());
        }
        let _: i32 = jvm
            .invoke_virtual(
                &this,
                "org/kwis/msp/lwc/ShellComponent",
                "addComponent",
                "(Lorg/kwis/msp/lwc/Component;)I",
                (component,),
            )
            .await?;
        Ok(())
    }

    async fn show(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::show({this:?})");

        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "validate", "()V", ()).await?;
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

    async fn layout(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::layout({this:?})");
        let work: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cmpWork", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if work.is_null() {
            return Ok(());
        }
        let width: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "h", "I").await?;
        jvm.invoke_virtual(&work, "org/kwis/msp/lwc/Component", "configure", "(IIIII)V", (0, 0, width, height, 3))
            .await
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, run_jvm_test_with_system};
    use wie_backend::{Event, KeyCode};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::net::wie::{KeyboardEventType, MIDPKeyCode};
    use wie_util::Result;

    use crate::{
        classes::{
            net::wie::{CardCanvas, WIPIKeyCode},
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
            assert_eq!(
                jvm.invoke_virtual::<_, bool>(&this, "org/kwis/msp/lwc/Component", "isShown", "()Z", ())
                    .await?,
                show
            );
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
        run_jvm_test_with_system(
            Box::new([
                wie_midp::get_protos().into(),
                get_protos().into(),
                Box::new([TestShell::as_proto(), jlet_proto]),
            ]),
            Box::new(TestPlatform::new().with_text_input("가나다")),
            |jvm, system| async move {
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

                let _: () = jvm
                    .invoke_virtual(&canvas, "net/wie/CardCanvas", "keyPressed", "(I)V", (MIDPKeyCode::KEY_STAR as i32,))
                    .await?;
                assert_eq!(
                    jvm.get_field::<i32>(&shell, "test/TestShell", "lastKey", "I").await?,
                    KeyboardEventType::KeyPressed as i32 * 1000 + WIPIKeyCode::STAR as i32
                );

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

                let display_width: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", ()).await?;
                let whole_display = jvm
                    .new_class("org/kwis/msp/lwc/ProxyCard", "(Lorg/kwis/msp/lwc/ContainerComponent;)V", (shell.clone(),))
                    .await?;
                let transparent = jvm
                    .new_class(
                        "org/kwis/msp/lwc/ProxyCard",
                        "(Lorg/kwis/msp/lwc/ContainerComponent;Z)V",
                        (shell.clone(), true),
                    )
                    .await?;
                let bounded = jvm
                    .new_class(
                        "org/kwis/msp/lwc/ProxyCard",
                        "(Lorg/kwis/msp/lwc/ContainerComponent;IIIIZ)V",
                        (shell.clone(), 1, 2, 30, 40, true),
                    )
                    .await?;
                for (card, expected_width) in [(whole_display, display_width), (transparent, display_width), (bounded, 30)] {
                    let width: i32 = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "getWidth", "()I", ()).await?;
                    assert_eq!(width, expected_width);
                }

                let shell_class = "org/kwis/msp/lwc/ShellComponent";
                let form_class = "org/kwis/msp/lwc/FormComponent";
                let text_class = "org/kwis/msp/lwc/TextComponent";
                let tracked_child = shell;
                let shell = jvm.new_class(shell_class, "()V", ()).await?;
                let form = jvm.new_class(form_class, "()V", ()).await?;
                let prefix = JavaLangString::from_rust_string(&jvm, "A").await?;
                let field = jvm
                    .new_class("org/kwis/msp/lwc/TextFieldComponent", "(Ljava/lang/String;I)V", (prefix, 0))
                    .await?;
                let textbox = jvm
                    .new_class(
                        "org/kwis/msp/lwc/TextBoxComponent",
                        "(Ljava/lang/String;II)V",
                        (ClassInstanceRef::<String>::from(None), 0, 30),
                    )
                    .await?;
                let _: () = jvm.invoke_virtual(&field, text_class, "setMaxLength", "(I)V", (3,)).await?;
                for child in [&field, &textbox] {
                    let _: i32 = jvm
                        .invoke_virtual(&form, form_class, "addComponent", "(Lorg/kwis/msp/lwc/Component;)I", (child.clone(),))
                        .await?;
                }
                let outer = jvm.new_class(form_class, "()V", ()).await?;
                let _: i32 = jvm
                    .invoke_virtual(&outer, form_class, "addComponent", "(Lorg/kwis/msp/lwc/Component;)I", (form.clone(),))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&shell, shell_class, "setWorkComponent", "(Lorg/kwis/msp/lwc/Component;)V", (outer,))
                    .await?;
                let _: () = jvm.invoke_virtual(&shell, shell_class, "show", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&shell, shell_class, "serviceRepaints", "()V", ()).await?;
                assert!(jvm.invoke_virtual::<_, i32>(&form, form_class, "getHeight", "()I", ()).await? > 0);
                for (method, key, widget, expected) in [
                    ("keyPressed", MIDPKeyCode::FIRE, &field, "A"),
                    ("keyRepeated", MIDPKeyCode::KEY_NUM1, &field, "A"),
                    ("keyPressed", MIDPKeyCode::KEY_NUM1, &field, "A가나"),
                    ("keyPressed", MIDPKeyCode::CLEAR, &field, "A가"),
                    ("keyPressed", MIDPKeyCode::DOWN, &textbox, ""),
                    ("keyPressed", MIDPKeyCode::KEY_STAR, &textbox, "가나다"),
                ] {
                    let _: () = jvm.invoke_virtual(&canvas, "net/wie/CardCanvas", method, "(I)V", (key as i32,)).await?;
                    let value: ClassInstanceRef<String> = jvm.invoke_virtual(widget, text_class, "getString", "()Ljava/lang/String;", ()).await?;
                    assert_eq!(JavaLangString::to_rust_string(&jvm, &value).await?, expected);
                }
                let _: i32 = jvm
                    .invoke_virtual(
                        &form,
                        form_class,
                        "addComponent",
                        "(Lorg/kwis/msp/lwc/Component;)I",
                        (tracked_child.clone(),),
                    )
                    .await?;
                let _: () = jvm.invoke_virtual(&shell, shell_class, "hide", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&shell, shell_class, "show", "()V", ()).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &form,
                        form_class,
                        "removeComponent",
                        "(Lorg/kwis/msp/lwc/Component;)V",
                        (tracked_child.clone(),),
                    )
                    .await?;
                assert_eq!(
                    (
                        jvm.get_field::<i32>(&tracked_child, "test/TestShell", "showCount", "I").await?,
                        jvm.get_field::<i32>(&tracked_child, "test/TestShell", "hideCount", "I").await?
                    ),
                    (3, 3)
                );
                let dialog_class = "org/kwis/msp/lwc/DialogComponent";
                let name = jvm
                    .new_class("org/kwis/msp/lwc/TextFieldComponent", "(Ljava/lang/String;I)V", (None, 0))
                    .await?;
                let dialog = jvm
                    .new_class(
                        dialog_class,
                        "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V",
                        (name.clone(), None, 2),
                    )
                    .await?;
                system.event_queue().push(Event::Keydown(KeyCode::NUM1));
                for (button, result) in [(KeyCode::LEFT_SOFT_KEY, 11), (KeyCode::RIGHT_SOFT_KEY, 12)] {
                    system.event_queue().push(Event::Keydown(button));
                    assert_eq!(jvm.invoke_virtual::<_, i32>(&dialog, dialog_class, "doModal", "()I", ()).await?, result);
                    assert!(!jvm.invoke_virtual::<_, bool>(&dialog, dialog_class, "isShown", "()Z", ()).await?);
                }
                let value: ClassInstanceRef<String> = jvm.invoke_virtual(&name, text_class, "getString", "()Ljava/lang/String;", ()).await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &value).await?, "가나다");
                let notice = jvm.new_class(dialog_class, "(I)V", (0,)).await?;
                let _: () = jvm.invoke_virtual(&notice, dialog_class, "setTimeout", "(I)V", (160,)).await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&notice, dialog_class, "doModal", "()I", ()).await?, 10);
                Ok(())
            },
        )
    }
}
