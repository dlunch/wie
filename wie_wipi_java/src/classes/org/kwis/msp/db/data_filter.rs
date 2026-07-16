use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.db.DataFilter
pub struct DataFilter;

impl DataFilter {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataFilter",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract("filter", "([B)Z", MethodAccessFlags::ABSTRACT)],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec};

    use java_class_proto::JavaMethodProto;
    use java_constants::{ClassAccessFlags, MethodAccessFlags};
    use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use super::DataFilter;

    struct FilterFixture;

    impl FilterFixture {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/FilterFixture",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["org/kwis/msp/db/DataFilter"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                    JavaMethodProto::new("filter", "([B)Z", Self::filter, Default::default()),
                ],
                fields: vec![],
                access_flags: Default::default(),
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn filter(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, data: ClassInstanceRef<Array<i8>>) -> JvmResult<bool> {
            let length = jvm.array_length(&data).await?;
            let sum: i32 = jvm.load_array::<i8>(&data, 0, length).await?.into_iter().map(i32::from).sum();

            Ok(sum > 0)
        }
    }

    #[test]
    fn test_interface_contract() {
        let proto = DataFilter::as_proto();
        assert_eq!(proto.name, "org/kwis/msp/db/DataFilter");
        assert!(proto.parent_class.is_none());
        assert!(proto.access_flags.contains(ClassAccessFlags::INTERFACE));
        assert_eq!(proto.methods.len(), 1);
        assert_eq!(proto.methods[0].name, "filter");
        assert_eq!(proto.methods[0].descriptor, "([B)Z");
        assert!(proto.methods[0].access_flags.contains(MethodAccessFlags::ABSTRACT));
        assert!(!proto.methods[0].access_flags.contains(MethodAccessFlags::STATIC));
    }

    #[test]
    fn test_callback_dispatches_byte_array() -> Result<()> {
        run_jvm_test(Box::new([[DataFilter::as_proto(), FilterFixture::as_proto()].into()]), |jvm| async move {
            let filter: ClassInstanceRef<FilterFixture> = jvm.new_class("test/FilterFixture", "()V", ()).await?.into();
            let mut data = jvm.instantiate_array("B", 3).await?;
            jvm.store_array(&mut data, 0, [-1i8, 2, 3]).await?;

            let result: bool = jvm.invoke_virtual(&filter, "filter", "([B)Z", (data,)).await?;
            assert!(result);

            Ok(())
        })
    }
}
