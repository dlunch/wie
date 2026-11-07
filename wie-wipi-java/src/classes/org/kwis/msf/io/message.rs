use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.io.Message
pub struct Message;

impl Message {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/Message",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;[B)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getData", "()[B", Self::get_data, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new("data", "[B", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        address: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msf.io.Message::<init>({this:?}, {address:?}, {data:?})");
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "org/kwis/msf/io/Message", "data", "[B", data).await?;
        Ok(())
    }

    async fn get_data(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("org.kwis.msf.io.Message::getData({this:?})");
        jvm.get_field(&this, "org/kwis/msf/io/Message", "data", "[B").await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn message_retains_the_supplied_buffer() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let address = JavaLangString::from_rust_string(&jvm, "127.0.0.1:1234").await?;
            let mut data = jvm.instantiate_array("B", 2).await?;
            jvm.store_array(&mut data, 0, [12i8, 34]).await?;
            let message = jvm
                .new_class("org/kwis/msf/io/Message", "(Ljava/lang/String;[B)V", (address, data.clone()))
                .await?;
            let buffer: ClassInstanceRef<Array<i8>> = jvm.invoke_virtual(&message, "org/kwis/msf/io/Message", "getData", "()[B", ()).await?;
            assert_eq!(jvm.load_array::<i8>(&buffer, 0, 2).await?, [12i8, 34]);

            jvm.store_array(&mut data, 1, [56i8]).await?;
            assert_eq!(jvm.load_array::<i8>(&buffer, 0, 2).await?, [12i8, 56]);
            Ok(())
        })
    }
}
