use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const APPLICATION_DATA: i32 = 0;
const DOWNLOAD_NOTIFICATION: i32 = 1;
const SHORT_MESSAGE: i32 = 2;
const UNKNOWN: i32 = 3;

// class com.skt.m.SMSMessage
pub struct SMSMessage;

impl SMSMessage {
    pub fn as_proto() -> WieJavaClassProto {
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
                JavaFieldProto::new(
                    "APPLICATION_DATA",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DOWNLOAD_NOTIFICATION",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SHORT_MESSAGE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "UNKNOWN",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
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
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "type", "I", UNKNOWN).await?;

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
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "type", "I", SHORT_MESSAGE).await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "shortMessage", "[B", data).await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "sender", "Ljava/lang/String;", sender)
            .await?;

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
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "type", "I", APPLICATION_DATA).await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "cname", "Ljava/lang/String;", cname)
            .await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "appData", "[B", data).await?;

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
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "type", "I", DOWNLOAD_NOTIFICATION)
            .await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "url", "Ljava/lang/String;", url).await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "name", "Ljava/lang/String;", name)
            .await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "comment", "Ljava/lang/String;", comment)
            .await?;

        Ok(())
    }

    async fn get_app_data(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "appData", "[B").await
    }

    async fn get_cname(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "cname", "Ljava/lang/String;").await
    }

    async fn get_comment(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "comment", "Ljava/lang/String;").await
    }

    async fn get_name(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "name", "Ljava/lang/String;").await
    }

    async fn get_sender(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "sender", "Ljava/lang/String;").await
    }

    async fn get_service_option(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i8> {
        tracing::warn!("stub com.skt.m.SMSMessage::getServiceOption({this:?})");

        Ok(0)
    }

    async fn get_short_message(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "shortMessage", "[B").await
    }

    async fn get_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "type", "I").await
    }

    async fn get_url(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "url", "Ljava/lang/String;").await
    }
}
