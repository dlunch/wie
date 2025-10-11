use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.skt.m.ProgressBar
pub struct ProgressBar;

impl ProgressBar {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/ProgressBar",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("setMaxValue", "(I)V", Self::set_max_value, Default::default()),
                JavaMethodProto::new("setValue", "(I)V", Self::set_value, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.ProgressBar::<init>({:?}, {:?})", &this, name);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_max_value(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.ProgressBar::setMaxValue({:?}, {:?})", &this, value);

        Ok(())
    }

    async fn set_value(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.ProgressBar::setValue({:?}, {:?})", &this, value);

        Ok(())
    }
}
