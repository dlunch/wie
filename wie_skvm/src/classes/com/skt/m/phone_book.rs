use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const ALL: i32 = 0;
const NAME: i32 = 1;
const GROUP: i32 = 2;
const HANDPHONE: i32 = 3;
const HOME: i32 = 4;
const OFFICE: i32 = 5;
const EMAIL: i32 = 6;
const MEMO: i32 = 7;

// class com.skt.m.PhoneBook
pub struct PhoneBook;

impl PhoneBook {
    pub fn as_proto() -> WieJavaClassProto {
        let public_static = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;
        let public_static_final = FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL;

        WieJavaClassProto {
            name: "com/skt/m/PhoneBook",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("findRecord", "(ILjava/lang/String;)V", Self::find_record, public_static),
                JavaMethodProto::new("first", "()V", Self::first, public_static),
                JavaMethodProto::new("getField", "(II)Ljava/lang/String;", Self::get_field, public_static),
                JavaMethodProto::new("getGroupNames", "()[Ljava/lang/String;", Self::get_group_names, public_static),
                JavaMethodProto::new("getMaxRecordID", "()I", Self::get_max_record_id, public_static),
                JavaMethodProto::new("getRecord", "(I)[Ljava/lang/String;", Self::get_record, public_static),
                JavaMethodProto::new("isUsed", "(I)Z", Self::is_used, public_static),
                JavaMethodProto::new("next", "()I", Self::next, public_static),
            ],
            fields: ["ALL", "NAME", "GROUP", "HANDPHONE", "HOME", "OFFICE", "EMAIL", "MEMO"]
                .into_iter()
                .map(|name| JavaFieldProto::new(name, "I", public_static_final))
                .collect(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.PhoneBook::<clinit>()");

        for (name, value) in [
            ("ALL", ALL),
            ("NAME", NAME),
            ("GROUP", GROUP),
            ("HANDPHONE", HANDPHONE),
            ("HOME", HOME),
            ("OFFICE", OFFICE),
            ("EMAIL", EMAIL),
            ("MEMO", MEMO),
        ] {
            jvm.put_static_field("com/skt/m/PhoneBook", name, "I", value).await?;
        }
        Ok(())
    }

    async fn find_record(jvm: &Jvm, _context: &mut WieJvmContext, field: i32, value: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.PhoneBook::findRecord({field}, {value:?})");
        if value.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "value is null").await);
        }
        Ok(())
    }

    async fn first(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.PhoneBook::first()");
        Ok(())
    }

    async fn get_field(_jvm: &Jvm, _context: &mut WieJvmContext, record_id: i32, field: i32) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub com.skt.m.PhoneBook::getField({record_id}, {field})");
        Ok(ClassInstanceRef::new(None))
    }

    async fn get_group_names(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::warn!("stub com.skt.m.PhoneBook::getGroupNames()");
        Ok(jvm.instantiate_array("Ljava/lang/String;", 0).await?.into())
    }

    async fn get_max_record_id(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.PhoneBook::getMaxRecordID()");
        Ok(0)
    }

    async fn get_record(_jvm: &Jvm, _context: &mut WieJvmContext, record_id: i32) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::warn!("stub com.skt.m.PhoneBook::getRecord({record_id})");
        Ok(ClassInstanceRef::new(None))
    }

    async fn is_used(_jvm: &Jvm, _context: &mut WieJvmContext, record_id: i32) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.PhoneBook::isUsed({record_id})");
        Ok(false)
    }

    async fn next(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.PhoneBook::next()");
        Ok(-1)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{Array, ClassInstanceRef};
    use test_utils::run_jvm_test;

    use crate::get_protos;

    #[test]
    fn test_empty_phone_book_uses_documented_constants_and_null_record() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            for (name, expected) in [
                ("ALL", 0),
                ("NAME", 1),
                ("GROUP", 2),
                ("HANDPHONE", 3),
                ("HOME", 4),
                ("OFFICE", 5),
                ("EMAIL", 6),
                ("MEMO", 7),
            ] {
                assert_eq!(jvm.get_static_field::<i32>("com/skt/m/PhoneBook", name, "I").await?, expected);
            }

            let record: ClassInstanceRef<Array<String>> = jvm
                .invoke_static("com/skt/m/PhoneBook", "getRecord", "(I)[Ljava/lang/String;", (1,))
                .await?;
            assert!(record.is_null());
            Ok(())
        });

        assert!(result.is_ok());
    }
}
