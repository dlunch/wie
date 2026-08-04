use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

pub struct JletWrapper;

impl JletWrapper {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/JletWrapper",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("startApp", "([Ljava/lang/String;)V", Self::start_app, Default::default()),
                JavaMethodProto::new("pauseApp", "()V", Self::pause_app, Default::default()),
                JavaMethodProto::new("resumeApp", "()V", Self::resume_app, Default::default()),
                JavaMethodProto::new("destroyApp", "(Z)V", Self::destroy_app, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn start_app(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, arguments: ClassInstanceRef<Array<String>>) -> JvmResult<()> {
        jvm.invoke_virtual(&this, "startApp", "([Ljava/lang/String;)V", (arguments,)).await
    }

    async fn pause_app(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.invoke_virtual(&this, "pauseApp", "()V", ()).await
    }

    async fn resume_app(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.invoke_virtual(&this, "resumeApp", "()V", ()).await
    }

    async fn destroy_app(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, unconditional: bool) -> JvmResult<()> {
        jvm.invoke_virtual(&this, "destroyApp", "(Z)V", (unconditional,)).await
    }
}
