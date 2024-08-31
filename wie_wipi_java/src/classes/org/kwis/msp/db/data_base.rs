use alloc::{borrow::ToOwned, boxed::Box, vec};

use bytemuck::cast_vec;
use wie_backend::Database;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.db.DataBase
pub struct DataBase {}

impl DataBase {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataBase",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    Self::open_data_base,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getNumberOfRecords", "()I", Self::get_number_of_records, Default::default()),
                JavaMethodProto::new("closeDataBase", "()V", Self::close_data_base, Default::default()),
                JavaMethodProto::new("insertRecord", "([BII)I", Self::insert_record, Default::default()),
                JavaMethodProto::new("selectRecord", "(I)[B", Self::select_record, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("dbName", "Ljava/lang/String;", Default::default())],
        }
    }
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, data_base_name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::<init>({:?}, {:?})", &this, &data_base_name);

        jvm.put_field(&mut this, "dbName", "Ljava/lang/String;", data_base_name).await?;

        Ok(())
    }

    async fn open_data_base(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data_base_name: ClassInstanceRef<String>,
        record_size: i32,
        create: bool,
    ) -> JvmResult<ClassInstanceRef<DataBase>> {
        tracing::warn!(
            "stub org.kwis.msp.db.DataBase::openDataBase({:?}, {}, {})",
            &data_base_name,
            record_size,
            create
        );

        let instance = jvm
            .new_class("org/kwis/msp/db/DataBase", "(Ljava/lang/String;)V", (data_base_name,))
            .await?;

        Ok(instance.into())
    }

    async fn get_number_of_records(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getNumberOfRecords({:?})", &this);

        let database = Self::get_database(jvm, context, &this).await?;

        let count = database.get_record_ids().len();

        Ok(count as _)
    }

    async fn close_data_base(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<DataBase>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:?})", &this);

        Ok(())
    }

    async fn insert_record(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        num_bytes: i32,
    ) -> JvmResult<i32> {
        tracing::debug!(
            "org.kwis.msp.db.DataBase::insertRecord({:?}, {:?}, {}, {})",
            &this,
            &data,
            offset,
            num_bytes
        );

        let mut database = Self::get_database(jvm, context, &this).await?;

        let data = jvm.load_byte_array(&data, offset as _, num_bytes as _).await?;
        let data_raw = cast_vec(data);

        let id = database.add(&data_raw);

        Ok(id as _)
    }

    async fn select_record(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<ClassInstanceRef<i8>> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({:?}, {})", &this, record_id);

        let database = Self::get_database(jvm, context, &this).await?;

        let data = database.get(record_id as _).unwrap();

        let mut array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_byte_array(&mut array, 0, cast_vec(data)).await?;

        Ok(array.into())
    }

    async fn get_database(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn Database>> {
        let db_name = jvm.get_field(this, "dbName", "Ljava/lang/String;").await?;
        let db_name_str = JavaLangString::to_rust_string(jvm, &db_name).await?;

        let app_id = context.system().app_id().to_owned();

        Ok(context.system().platform().database_repository().open(&db_name_str, &app_id))
    }
}
