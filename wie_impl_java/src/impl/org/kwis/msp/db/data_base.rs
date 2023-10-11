use alloc::vec;

use crate::{
    array::Array,
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::java::lang::String,
};

// class org.kwis.msp.db.DataBase
pub struct DataBase {}

impl DataBase {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    Self::open_data_base,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new("getNumberOfRecords", "()I", Self::get_number_of_records, JavaMethodFlag::NONE),
                JavaMethodProto::new("closeDataBase", "()V", Self::close_data_base, JavaMethodFlag::NONE),
                JavaMethodProto::new("insertRecord", "([BII)I", Self::insert_record, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }
    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<DataBase>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn open_data_base(
        context: &mut dyn JavaContext,
        data_base_name: JavaObjectProxy<String>,
        record_size: i32,
        create: i32,
    ) -> JavaResult<JavaObjectProxy<DataBase>> {
        tracing::warn!(
            "stub org.kwis.msp.db.DataBase::openDataBase({:#x}, {}, {})",
            data_base_name.ptr_instance,
            record_size,
            create
        );

        let instance = context.instantiate("Lorg/kwis/msp/db/DataBase;").await?.cast();
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_number_of_records(_: &mut dyn JavaContext, this: JavaObjectProxy<DataBase>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::getNumberOfRecords({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn close_data_base(_: &mut dyn JavaContext, this: JavaObjectProxy<DataBase>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn insert_record(
        _: &mut dyn JavaContext,
        this: JavaObjectProxy<DataBase>,
        data: JavaObjectProxy<Array>,
        offset: i32,
        num_bytes: i32,
    ) -> JavaResult<i32> {
        tracing::warn!(
            "stub org.kwis.msp.db.DataBase::insertRecord({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            data.ptr_instance,
            offset,
            num_bytes
        );

        Ok(0)
    }
}
