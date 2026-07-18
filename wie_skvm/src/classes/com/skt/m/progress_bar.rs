use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
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
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxValue", "()I", Self::get_max_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getValue", "()I", Self::get_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxValue", "(I)V", Self::set_max_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setValue", "(I)V", Self::set_value, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("maxValue", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("value", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.ProgressBar::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "maxValue", "I", 100).await?;
        jvm.put_field(&mut this, "value", "I", 0).await
    }

    async fn get_max_value(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "maxValue", "I").await
    }

    async fn get_value(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "value", "I").await
    }

    async fn set_max_value(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        let current: i32 = jvm.get_field(&this, "value", "I").await?;
        jvm.put_field(&mut this, "maxValue", "I", value).await?;
        if current > value {
            jvm.put_field(&mut this, "value", "I", value).await?;
        }
        Ok(())
    }

    async fn set_value(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        let max_value: i32 = jvm.get_field(&this, "maxValue", "I").await?;
        jvm.put_field(&mut this, "value", "I", value.min(max_value)).await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;

    use crate::get_protos;

    use super::ProgressBar;

    #[test]
    fn test_value_and_max_value_enforce_documented_bounds() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "Loading").await?.into();
            let progress_bar: ClassInstanceRef<ProgressBar> = jvm.new_class("com/skt/m/ProgressBar", "(Ljava/lang/String;)V", (name,)).await?.into();

            assert_eq!(jvm.invoke_virtual::<_, i32>(&progress_bar, "getMaxValue", "()I", ()).await?, 100);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&progress_bar, "getValue", "()I", ()).await?, 0);

            let _: () = jvm.invoke_virtual(&progress_bar, "setValue", "(I)V", (125,)).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&progress_bar, "getValue", "()I", ()).await?, 100);

            let _: () = jvm.invoke_virtual(&progress_bar, "setMaxValue", "(I)V", (40,)).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&progress_bar, "getMaxValue", "()I", ()).await?, 40);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&progress_bar, "getValue", "()I", ()).await?, 40);
            Ok(())
        });

        assert!(result.is_ok());
    }
}
