use alloc::format;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::Display;

// abstract class javax.microedition.midlet.MIDlet
#[java_class(name = "javax/microedition/midlet/MIDlet", public, abstract)]
pub struct MIDlet {
    #[field(private, static)]
    currentMIDlet: ClassInstanceRef<Self>,
    #[field(private)]
    display: ClassInstanceRef<Display>,
}

#[java_methods]
impl MIDlet {
    #[method(protected)]
    async fn start_app(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(protected)]
    async fn pause_app(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(protected)]
    async fn destroy_app(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, unconditional: bool) -> JvmResult<()>;

    #[method(constructor, protected)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.midlet.MIDlet::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_static_field(
            "javax/microedition/midlet/MIDlet",
            "currentMIDlet",
            "Ljavax/microedition/midlet/MIDlet;",
            this.clone(),
        )
        .await?;

        let display = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?;

        jvm.put_field(
            &mut this,
            "javax/microedition/midlet/MIDlet",
            "display",
            "Ljavax/microedition/lcdui/Display;",
            display,
        )
        .await?;

        Ok(())
    }

    #[method(public, final)]
    async fn check_permission(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, permission: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.midlet.MIDlet::checkPermission({this:?}, {permission:?})");

        Ok(1)
    }

    #[method(public, final)]
    async fn get_app_property(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        key: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.midlet.MIDlet::getAppProperty({this:?}, {key:?})");

        let key = JavaLangString::to_rust_string(jvm, &key).await?;
        let system_key = format!("wie.appProperty.{key}");
        let system_key = JavaLangString::from_rust_string(jvm, &system_key).await?;

        jvm.invoke_static("java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;", (system_key,))
            .await
    }

    #[method(public, final)]
    async fn notify_destroyed(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.midlet.MIDlet::notifyDestroyed({this:?})");

        Ok(())
    }

    #[method(public, final)]
    async fn notify_paused(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.midlet.MIDlet::notifyPaused({this:?})");

        Ok(())
    }

    #[method(public, final)]
    async fn platform_request(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        url: ClassInstanceRef<String>,
    ) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.midlet.MIDlet::platformRequest({this:?}, {url:?})");

        Ok(false)
    }

    #[method(public, final)]
    async fn resume_request(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.midlet.MIDlet::resumeRequest({this:?})");

        Ok(())
    }

    pub async fn display(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Display>> {
        jvm.get_field(this, "javax/microedition/midlet/MIDlet", "display", "Ljavax/microedition/lcdui/Display;")
            .await
    }
}
