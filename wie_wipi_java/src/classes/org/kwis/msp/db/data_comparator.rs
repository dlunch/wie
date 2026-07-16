use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.db.DataComparator
pub struct DataComparator;

impl DataComparator {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataComparator",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract("compare", "([B[B)I", MethodAccessFlags::ABSTRACT)],
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

    use super::DataComparator;

    struct ComparatorFixture;

    impl ComparatorFixture {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/ComparatorFixture",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["org/kwis/msp/db/DataComparator"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                    JavaMethodProto::new("compare", "([B[B)I", Self::compare, Default::default()),
                ],
                fields: vec![],
                access_flags: Default::default(),
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn compare(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            _: ClassInstanceRef<Self>,
            left: ClassInstanceRef<Array<i8>>,
            right: ClassInstanceRef<Array<i8>>,
        ) -> JvmResult<i32> {
            let left_length = jvm.array_length(&left).await?;
            let right_length = jvm.array_length(&right).await?;
            let left_sum: i32 = jvm.load_array::<i8>(&left, 0, left_length).await?.into_iter().map(i32::from).sum();
            let right_sum: i32 = jvm.load_array::<i8>(&right, 0, right_length).await?.into_iter().map(i32::from).sum();

            Ok(left_sum - right_sum)
        }
    }

    #[test]
    fn test_interface_contract() {
        let proto = DataComparator::as_proto();
        assert_eq!(proto.name, "org/kwis/msp/db/DataComparator");
        assert!(proto.parent_class.is_none());
        assert!(proto.access_flags.contains(ClassAccessFlags::INTERFACE));
        assert_eq!(proto.methods.len(), 1);
        assert_eq!(proto.methods[0].name, "compare");
        assert_eq!(proto.methods[0].descriptor, "([B[B)I");
        assert!(proto.methods[0].access_flags.contains(MethodAccessFlags::ABSTRACT));
        assert!(!proto.methods[0].access_flags.contains(MethodAccessFlags::STATIC));
    }

    #[test]
    fn test_callback_dispatches_byte_arrays() -> Result<()> {
        run_jvm_test(
            Box::new([[DataComparator::as_proto(), ComparatorFixture::as_proto()].into()]),
            |jvm| async move {
                let comparator: ClassInstanceRef<ComparatorFixture> = jvm.new_class("test/ComparatorFixture", "()V", ()).await?.into();
                let mut left = jvm.instantiate_array("B", 2).await?;
                let mut right = jvm.instantiate_array("B", 2).await?;
                jvm.store_array(&mut left, 0, [5i8, 7]).await?;
                jvm.store_array(&mut right, 0, [1i8, 2]).await?;

                let result: i32 = jvm.invoke_virtual(&comparator, "compare", "([B[B)I", (left, right)).await?;
                assert_eq!(result, 9);

                Ok(())
            },
        )
    }
}
