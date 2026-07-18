use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::{SMSListener, SMSMessage};

// class com.skt.m.SMS
pub struct SMS;

impl SMS {
    pub fn as_proto() -> WieJavaClassProto {
        let public_static = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;

        WieJavaClassProto {
            name: "com/skt/m/SMS",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("get", "(I)Lcom/skt/m/SMSMessage;", Self::get, public_static),
                JavaMethodProto::new("get", "(ILcom/skt/m/SMSMessage;)Z", Self::get_into_message, public_static),
                JavaMethodProto::new("getSMSListener", "()Lcom/skt/m/SMSListener;", Self::get_sms_listener, public_static),
                JavaMethodProto::new("send", "(Ljava/lang/String;Lcom/skt/m/SMSMessage;)Z", Self::send, public_static),
                JavaMethodProto::new("setSMSListener", "(Lcom/skt/m/SMSListener;)V", Self::set_sms_listener, public_static),
            ],
            fields: vec![JavaFieldProto::new(
                "listener",
                "Lcom/skt/m/SMSListener;",
                FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
            )],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field(
            "com/skt/m/SMS",
            "listener",
            "Lcom/skt/m/SMSListener;",
            ClassInstanceRef::<SMSListener>::new(None),
        )
        .await
    }

    async fn get(jvm: &Jvm, _context: &mut WieJvmContext, index: i32) -> JvmResult<ClassInstanceRef<SMSMessage>> {
        tracing::warn!("stub com.skt.m.SMS::get({index})");

        if index != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "index must be zero").await);
        }

        Ok(ClassInstanceRef::new(None))
    }

    async fn get_into_message(_jvm: &Jvm, _context: &mut WieJvmContext, index: i32, message: ClassInstanceRef<SMSMessage>) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.SMS::get({index}, {message:?})");

        Ok(false)
    }

    async fn get_sms_listener(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<SMSListener>> {
        jvm.get_static_field("com/skt/m/SMS", "listener", "Lcom/skt/m/SMSListener;").await
    }

    async fn send(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        receiver: ClassInstanceRef<String>,
        message: ClassInstanceRef<SMSMessage>,
    ) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.SMS::send({receiver:?}, {message:?})");

        if receiver.is_null() || message.is_null() {
            return Err(jvm
                .exception("java/lang/NullPointerException", "receiver and message must not be null")
                .await);
        }

        Ok(false)
    }

    async fn set_sms_listener(jvm: &Jvm, _context: &mut WieJvmContext, listener: ClassInstanceRef<SMSListener>) -> JvmResult<()> {
        if listener.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "listener must not be null").await);
        }

        jvm.put_static_field("com/skt/m/SMS", "listener", "Lcom/skt/m/SMSListener;", listener)
            .await
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec};

    use java_class_proto::JavaMethodProto;
    use java_constants::{ClassAccessFlags, MethodAccessFlags};
    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;

    use super::SMS;
    use crate::classes::com::skt::m::{SMSListener, SMSMessage};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

    struct TestSMSListener;

    impl TestSMSListener {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/SMSListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["com/skt/m/SMSListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("onMessage", "(Lcom/skt/m/SMSMessage;)V", Self::on_message, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
            Ok(())
        }

        async fn on_message(
            _jvm: &Jvm,
            _context: &mut WieJvmContext,
            _this: ClassInstanceRef<Self>,
            _message: ClassInstanceRef<SMSMessage>,
        ) -> JvmResult<()> {
            Ok(())
        }
    }

    #[test]
    fn listener_round_trips_and_network_operations_fail_deterministically() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                Box::new([
                    SMS::as_proto(),
                    SMSListener::as_proto(),
                    SMSMessage::as_proto(),
                    TestSMSListener::as_proto(),
                ]),
            ]),
            |jvm| async move {
                let listener: ClassInstanceRef<SMSListener> = jvm.new_class("test/SMSListener", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_static("com/skt/m/SMS", "setSMSListener", "(Lcom/skt/m/SMSListener;)V", (listener.clone(),))
                    .await?;
                let returned: ClassInstanceRef<SMSListener> = jvm
                    .invoke_static("com/skt/m/SMS", "getSMSListener", "()Lcom/skt/m/SMSListener;", ())
                    .await?;
                assert_eq!(
                    returned.instance.as_ref().map(|instance| instance.identity()),
                    listener.instance.as_ref().map(|instance| instance.identity())
                );

                let message: ClassInstanceRef<SMSMessage> = jvm.new_class("com/skt/m/SMSMessage", "()V", ()).await?.into();
                let receiver: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "01098765432").await?.into();
                let sent: bool = jvm
                    .invoke_static(
                        "com/skt/m/SMS",
                        "send",
                        "(Ljava/lang/String;Lcom/skt/m/SMSMessage;)Z",
                        (receiver, message.clone()),
                    )
                    .await?;
                let filled: bool = jvm
                    .invoke_static("com/skt/m/SMS", "get", "(ILcom/skt/m/SMSMessage;)Z", (0, message))
                    .await?;
                let received: ClassInstanceRef<SMSMessage> = jvm.invoke_static("com/skt/m/SMS", "get", "(I)Lcom/skt/m/SMSMessage;", (0,)).await?;
                assert!(!sent);
                assert!(!filled);
                assert!(received.is_null());

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn documented_input_errors_raise_java_exceptions_and_preserve_the_listener() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                Box::new([
                    SMS::as_proto(),
                    SMSListener::as_proto(),
                    SMSMessage::as_proto(),
                    TestSMSListener::as_proto(),
                ]),
            ]),
            |jvm| async move {
                let listener: ClassInstanceRef<SMSListener> = jvm.new_class("test/SMSListener", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_static("com/skt/m/SMS", "setSMSListener", "(Lcom/skt/m/SMSListener;)V", (listener.clone(),))
                    .await?;

                let null_listener = ClassInstanceRef::<SMSListener>::new(None);
                let listener_result: JvmResult<()> = jvm
                    .invoke_static("com/skt/m/SMS", "setSMSListener", "(Lcom/skt/m/SMSListener;)V", (null_listener,))
                    .await;
                let Err(JavaError::JavaException(exception)) = listener_result else {
                    panic!("SMS.setSMSListener accepted null");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                let returned: ClassInstanceRef<SMSListener> = jvm
                    .invoke_static("com/skt/m/SMS", "getSMSListener", "()Lcom/skt/m/SMSListener;", ())
                    .await?;
                assert_eq!(
                    returned.instance.as_ref().map(|instance| instance.identity()),
                    listener.instance.as_ref().map(|instance| instance.identity())
                );

                let invalid_get: JvmResult<ClassInstanceRef<SMSMessage>> =
                    jvm.invoke_static("com/skt/m/SMS", "get", "(I)Lcom/skt/m/SMSMessage;", (1,)).await;
                let Err(JavaError::JavaException(exception)) = invalid_get else {
                    panic!("SMS.get accepted an index other than zero");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let message: ClassInstanceRef<SMSMessage> = jvm.new_class("com/skt/m/SMSMessage", "()V", ()).await?.into();
                let receiver: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "01098765432").await?.into();
                let null_receiver = ClassInstanceRef::<String>::new(None);
                let send_result: JvmResult<bool> = jvm
                    .invoke_static(
                        "com/skt/m/SMS",
                        "send",
                        "(Ljava/lang/String;Lcom/skt/m/SMSMessage;)Z",
                        (null_receiver, message),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = send_result else {
                    panic!("SMS.send accepted a null receiver");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                let null_message = ClassInstanceRef::<SMSMessage>::new(None);
                let send_result: JvmResult<bool> = jvm
                    .invoke_static(
                        "com/skt/m/SMS",
                        "send",
                        "(Ljava/lang/String;Lcom/skt/m/SMSMessage;)Z",
                        (receiver, null_message),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = send_result else {
                    panic!("SMS.send accepted a null message");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
