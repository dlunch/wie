use alloc::{vec, vec::Vec};

use bytemuck::cast_vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{Array, JvmClassInstanceProxy},
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
    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, data_base_name: JvmClassInstanceProxy<String>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::<init>({:?}, {:?})", &this, &data_base_name);

        context.jvm().put_field(&this, "dbName", "Ljava/lang/String;", data_base_name)?;

        Ok(())
    }

    async fn open_data_base(
        context: &mut dyn JavaContext,
        data_base_name: JvmClassInstanceProxy<String>,
        record_size: i32,
        create: bool,
    ) -> JavaResult<JvmClassInstanceProxy<DataBase>> {
        tracing::warn!(
            "stub org.kwis.msp.db.DataBase::openDataBase({:?}, {}, {})",
            &data_base_name,
            record_size,
            create
        );

        let instance = context.jvm().instantiate_class("org/kwis/msp/db/DataBase").await?;
        context
            .jvm()
            .invoke_special(
                &instance,
                "org/kwis/msp/db/DataBase",
                "<init>",
                "(Ljava/lang/String;)V",
                (data_base_name,),
            )
            .await?;

        Ok(instance.into())
    }

    async fn get_number_of_records(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getNumberOfRecords({:?})", &this);

        let db_name = context.jvm().get_field(&this, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, &db_name)?;

        let count = context.backend().database().open(&db_name_str)?.count()?;

        Ok(count as _)
    }

    async fn close_data_base(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<DataBase>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:?})", &this);

        Ok(())
    }

    async fn insert_record(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        data: JvmClassInstanceProxy<Array<i8>>,
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

        let db_name = context.jvm().get_field(&this, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, &db_name)?;

        let data: Vec<i8> = context.jvm().load_array(&data, offset as _, num_bytes as _)?;
        let data_raw = cast_vec(data);

        let id = context.backend().database().open(&db_name_str)?.add(&data_raw)?;

        Ok(id as _)
    }

    async fn select_record(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        record_id: i32,
    ) -> JavaResult<JvmClassInstanceProxy<i8>> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({:?}, {})", &this, record_id);

        let db_name = context.jvm().get_field(&this, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, &db_name)?;

        let data = context.backend().database().open(&db_name_str)?.get(record_id as _)?;
        let data: Vec<i8> = cast_vec(data);

        let array = context.jvm().instantiate_array("B", data.len() as _).await?;
        context.jvm().store_array(&array, 0, data)?;

        Ok(array.into())
    }
}
