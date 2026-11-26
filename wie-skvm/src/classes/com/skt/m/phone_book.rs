use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

const ALL: i32 = 0;
const NAME: i32 = 1;
const GROUP: i32 = 2;
const HANDPHONE: i32 = 3;
const HOME: i32 = 4;
const OFFICE: i32 = 5;
const EMAIL: i32 = 6;
const MEMO: i32 = 7;

// class com.skt.m.PhoneBook
#[java_class(name = "com/skt/m/PhoneBook", public)]
pub struct PhoneBook {
    #[field(public, static, final)]
    ALL: i32,
    #[field(public, static, final)]
    NAME: i32,
    #[field(public, static, final)]
    GROUP: i32,
    #[field(public, static, final)]
    HANDPHONE: i32,
    #[field(public, static, final)]
    HOME: i32,
    #[field(public, static, final)]
    OFFICE: i32,
    #[field(public, static, final)]
    EMAIL: i32,
    #[field(public, static, final)]
    MEMO: i32,
}

#[java_methods]
impl PhoneBook {
    #[method(static_initializer)]
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

    #[method(public, static)]
    async fn find_record(jvm: &Jvm, _context: &mut WieJvmContext, field: i32, value: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.PhoneBook::findRecord({field}, {value:?})");
        if value.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "value is null").await);
        }
        Ok(())
    }

    #[method(public, static)]
    async fn first(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.PhoneBook::first()");
        Ok(())
    }

    #[method(public, static)]
    async fn get_field(_jvm: &Jvm, _context: &mut WieJvmContext, record_id: i32, field: i32) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub com.skt.m.PhoneBook::getField({record_id}, {field})");
        Ok(ClassInstanceRef::new(None))
    }

    #[method(public, static)]
    async fn get_group_names(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::warn!("stub com.skt.m.PhoneBook::getGroupNames()");
        Ok(jvm.instantiate_array("Ljava/lang/String;", 0).await?.into())
    }

    #[method(name = "getMaxRecordID", public, static)]
    async fn get_max_record_id(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.PhoneBook::getMaxRecordID()");
        Ok(0)
    }

    #[method(public, static)]
    async fn get_record(_jvm: &Jvm, _context: &mut WieJvmContext, record_id: i32) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::warn!("stub com.skt.m.PhoneBook::getRecord({record_id})");
        Ok(ClassInstanceRef::new(None))
    }

    #[method(public, static)]
    async fn is_used(_jvm: &Jvm, _context: &mut WieJvmContext, record_id: i32) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.PhoneBook::isUsed({record_id})");
        Ok(false)
    }

    #[method(public, static)]
    async fn next(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.PhoneBook::next()");
        Ok(-1)
    }
}
