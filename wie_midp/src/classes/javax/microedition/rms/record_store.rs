use alloc::{borrow::ToOwned, boxed::Box, vec, vec::Vec};

use bytemuck::cast_vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_backend::Database;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.rms.RecordStore
pub struct RecordStore;

impl RecordStore {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/rms/RecordStore",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("addRecord", "([BII)I", Self::add_record, Default::default()),
                JavaMethodProto::new("getSizeAvailable", "()I", Self::get_size_available, Default::default()),
                JavaMethodProto::new("getNextRecordID", "()I", Self::get_next_record_id, Default::default()),
                JavaMethodProto::new("getRecord", "(I)[B", Self::get_record, Default::default()),
                JavaMethodProto::new("getRecord", "(I[BI)I", Self::get_record_array, Default::default()),
                JavaMethodProto::new("getRecordSize", "(I)I", Self::get_record_size, Default::default()),
                JavaMethodProto::new("setRecord", "(I[BII)V", Self::set_record, Default::default()),
                JavaMethodProto::new("getNumRecords", "()I", Self::get_num_records, Default::default()),
                JavaMethodProto::new("closeRecordStore", "()V", Self::close_record_store, Default::default()),
                JavaMethodProto::new(
                    "openRecordStore",
                    "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                    Self::open_record_store,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "deleteRecordStore",
                    "(Ljava/lang/String;)V",
                    Self::delete_record_store,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![JavaFieldProto::new("dbName", "Ljava/lang/String;", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, db_name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.rms.RecordStore::<init>({:?}, {:?})", &this, &db_name);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "dbName", "Ljava/lang/String;", db_name).await?;

        Ok(())
    }

    async fn add_record(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!(
            "javax.microedition.rms.RecordStore::addRecord({:?}, {:?}, {}, {})",
            &this,
            &data,
            offset,
            length
        );

        let mut database = Self::get_database(jvm, context, &this).await?;

        let data: Vec<i8> = jvm.load_array(&data, offset as _, length as _).await?;

        let id = database.add(&cast_vec(data)).await;

        Ok(id as _)
    }

    async fn get_size_available(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::getSizeAvailable({:?})", &this);

        Ok(1000000 as _)
    }

    async fn get_next_record_id(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::getNextRecordID({:?})", &this);

        let database = Self::get_database(jvm, context, &this).await?;

        let next_id = database.next_id().await;

        Ok(next_id as _)
    }

    async fn get_record(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
    ) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("javax.microedition.rms.RecordStore::getRecord({:?}, {})", &this, record_id);

        let database = Self::get_database(jvm, context, &this).await?;

        let result = database.get(record_id as _).await;
        if result.is_none() {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "Record not found").await);
        }

        let data = result.unwrap();

        let mut array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_array(&mut array, 0, cast_vec::<u8, i8>(data)).await?;

        Ok(array.into())
    }

    async fn get_record_array(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        mut buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
    ) -> JvmResult<i32> {
        tracing::debug!(
            "javax.microedition.rms.RecordStore::getRecord({:?}, {}, {:?}, {})",
            &this,
            record_id,
            &buffer,
            offset
        );

        let database = Self::get_database(jvm, context, &this).await?;

        let result = database.get(record_id as _).await;
        if result.is_none() {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "Record not found").await);
        }

        let data = result.unwrap();
        let data_length = data.len();
        jvm.store_array(&mut buffer, offset as _, cast_vec::<u8, i8>(data)).await?;

        Ok(data_length as _)
    }

    async fn get_record_size(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::getRecordSize({:?}, {})", &this, record_id);

        let database = Self::get_database(jvm, context, &this).await?;

        let result = database.get(record_id as _).await;
        if result.is_none() {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "Record not found").await);
        }

        let data = result.unwrap();

        Ok(data.len() as _)
    }

    async fn set_record(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.rms.RecordStore::setRecord({:?}, {}, {:?}, {}, {})",
            &this,
            record_id,
            &data,
            offset,
            length
        );

        let data: Vec<i8> = jvm.load_array(&data, offset as _, length as _).await?;

        let mut database = Self::get_database(jvm, context, &this).await?;
        database.set(record_id as _, &cast_vec(data)).await;

        Ok(())
    }

    async fn get_num_records(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.rms.RecordStore::getNumRecords({:?})", &this);

        let database = Self::get_database(jvm, context, &this).await?;

        let count = database.get_record_ids().await.len();

        Ok(count as _)
    }

    async fn close_record_store(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::closeRecordStore({:?})", &this);

        Ok(())
    }

    async fn open_record_store(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        create: bool,
    ) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.rms.RecordStore::openRecordStore({:?}, {:?})", name, create);

        let store = jvm
            .new_class("javax/microedition/rms/RecordStore", "(Ljava/lang/String;)V", (name,))
            .await?;

        Ok(store.into())
    }

    async fn delete_record_store(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::deleteRecordStore({name:?})");

        Ok(())
    }

    async fn get_database(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn Database>> {
        let db_name = jvm.get_field(this, "dbName", "Ljava/lang/String;").await?;
        let db_name_str = JavaLangString::to_rust_string(jvm, &db_name).await?;

        let app_id = context.system().app_id().to_owned();

        Ok(context.system().platform().database_repository().open(&db_name_str, &app_id).await)
    }
}
