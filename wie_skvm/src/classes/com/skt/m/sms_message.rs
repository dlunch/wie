use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const APPLICATION_DATA: i32 = 0;
const DOWNLOAD_NOTIFICATION: i32 = 1;
const SHORT_MESSAGE: i32 = 2;
const UNKNOWN: i32 = 3;

// class com.skt.m.SMSMessage
pub struct SMSMessage;

impl SMSMessage {
    pub fn as_proto() -> WieJavaClassProto {
        let public_static_final = FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL;

        WieJavaClassProto {
            name: "com/skt/m/SMSMessage",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "([BLjava/lang/String;)V", Self::init_short_message, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;[B)V",
                    Self::init_application_data,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                    Self::init_download_notification,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getAppData", "()[B", Self::get_app_data, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCName", "()Ljava/lang/String;", Self::get_cname, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getComment", "()Ljava/lang/String;", Self::get_comment, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getName", "()Ljava/lang/String;", Self::get_name, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSender", "()Ljava/lang/String;", Self::get_sender, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getServiceOption", "()B", Self::get_service_option, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getShortMessage", "()[B", Self::get_short_message, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getType", "()I", Self::get_type, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getURL", "()Ljava/lang/String;", Self::get_url, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("APPLICATION_DATA", "I", public_static_final),
                JavaFieldProto::new("DOWNLOAD_NOTIFICATION", "I", public_static_final),
                JavaFieldProto::new("SHORT_MESSAGE", "I", public_static_final),
                JavaFieldProto::new("UNKNOWN", "I", public_static_final),
                JavaFieldProto::new("type", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("shortMessage", "[B", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("sender", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("url", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("name", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("comment", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("cname", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("appData", "[B", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field("com/skt/m/SMSMessage", "APPLICATION_DATA", "I", APPLICATION_DATA)
            .await?;
        jvm.put_static_field("com/skt/m/SMSMessage", "DOWNLOAD_NOTIFICATION", "I", DOWNLOAD_NOTIFICATION)
            .await?;
        jvm.put_static_field("com/skt/m/SMSMessage", "SHORT_MESSAGE", "I", SHORT_MESSAGE).await?;
        jvm.put_static_field("com/skt/m/SMSMessage", "UNKNOWN", "I", UNKNOWN).await?;

        Ok(())
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.SMSMessage::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "type", "I", UNKNOWN).await?;

        Ok(())
    }

    async fn init_short_message(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        sender: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.SMSMessage::<init>({this:?}, {data:?}, {sender:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "type", "I", SHORT_MESSAGE).await?;
        jvm.put_field(&mut this, "shortMessage", "[B", data).await?;
        jvm.put_field(&mut this, "sender", "Ljava/lang/String;", sender).await?;

        Ok(())
    }

    async fn init_application_data(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        cname: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.SMSMessage::<init>({this:?}, {cname:?}, {data:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "type", "I", APPLICATION_DATA).await?;
        jvm.put_field(&mut this, "cname", "Ljava/lang/String;", cname).await?;
        jvm.put_field(&mut this, "appData", "[B", data).await?;

        Ok(())
    }

    async fn init_download_notification(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        url: ClassInstanceRef<String>,
        name: ClassInstanceRef<String>,
        comment: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.SMSMessage::<init>({this:?}, {url:?}, {name:?}, {comment:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "type", "I", DOWNLOAD_NOTIFICATION).await?;
        jvm.put_field(&mut this, "url", "Ljava/lang/String;", url).await?;
        jvm.put_field(&mut this, "name", "Ljava/lang/String;", name).await?;
        jvm.put_field(&mut this, "comment", "Ljava/lang/String;", comment).await?;

        Ok(())
    }

    async fn get_app_data(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        jvm.get_field(&this, "appData", "[B").await
    }

    async fn get_cname(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "cname", "Ljava/lang/String;").await
    }

    async fn get_comment(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "comment", "Ljava/lang/String;").await
    }

    async fn get_name(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "name", "Ljava/lang/String;").await
    }

    async fn get_sender(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "sender", "Ljava/lang/String;").await
    }

    async fn get_service_option(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i8> {
        tracing::warn!("stub com.skt.m.SMSMessage::getServiceOption({this:?})");

        Ok(0)
    }

    async fn get_short_message(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        jvm.get_field(&this, "shortMessage", "[B").await
    }

    async fn get_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "type", "I").await
    }

    async fn get_url(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "url", "Ljava/lang/String;").await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{Array, ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;

    use super::SMSMessage;

    #[test]
    fn constructors_preserve_their_message_payloads_and_types() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), Box::new([SMSMessage::as_proto()])]),
            |jvm| async move {
                let unknown: i32 = jvm.get_static_field("com/skt/m/SMSMessage", "UNKNOWN", "I").await?;
                let default_message: ClassInstanceRef<SMSMessage> = jvm.new_class("com/skt/m/SMSMessage", "()V", ()).await?.into();
                let default_type: i32 = jvm.invoke_virtual(&default_message, "getType", "()I", ()).await?;
                assert_eq!(default_type, unknown);

                let mut short_data = jvm.instantiate_array("B", 3).await?;
                jvm.store_array(&mut short_data, 0, [1i8, 2, 3]).await?;
                let sender = JavaLangString::from_rust_string(&jvm, "01012345678").await?;
                let short_message: ClassInstanceRef<SMSMessage> = jvm
                    .new_class("com/skt/m/SMSMessage", "([BLjava/lang/String;)V", (short_data.clone(), sender.clone()))
                    .await?
                    .into();
                let short_type: i32 = jvm.invoke_virtual(&short_message, "getType", "()I", ()).await?;
                let expected_short_type: i32 = jvm.get_static_field("com/skt/m/SMSMessage", "SHORT_MESSAGE", "I").await?;
                let returned_short_data: ClassInstanceRef<Array<i8>> = jvm.invoke_virtual(&short_message, "getShortMessage", "()[B", ()).await?;
                let returned_sender: ClassInstanceRef<String> = jvm.invoke_virtual(&short_message, "getSender", "()Ljava/lang/String;", ()).await?;
                assert_eq!(short_type, expected_short_type);
                assert_eq!(
                    returned_short_data.instance.as_ref().map(|instance| instance.identity()),
                    Some(short_data.identity())
                );
                assert_eq!(jvm.load_array::<i8>(&returned_short_data, 0, 3).await?, [1i8, 2, 3]);
                assert_eq!(JavaLangString::to_rust_string(&jvm, &returned_sender).await?, "01012345678");

                let cname = JavaLangString::from_rust_string(&jvm, "weather").await?;
                let mut app_data = jvm.instantiate_array("B", 2).await?;
                jvm.store_array(&mut app_data, 0, [10i8, 20]).await?;
                let app_message: ClassInstanceRef<SMSMessage> = jvm
                    .new_class("com/skt/m/SMSMessage", "(Ljava/lang/String;[B)V", (cname.clone(), app_data.clone()))
                    .await?
                    .into();
                let app_type: i32 = jvm.invoke_virtual(&app_message, "getType", "()I", ()).await?;
                let expected_app_type: i32 = jvm.get_static_field("com/skt/m/SMSMessage", "APPLICATION_DATA", "I").await?;
                let returned_cname: ClassInstanceRef<String> = jvm.invoke_virtual(&app_message, "getCName", "()Ljava/lang/String;", ()).await?;
                let returned_app_data: ClassInstanceRef<Array<i8>> = jvm.invoke_virtual(&app_message, "getAppData", "()[B", ()).await?;
                assert_eq!(app_type, expected_app_type);
                assert_eq!(JavaLangString::to_rust_string(&jvm, &returned_cname).await?, "weather");
                assert_eq!(jvm.load_array::<i8>(&returned_app_data, 0, 2).await?, [10i8, 20]);

                let url = JavaLangString::from_rust_string(&jvm, "https://example.invalid/app").await?;
                let name = JavaLangString::from_rust_string(&jvm, "Example").await?;
                let comment = JavaLangString::from_rust_string(&jvm, "Install").await?;
                let download_message: ClassInstanceRef<SMSMessage> = jvm
                    .new_class(
                        "com/skt/m/SMSMessage",
                        "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                        (url.clone(), name.clone(), comment.clone()),
                    )
                    .await?
                    .into();
                let download_type: i32 = jvm.invoke_virtual(&download_message, "getType", "()I", ()).await?;
                let expected_download_type: i32 = jvm.get_static_field("com/skt/m/SMSMessage", "DOWNLOAD_NOTIFICATION", "I").await?;
                let returned_url: ClassInstanceRef<String> = jvm.invoke_virtual(&download_message, "getURL", "()Ljava/lang/String;", ()).await?;
                let returned_name: ClassInstanceRef<String> = jvm.invoke_virtual(&download_message, "getName", "()Ljava/lang/String;", ()).await?;
                let returned_comment: ClassInstanceRef<String> =
                    jvm.invoke_virtual(&download_message, "getComment", "()Ljava/lang/String;", ()).await?;
                assert_eq!(download_type, expected_download_type);
                assert_eq!(JavaLangString::to_rust_string(&jvm, &returned_url).await?, "https://example.invalid/app");
                assert_eq!(JavaLangString::to_rust_string(&jvm, &returned_name).await?, "Example");
                assert_eq!(JavaLangString::to_rust_string(&jvm, &returned_comment).await?, "Install");

                let service_option: i8 = jvm.invoke_virtual(&download_message, "getServiceOption", "()B", ()).await?;
                assert_eq!(service_option, 0);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
