use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::rms::RecordStore;

// class org.kwis.msp.db.DataBase
pub struct DataBase;

impl DataBase {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataBase",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljavax/microedition/rms/RecordStore;)V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    Self::open_data_base,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openDataBase",
                    "(Ljava/lang/String;IZI)Lorg/kwis/msp/db/DataBase;",
                    Self::open_data_base_with_flags,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getNumberOfRecords", "()I", Self::get_number_of_records, Default::default()),
                JavaMethodProto::new("closeDataBase", "()V", Self::close_data_base, Default::default()),
                JavaMethodProto::new("insertRecord", "([B)I", Self::insert_record, Default::default()),
                JavaMethodProto::new("insertRecord", "([BII)I", Self::insert_record_with_offset, Default::default()),
                JavaMethodProto::new("selectRecord", "(I)[B", Self::select_record, Default::default()),
                JavaMethodProto::new("updateRecord", "(I[B)V", Self::update_record, Default::default()),
                JavaMethodProto::new("updateRecord", "(I[BII)V", Self::update_record_with_offset, Default::default()),
                JavaMethodProto::new(
                    "deleteDataBase",
                    "(Ljava/lang/String;)V",
                    Self::delete_data_base,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![JavaFieldProto::new(
                "recordStore",
                "Ljavax/microedition/rms/RecordStore;",
                Default::default(),
            )],
            access_flags: Default::default(),
        }
    }
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, record_store: ClassInstanceRef<RecordStore>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::<init>({:?}, {:?})", &this, &record_store);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "recordStore", "Ljavax/microedition/rms/RecordStore;", record_store)
            .await?;

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
            "org.kwis.msp.db.DataBase::openDataBase({:?}, {}, {})",
            &data_base_name,
            record_size,
            create
        );

        let result = jvm
            .invoke_static(
                "org/kwis/msp/db/DataBase",
                "openDataBase",
                "(Ljava/lang/String;IZI)Lorg/kwis/msp/db/DataBase;",
                (data_base_name, record_size, create, 0),
            )
            .await?;

        Ok(result)
    }

    async fn open_data_base_with_flags(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data_base_name: ClassInstanceRef<String>,
        record_size: i32,
        create: bool,
        flags: i32,
    ) -> JvmResult<ClassInstanceRef<DataBase>> {
        tracing::debug!(
            "org.kwis.msp.db.DataBase::openDataBase({:?}, {}, {}, {})",
            &data_base_name,
            record_size,
            create,
            flags
        );

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .invoke_static(
                "javax/microedition/rms/RecordStore",
                "openRecordStore",
                "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                (data_base_name, create),
            )
            .await?;

        let instance = jvm
            .new_class("org/kwis/msp/db/DataBase", "(Ljavax/microedition/rms/RecordStore;)V", (record_store,))
            .await?;

        Ok(instance.into())
    }

    async fn get_number_of_records(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getNumberOfRecords({:?})", &this);

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        jvm.invoke_virtual(&record_store, "getNumRecords", "()I", ()).await
    }

    async fn close_data_base(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<DataBase>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::closeDataBase({:?})", &this);

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        jvm.invoke_virtual(&record_store, "closeRecordStore", "()V", ()).await
    }

    async fn insert_record(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::insertRecord({:?}, {:?})", &this, &data);

        let length = jvm.array_length(&data).await? as i32;
        let result = jvm.invoke_virtual(&this, "insertRecord", "([BII)I", (data, 0, length)).await?;

        Ok(result)
    }

    async fn insert_record_with_offset(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
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

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        let record_id = jvm
            .invoke_virtual(&record_store, "addRecord", "([BII)I", (data, offset, num_bytes))
            .await?;

        Ok(DataBase::to_wipi_record_id(record_id))
    }

    async fn select_record(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<ClassInstanceRef<i8>> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({:?}, {})", &this, record_id);

        let record_id = DataBase::to_midp_record_id(record_id);

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        let result = jvm.invoke_virtual(&record_store, "getRecord", "(I)[B", (record_id,)).await;

        // TODO check exception type
        if result.is_err() {
            return Err(jvm.exception("org/kwis/msp/db/DataBaseRecordException", "Record not found").await);
        }

        Ok(result.unwrap())
    }

    async fn update_record(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::updateRecord({:?}, {}, {:?})", &this, record_id, &data);

        let length = jvm.array_length(&data).await? as i32;

        let _: () = jvm
            .invoke_virtual(&this, "updateRecord", "(I[BII)V", (record_id, data, 0, length))
            .await?;

        Ok(())
    }

    async fn update_record_with_offset(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        num_bytes: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "org.kwis.msp.db.DataBase::updateRecord({:?}, {}, {:?}, {}, {})",
            &this,
            record_id,
            &data,
            offset,
            num_bytes
        );

        let record_id = DataBase::to_midp_record_id(record_id);

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        let _: () = jvm
            .invoke_virtual(&record_store, "setRecord", "(I[BII)V", (record_id, data, offset, num_bytes))
            .await?;

        Ok(())
    }

    async fn delete_data_base(jvm: &Jvm, _: &mut WieJvmContext, data_base_name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::deleteDataBase({data_base_name:?})");

        let _: () = jvm
            .invoke_static(
                "javax/microedition/rms/RecordStore",
                "deleteRecordStore",
                "(Ljava/lang/String;)V",
                (data_base_name,),
            )
            .await?;

        Ok(())
    }

    // wipi record id starts with 0 but midp record id starts with 1
    fn to_midp_record_id(wipi_record_id: i32) -> i32 {
        wipi_record_id + 1
    }

    fn to_wipi_record_id(midp_record_id: i32) -> i32 {
        midp_record_id - 1
    }
}
