use alloc::{vec, vec::Vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::InputMethodListener;

// class org.kwis.msp.lcdui.InputMethodHandler
pub struct InputMethodHandler;

impl InputMethodHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/InputMethodHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCurrentMode", "()I", Self::get_current_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setCurrentMode", "(I)Z", Self::set_current_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "notifyKeyInput",
                    "(II)Z",
                    Self::notify_key_input,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new("hideSymbolCard", "()V", Self::hide_symbol_card, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setInputMethodListener",
                    "(Lorg/kwis/msp/lcdui/InputMethodListener;)V",
                    Self::set_input_method_listener,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("currentMode", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("allowedModes", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("listener", "Lorg/kwis/msp/lcdui/InputMethodListener;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraint: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::<init>({this:?}, {constraint})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        // Mode indices follow the native supported-mode order: EN/L, EN/S, KO, N123.
        let (allowed_modes, current_mode) = match constraint {
            0 => (0b1111, 0),
            1 | 2 | 5 => (0b1000, 3),
            3 | 4 => (0b1011, 0),
            _ => return Err(jvm.exception("java/lang/IllegalArgumentException", "Unsupported input constraint").await),
        };
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/InputMethodHandler", "allowedModes", "I", allowed_modes)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/InputMethodHandler", "currentMode", "I", current_mode)
            .await?;

        Ok(())
    }

    async fn get_current_mode(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::getCurrentMode({this:?})");
        jvm.get_field(&this, "org/kwis/msp/lcdui/InputMethodHandler", "currentMode", "I").await
    }

    async fn set_current_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, mode: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::setCurrentMode({this:?}, {mode})");

        if !(0..4).contains(&mode) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Unsupported input mode").await);
        }
        let allowed_modes: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/InputMethodHandler", "allowedModes", "I").await?;
        if allowed_modes & (1 << mode) == 0 {
            return Ok(false);
        }
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/InputMethodHandler", "currentMode", "I", mode)
            .await?;

        Ok(true)
    }

    async fn notify_key_input(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32, kind: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::notifyKeyInput({this:?}, {key_code}, {kind})");

        // The API has no begin-edit call, so any key press reaching the input method starts typing.
        if kind != 1 {
            return Ok(false);
        }
        let listener: ClassInstanceRef<InputMethodListener> = jvm
            .get_field(
                &this,
                "org/kwis/msp/lcdui/InputMethodHandler",
                "listener",
                "Lorg/kwis/msp/lcdui/InputMethodListener;",
            )
            .await?;
        if listener.is_null() {
            return Ok(false);
        }
        let text: Vec<u16> = context
            .system()
            .platform()
            .text_input("")
            .await
            .unwrap_or_default()
            .encode_utf16()
            .collect();
        if text.is_empty() {
            return Ok(false);
        }
        let length = text.len();
        let mut array = jvm.instantiate_array("C", length).await?;
        jvm.store_array(&mut array, 0, text).await?;
        // The listener owns the text being edited, so the host input starts empty and -1 inserts its result at the cursor.
        let _: () = jvm
            .invoke_virtual(
                &listener,
                "org/kwis/msp/lcdui/InputMethodListener",
                "notifyTextChanged",
                "([CII)V",
                (array, length as i32, -1),
            )
            .await?;

        Ok(true)
    }

    async fn hide_symbol_card(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::hideSymbolCard({this:?})");

        Ok(())
    }

    async fn set_input_method_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<InputMethodListener>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::setInputMethodListener({this:?}, {listener:?})");

        jvm.put_field(
            &mut this,
            "org/kwis/msp/lcdui/InputMethodHandler",
            "listener",
            "Lorg/kwis/msp/lcdui/InputMethodListener;",
            listener,
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec, vec::Vec};

    use jvm::{Array, ClassInstanceRef, JavaChar, JavaError, Jvm, Result as JvmResult};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::get_protos;

    struct RecordingListener;

    impl RecordingListener {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/RecordingListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["org/kwis/msp/lcdui/InputMethodListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("notifyTextChanged", "([CII)V", Self::notify_text_changed, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("text", "[C", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("length", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("mode", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn notify_text_changed(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            text: ClassInstanceRef<Array<JavaChar>>,
            length: i32,
            mode: i32,
        ) -> JvmResult<()> {
            jvm.put_field(&mut this, "test/RecordingListener", "text", "[C", text).await?;
            jvm.put_field(&mut this, "test/RecordingListener", "length", "I", length).await?;
            jvm.put_field(&mut this, "test/RecordingListener", "mode", "I", mode).await
        }
    }

    #[test]
    fn key_press_inserts_host_text_through_the_listener() -> Result<()> {
        run_jvm_test_with_system(
            Box::new([get_protos().into(), Box::new([RecordingListener::as_proto()])]),
            Box::new(TestPlatform::new().with_text_input("안녕")),
            |jvm, _| async move {
                let class = "org/kwis/msp/lcdui/InputMethodHandler";
                let handler = jvm.new_class(class, "(I)V", (0,)).await?;
                let listener = jvm.new_class("test/RecordingListener", "()V", ()).await?;
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&handler, class, "notifyKeyInput", "(II)Z", (i32::from(b'1'), 1))
                        .await?
                );
                let _: () = jvm
                    .invoke_virtual(
                        &handler,
                        class,
                        "setInputMethodListener",
                        "(Lorg/kwis/msp/lcdui/InputMethodListener;)V",
                        (listener.clone(),),
                    )
                    .await?;
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&handler, class, "notifyKeyInput", "(II)Z", (i32::from(b'1'), 2))
                        .await?
                );
                assert!(jvm.invoke_virtual::<_, bool>(&handler, class, "notifyKeyInput", "(II)Z", (-5, 1)).await?);

                let text: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&listener, "test/RecordingListener", "text", "[C").await?;
                let length: i32 = jvm.get_field(&listener, "test/RecordingListener", "length", "I").await?;
                assert_eq!(
                    jvm.load_array::<JavaChar>(&text, 0, length as usize).await?,
                    "안녕".encode_utf16().collect::<Vec<_>>()
                );
                assert_eq!(jvm.get_field::<i32>(&listener, "test/RecordingListener", "mode", "I").await?, -1);
                Ok(())
            },
        )
    }

    #[test]
    fn input_modes_respect_constructor_constraints() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let class = "org/kwis/msp/lcdui/InputMethodHandler";
            for (constraint, initial, mode, accepted) in [
                (0, 0, 2, true),
                (1, 3, 0, false),
                (2, 3, 0, false),
                (5, 3, 0, false),
                (3, 0, 2, false),
                (4, 0, 2, false),
                (3, 0, 1, true),
                (4, 0, 3, true),
            ] {
                let handler = jvm.new_class(class, "(I)V", (constraint,)).await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&handler, class, "getCurrentMode", "()I", ()).await?, initial);
                assert_eq!(
                    jvm.invoke_virtual::<_, bool>(&handler, class, "setCurrentMode", "(I)Z", (mode,)).await?,
                    accepted
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&handler, class, "getCurrentMode", "()I", ()).await?,
                    if accepted { mode } else { initial }
                );
            }
            let handler = jvm.new_class(class, "(I)V", (0,)).await?;
            for mode in [-1, 4] {
                let result: JvmResult<bool> = jvm.invoke_virtual(&handler, class, "setCurrentMode", "(I)Z", (mode,)).await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("unsupported input mode was accepted");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                assert_eq!(jvm.invoke_virtual::<_, i32>(&handler, class, "getCurrentMode", "()I", ()).await?, 0);
            }
            let result = jvm.new_class(class, "(I)V", (6,)).await;
            let Err(JavaError::JavaException(exception)) = result else {
                panic!("unsupported input constraint was accepted");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
            Ok(())
        })
    }
}
