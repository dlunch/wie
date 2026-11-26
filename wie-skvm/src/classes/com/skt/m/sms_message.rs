use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

const APPLICATION_DATA: i32 = 0;
const DOWNLOAD_NOTIFICATION: i32 = 1;
const SHORT_MESSAGE: i32 = 2;
const UNKNOWN: i32 = 3;

// class com.skt.m.SMSMessage
#[java_class(name = "com/skt/m/SMSMessage", public)]
pub struct SMSMessage {
    #[field(public, static, final)]
    APPLICATION_DATA: i32,
    #[field(public, static, final)]
    DOWNLOAD_NOTIFICATION: i32,
    #[field(public, static, final)]
    SHORT_MESSAGE: i32,
    #[field(public, static, final)]
    UNKNOWN: i32,
    #[field(private)]
    r#type: i32,
    #[field(private)]
    shortMessage: ClassInstanceRef<Array<i8>>,
    #[field(private)]
    sender: ClassInstanceRef<String>,
    #[field(private)]
    url: ClassInstanceRef<String>,
    #[field(private)]
    name: ClassInstanceRef<String>,
    #[field(private)]
    comment: ClassInstanceRef<String>,
    #[field(private)]
    cname: ClassInstanceRef<String>,
    #[field(private)]
    appData: ClassInstanceRef<Array<i8>>,
}

#[java_methods]
impl SMSMessage {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field("com/skt/m/SMSMessage", "APPLICATION_DATA", "I", APPLICATION_DATA)
            .await?;
        jvm.put_static_field("com/skt/m/SMSMessage", "DOWNLOAD_NOTIFICATION", "I", DOWNLOAD_NOTIFICATION)
            .await?;
        jvm.put_static_field("com/skt/m/SMSMessage", "SHORT_MESSAGE", "I", SHORT_MESSAGE).await?;
        jvm.put_static_field("com/skt/m/SMSMessage", "UNKNOWN", "I", UNKNOWN).await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.SMSMessage::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "com/skt/m/SMSMessage", "type", "I", UNKNOWN).await?;

        Ok(())
    }

    #[method(constructor, public)]
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

    #[method(constructor, public)]
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

    #[method(constructor, public)]
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

    #[method(public)]
    async fn get_app_data(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "appData", "[B").await
    }

    #[method(name = "getCName", public)]
    async fn get_cname(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "cname", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn get_comment(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "comment", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn get_name(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "name", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn get_sender(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "sender", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn get_service_option(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i8> {
        tracing::warn!("stub com.skt.m.SMSMessage::getServiceOption({this:?})");

        Ok(0)
    }

    #[method(public)]
    async fn get_short_message(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "shortMessage", "[B").await
    }

    #[method(public)]
    async fn get_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "type", "I").await
    }

    #[method(name = "getURL", public)]
    async fn get_url(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "com/skt/m/SMSMessage", "url", "Ljava/lang/String;").await
    }
}
