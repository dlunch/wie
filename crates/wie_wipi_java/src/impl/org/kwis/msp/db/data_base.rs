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
    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.db.DataBase::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn open_data_base(
        context: &mut dyn JavaContext,
        data_base_name: JavaObjectProxy,
        record_size: u32,
        create: u32,
    ) -> JavaResult<JavaObjectProxy> {
        log::warn!(
            "stub org.kwis.msp.db.DataBase::openDataBase({:#x}, {}, {})",
            data_base_name.ptr_instance,
            record_size,
            create
        );

        let instance = context.instantiate("Lorg/kwis/msp/db/DataBase;")?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_number_of_records(_: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.db.DataBase::getNumberOfRecords({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn close_data_base(_: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn insert_record(_: &mut dyn JavaContext, this: JavaObjectProxy, data: JavaObjectProxy, offset: u32, num_bytes: u32) -> JavaResult<u32> {
        log::warn!(
            "stub org.kwis.msp.db.DataBase::insertRecord({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            data.ptr_instance,
            offset,
            num_bytes
        );

        Ok(0)
    }
}
