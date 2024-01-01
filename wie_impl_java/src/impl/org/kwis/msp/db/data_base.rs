use alloc::{boxed::Box, vec, vec::Vec};

use bytemuck::cast_vec;
use wie_backend::Database;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    handle::{Array, JvmClassInstanceHandle},
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
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    Self::open_data_base,
                    JavaMethodFlag::STATIC,
                ),
                JavaMethodProto::new("getNumberOfRecords", "()I", Self::get_number_of_records, JavaMethodFlag::NONE),
                JavaMethodProto::new("closeDataBase", "()V", Self::close_data_base, JavaMethodFlag::NONE),
                JavaMethodProto::new("insertRecord", "([BII)I", Self::insert_record, JavaMethodFlag::NONE),
                JavaMethodProto::new("selectRecord", "(I)[B", Self::select_record, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("dbName", "Ljava/lang/String;", JavaFieldAccessFlag::NONE)],
        }
    }
    async fn init(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        data_base_name: JvmClassInstanceHandle<String>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::<init>({:?}, {:?})", &this, &data_base_name);

        context.jvm().put_field(&this, "dbName", "Ljava/lang/String;", data_base_name)?;

        Ok(())
    }

    async fn open_data_base(
        context: &mut dyn JavaContext,
        data_base_name: JvmClassInstanceHandle<String>,
        record_size: i32,
        create: bool,
    ) -> JavaResult<JvmClassInstanceHandle<DataBase>> {
        tracing::warn!(
            "stub org.kwis.msp.db.DataBase::openDataBase({:?}, {}, {})",
            &data_base_name,
            record_size,
            create
        );

        let instance = context
            .jvm()
            .new_class("org/kwis/msp/db/DataBase", "(Ljava/lang/String;)V", (data_base_name,))
            .await?;

        Ok(instance.into())
    }

    async fn get_number_of_records(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getNumberOfRecords({:?})", &this);

        let database = Self::get_database(context, &this)?;

        let count = database.get_record_ids().len();

        Ok(count as _)
    }

    async fn close_data_base(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<DataBase>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:?})", &this);

        Ok(())
    }

    async fn insert_record(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        data: JvmClassInstanceHandle<Array<i8>>,
        offset: i32,
        num_bytes: i32,
    ) -> JavaResult<i32> {
        tracing::debug!(
            "org.kwis.msp.db.DataBase::insertRecord({:?}, {:?}, {}, {})",
            &this,
            &data,
            offset,
            num_bytes
        );

        let mut database = Self::get_database(context, &this)?;

        let data: Vec<i8> = context.jvm().load_array(&data, offset as _, num_bytes as _)?;
        let data_raw = cast_vec(data);

        let id = database.add(&data_raw);

        Ok(id as _)
    }

    async fn select_record(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        record_id: i32,
    ) -> JavaResult<JvmClassInstanceHandle<i8>> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({:?}, {})", &this, record_id);

        let database = Self::get_database(context, &this)?;

        let data = database.get(record_id as _).unwrap();
        let data: Vec<i8> = cast_vec(data);

        let array = context.jvm().instantiate_array("B", data.len() as _).await?;
        context.jvm().store_array(&array, 0, data)?;

        Ok(array.into())
    }

    fn get_database(context: &mut dyn JavaContext, this: &JvmClassInstanceHandle<Self>) -> JavaResult<Box<dyn Database>> {
        let db_name = context.jvm().get_field(this, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, &db_name)?;

        Ok(context.system().platform().database_repository().open(&db_name_str))
    }
}
