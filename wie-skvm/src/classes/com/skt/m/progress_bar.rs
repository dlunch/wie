use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class com.skt.m.ProgressBar
#[java_class(name = "com/skt/m/ProgressBar", public, final)]
pub struct ProgressBar {
    #[field(private)]
    maxValue: i32,
    #[field(private)]
    value: i32,
}

#[java_methods]
impl ProgressBar {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.ProgressBar::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "com/skt/m/ProgressBar", "maxValue", "I", 100).await?;
        jvm.put_field(&mut this, "com/skt/m/ProgressBar", "value", "I", 0).await
    }

    #[method(public)]
    async fn get_max_value(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "com/skt/m/ProgressBar", "maxValue", "I").await
    }

    #[method(public)]
    async fn get_value(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "com/skt/m/ProgressBar", "value", "I").await
    }

    #[method(public)]
    async fn set_max_value(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        let current: i32 = jvm.get_field(&this, "com/skt/m/ProgressBar", "value", "I").await?;
        jvm.put_field(&mut this, "com/skt/m/ProgressBar", "maxValue", "I", value).await?;
        if current > value {
            jvm.put_field(&mut this, "com/skt/m/ProgressBar", "value", "I", value).await?;
        }
        Ok(())
    }

    #[method(public)]
    async fn set_value(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        let max_value: i32 = jvm.get_field(&this, "com/skt/m/ProgressBar", "maxValue", "I").await?;
        jvm.put_field(&mut this, "com/skt/m/ProgressBar", "value", "I", value.min(max_value))
            .await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;

    use crate::get_protos;

    use super::ProgressBar;

    #[test]
    fn test_value_and_max_value_enforce_documented_bounds() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "Loading").await?.into();
            let progress_bar: ClassInstanceRef<ProgressBar> = jvm.new_class("com/skt/m/ProgressBar", "(Ljava/lang/String;)V", (name,)).await?.into();

            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&progress_bar, "com/skt/m/ProgressBar", "getMaxValue", "()I", ())
                    .await?,
                100
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&progress_bar, "com/skt/m/ProgressBar", "getValue", "()I", ())
                    .await?,
                0
            );

            let _: () = jvm
                .invoke_virtual(&progress_bar, "com/skt/m/ProgressBar", "setValue", "(I)V", (125,))
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&progress_bar, "com/skt/m/ProgressBar", "getValue", "()I", ())
                    .await?,
                100
            );

            let _: () = jvm
                .invoke_virtual(&progress_bar, "com/skt/m/ProgressBar", "setMaxValue", "(I)V", (40,))
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&progress_bar, "com/skt/m/ProgressBar", "getMaxValue", "()I", ())
                    .await?,
                40
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&progress_bar, "com/skt/m/ProgressBar", "getValue", "()I", ())
                    .await?,
                40
            );
            Ok(())
        });

        assert!(result.is_ok());
    }
}
