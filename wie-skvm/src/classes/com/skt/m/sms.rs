use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use super::{SMSListener, SMSMessage};

// class com.skt.m.SMS
#[java_class(name = "com/skt/m/SMS", public, final)]
pub struct SMS {
    #[field(private, static)]
    listener: ClassInstanceRef<SMSListener>,
}

#[java_methods]
impl SMS {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field(
            "com/skt/m/SMS",
            "listener",
            "Lcom/skt/m/SMSListener;",
            ClassInstanceRef::<SMSListener>::new(None),
        )
        .await
    }

    #[method(public, static)]
    async fn get(jvm: &Jvm, _context: &mut WieJvmContext, index: i32) -> JvmResult<ClassInstanceRef<SMSMessage>> {
        tracing::warn!("stub com.skt.m.SMS::get({index})");

        if index != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "index must be zero").await);
        }

        Ok(ClassInstanceRef::new(None))
    }

    #[method(name = "get", public, static)]
    async fn get_into_message(_jvm: &Jvm, _context: &mut WieJvmContext, index: i32, message: ClassInstanceRef<SMSMessage>) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.SMS::get({index}, {message:?})");

        Ok(false)
    }

    #[method(name = "getSMSListener", public, static)]
    async fn get_sms_listener(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<SMSListener>> {
        jvm.get_static_field("com/skt/m/SMS", "listener", "Lcom/skt/m/SMSListener;").await
    }

    #[method(public, static)]
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

    #[method(name = "setSMSListener", public, static)]
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
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult};
    use jvm_class_proto::{java_class, java_methods};
    use test_utils::run_jvm_test;

    use super::SMS;
    use crate::classes::com::skt::m::{SMSListener, SMSMessage};
    use wie_jvm_support::WieJvmContext;

    #[java_class(name = "test/SMSListener", implements("com/skt/m/SMSListener"), public)]
    struct TestSMSListener;

    #[java_methods]
    impl TestSMSListener {
        #[method(constructor, public)]
        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
            Ok(())
        }

        #[method(public)]
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

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
