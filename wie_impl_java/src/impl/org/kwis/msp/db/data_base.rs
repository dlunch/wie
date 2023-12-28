use alloc::{vec, vec::Vec};

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{JavaObjectProxy, JvmArrayClassInstanceProxy, JvmClassInstanceProxy},
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
                JavaMethodProto::new("selectRecord", "(I)[B", Self::select_record, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("dbName", "Ljava/lang/String;", JavaFieldAccessFlag::NONE)],
        }
    }
    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, data_base_name: JvmClassInstanceProxy<String>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::<init>({:?}, {:?})", &this, &data_base_name);

        context.jvm().put_field(
            &this.class_instance,
            "dbName",
            "Ljava/lang/String;",
            JavaValue::Object(Some(data_base_name.class_instance)),
        )?;

        Ok(())
    }

    async fn open_data_base(
        context: &mut dyn JavaContext,
        data_base_name: JvmClassInstanceProxy<String>,
        record_size: i32,
        create: i32,
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
            .invoke_method(
                &instance,
                "org/kwis/msp/db/DataBase",
                "<init>",
                "()V",
                &[JavaValue::Object(Some(data_base_name.class_instance))],
            )
            .await?;

        Ok(JvmClassInstanceProxy::new(instance))
    }

    async fn get_number_of_records(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getNumberOfRecords({:?})", &this);

        let db_name = context.jvm().get_field(&this.class_instance, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, db_name.as_object_ref().unwrap())?;

        let count = context.backend().database().open(&db_name_str)?.count()?;

        Ok(count as _)
    }

    async fn close_data_base(_: &mut dyn JavaContext, this: JavaObjectProxy<DataBase>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:?})", this.ptr_instance);

        Ok(())
    }

    async fn insert_record(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        data: JvmArrayClassInstanceProxy<i8>,
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

        let db_name = context.jvm().get_field(&this.class_instance, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, db_name.as_object_ref().unwrap())?;

        let data = context.jvm().load_array(&data.class_instance, offset as _, num_bytes as _)?;
        let data_raw = data.into_iter().map(|x| x.as_byte() as u8).collect::<Vec<_>>();

        let id = context.backend().database().open(&db_name_str)?.add(&data_raw)?;

        Ok(id as _)
    }

    async fn select_record(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        record_id: i32,
    ) -> JavaResult<JvmArrayClassInstanceProxy<i8>> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({:?}, {})", &this, record_id);

        let db_name = context.jvm().get_field(&this.class_instance, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, db_name.as_object_ref().unwrap())?;

        let data = context.backend().database().open(&db_name_str)?.get(record_id as _)?;
        let data = data.into_iter().map(|x| JavaValue::Byte(x as _)).collect::<Vec<_>>();

        let array = context.jvm().instantiate_array("B", data.len() as _).await?;
        context.jvm().store_array(&array, 0, &data)?;

        Ok(JvmArrayClassInstanceProxy::new(array))
    }
}
