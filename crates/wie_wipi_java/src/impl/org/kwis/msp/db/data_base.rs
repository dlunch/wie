use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.db.DataBase
pub struct DataBase {}

impl DataBase {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    Self::open_data_base,
                    JavaMethodAccessFlag::NONE,
                ),
                JavaMethodProto::new("getNumberOfRecords", "()I", Self::get_number_of_records, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("closeDataBase", "()V", Self::close_data_base, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("insertRecord", "([BII)I", Self::insert_record, JavaMethodAccessFlag::NONE),
            ],
            fields: vec![],
        }
    }
    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.db.DataBase::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn open_data_base(context: &mut dyn JavaContext, _a0: JavaObjectProxy, _a1: u32, _a2: u32) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub org.kwis.msp.db.DataBase::openDataBase({:#x}, {}, {})", _a0.ptr_instance, _a1, _a2);

        let instance = context.instantiate("Lorg/kwis/msp/db/DataBase;")?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_number_of_records(_: &mut dyn JavaContext, _a0: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.db.DataBase::getNumberOfRecords({:#x})", _a0.ptr_instance);

        Ok(0)
    }

    async fn close_data_base(_: &mut dyn JavaContext, _a0: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:#x})", _a0.ptr_instance);

        Ok(())
    }

    async fn insert_record(_: &mut dyn JavaContext, _a0: JavaObjectProxy, _a1: JavaObjectProxy, _a2: u32, _a3: u32) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.db.DataBase::insertRecord({:#x})", _a0.ptr_instance);

        Ok(0)
    }
}
