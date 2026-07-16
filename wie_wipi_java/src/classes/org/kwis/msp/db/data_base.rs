use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::rms::RecordStore;

use super::{DataComparator, DataFilter};

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
                JavaMethodProto::new(
                    "deleteDataBase",
                    "(Ljava/lang/String;I)V",
                    Self::delete_data_base_with_flag,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("deleteRecord", "(I)V", Self::delete_record, Default::default()),
                JavaMethodProto::new("selectRecord", "(I[BI)V", Self::select_record_into, Default::default()),
                JavaMethodProto::new(
                    "sortRecord",
                    "(Lorg/kwis/msp/db/DataFilter;Lorg/kwis/msp/db/DataComparator;)[I",
                    Self::sort_record,
                    Default::default(),
                ),
                JavaMethodProto::new("listDataBases", "()[Ljava/lang/String;", Self::list_data_bases, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getAccessMode", "(Ljava/lang/String;)I", Self::get_access_mode, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getDataBaseName", "()Ljava/lang/String;", Self::get_data_base_name, Default::default()),
                JavaMethodProto::new("getDataBaseSize", "()I", Self::get_data_base_size, Default::default()),
                JavaMethodProto::new("getRecordSize", "()I", Self::get_record_size, Default::default()),
                JavaMethodProto::new("getSizeAvailable", "()I", Self::get_size_available, Default::default()),
                JavaMethodProto::new("getLastModified", "()J", Self::get_last_modified, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("recordStore", "Ljavax/microedition/rms/RecordStore;", Default::default()),
                JavaFieldProto::new("recordSize", "I", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, record_store: ClassInstanceRef<RecordStore>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::<init>({this:?}, {record_store:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "recordStore", "Ljavax/microedition/rms/RecordStore;", record_store)
            .await?;
        jvm.put_field(&mut this, "recordSize", "I", 0).await?;

        Ok(())
    }

    async fn open_data_base(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data_base_name: ClassInstanceRef<String>,
        record_size: i32,
        create: bool,
    ) -> JvmResult<ClassInstanceRef<DataBase>> {
        tracing::warn!("org.kwis.msp.db.DataBase::openDataBase({data_base_name:?}, {record_size}, {create})");

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
        tracing::debug!("org.kwis.msp.db.DataBase::openDataBase({data_base_name:?}, {record_size}, {create}, {flags})");

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .invoke_static(
                "javax/microedition/rms/RecordStore",
                "openRecordStore",
                "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                (data_base_name, create),
            )
            .await?;

        let mut instance: ClassInstanceRef<DataBase> = jvm
            .new_class("org/kwis/msp/db/DataBase", "(Ljavax/microedition/rms/RecordStore;)V", (record_store,))
            .await?
            .into();
        jvm.put_field(&mut instance, "recordSize", "I", record_size).await?;

        Ok(instance)
    }

    async fn get_number_of_records(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getNumberOfRecords({this:?})");

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        jvm.invoke_virtual(&record_store, "getNumRecords", "()I", ()).await
    }

    async fn close_data_base(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<DataBase>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::closeDataBase({this:?})");

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        jvm.invoke_virtual(&record_store, "closeRecordStore", "()V", ()).await
    }

    async fn insert_record(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::insertRecord({this:?}, {data:?})");

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
        tracing::debug!("org.kwis.msp.db.DataBase::insertRecord({this:?}, {data:?}, {offset}, {num_bytes})");

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        let record_id = jvm
            .invoke_virtual(&record_store, "addRecord", "([BII)I", (data, offset, num_bytes))
            .await?;

        Ok(DataBase::to_wipi_record_id(record_id))
    }

    async fn select_record(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<ClassInstanceRef<i8>> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({this:?}, {record_id})");

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
        tracing::debug!("org.kwis.msp.db.DataBase::updateRecord({this:?}, {record_id}, {data:?})");

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
        tracing::debug!("org.kwis.msp.db.DataBase::updateRecord({this:?}, {record_id}, {data:?}, {offset}, {num_bytes})");

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

    async fn delete_data_base_with_flag(_: &Jvm, _: &mut WieJvmContext, data_base_name: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::deleteDataBase({data_base_name:?}, {flag})");

        Ok(())
    }

    async fn delete_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::deleteRecord({this:?}, {record_id})");

        let record_id = DataBase::to_midp_record_id(record_id);
        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        let result: JvmResult<()> = jvm.invoke_virtual(&record_store, "deleteRecord", "(I)V", (record_id,)).await;
        if result.is_err() {
            return Err(jvm.exception("org/kwis/msp/db/DataBaseRecordException", "Record not found").await);
        }

        Ok(())
    }

    async fn select_record_into(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({this:?}, {record_id}, {buffer:?}, {offset})");

        let record_id = DataBase::to_midp_record_id(record_id);
        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        let record_size: JvmResult<i32> = jvm.invoke_virtual(&record_store, "getRecordSize", "(I)I", (record_id,)).await;
        let record_size = match record_size {
            Ok(record_size) => record_size,
            Err(_) => return Err(jvm.exception("org/kwis/msp/db/DataBaseRecordException", "Record not found").await),
        };

        let buffer_length = jvm.array_length(&buffer).await? as i64;
        if offset < 0 || i64::from(offset) + i64::from(record_size) > buffer_length {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "buffer is too small for the record")
                .await);
        }

        let _: i32 = jvm
            .invoke_virtual(&record_store, "getRecord", "(I[BI)I", (record_id, buffer, offset))
            .await?;

        Ok(())
    }

    async fn sort_record(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        filter: ClassInstanceRef<DataFilter>,
        comparator: ClassInstanceRef<DataComparator>,
    ) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::sortRecord({this:?}, {filter:?}, {comparator:?})");

        Ok(ClassInstanceRef::new(None))
    }

    async fn list_data_bases(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::debug!("org.kwis.msp.db.DataBase::listDataBases()");

        jvm.invoke_static("javax/microedition/rms/RecordStore", "listRecordStores", "()[Ljava/lang/String;", ())
            .await
    }

    async fn get_access_mode(_: &Jvm, _: &mut WieJvmContext, data_base_name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::getAccessMode({data_base_name:?})");

        Ok(0)
    }

    async fn get_data_base_name(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.db.DataBase::getDataBaseName({this:?})");

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        jvm.get_field(&record_store, "dbName", "Ljava/lang/String;").await
    }

    async fn get_data_base_size(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::getDataBaseSize({this:?})");

        Ok(0)
    }

    async fn get_record_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getRecordSize({this:?})");

        jvm.get_field(&this, "recordSize", "I").await
    }

    async fn get_size_available(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getSizeAvailable({this:?})");

        let record_store = jvm.get_field(&this, "recordStore", "Ljavax/microedition/rms/RecordStore;").await?;
        jvm.invoke_virtual(&record_store, "getSizeAvailable", "()I", ()).await
    }

    async fn get_last_modified(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i64> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::getLastModified({this:?})");

        Ok(0)
    }

    // wipi record id starts with 0 but midp record id starts with 1
    fn to_midp_record_id(wipi_record_id: i32) -> i32 {
        wipi_record_id + 1
    }

    fn to_wipi_record_id(midp_record_id: i32) -> i32 {
        midp_record_id - 1
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::DataBase;

    #[test]
    fn test_database_state_selection_and_stubs() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "storage-handset").await?.into();
            let database: ClassInstanceRef<DataBase> = jvm
                .invoke_static(
                    "org/kwis/msp/db/DataBase",
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    (name.clone(), 32, true),
                )
                .await?;

            let database_name: ClassInstanceRef<String> = jvm.invoke_virtual(&database, "getDataBaseName", "()Ljava/lang/String;", ()).await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &database_name).await?.as_str(), "storage-handset");

            let record_size: i32 = jvm.invoke_virtual(&database, "getRecordSize", "()I", ()).await?;
            assert_eq!(record_size, 32);

            let mut record = jvm.instantiate_array("B", 3).await?;
            jvm.store_array(&mut record, 0, [1i8, 2, 3]).await?;
            let record_id: i32 = jvm.invoke_virtual(&database, "insertRecord", "([B)I", (record,)).await?;

            let mut buffer = jvm.instantiate_array("B", 5).await?;
            jvm.store_array(&mut buffer, 0, [9i8; 5]).await?;
            let _: () = jvm
                .invoke_virtual(&database, "selectRecord", "(I[BI)V", (record_id, buffer.clone(), 1))
                .await?;
            assert_eq!(jvm.load_array::<i8>(&buffer, 0, 5).await?, [9i8, 1, 2, 3, 9]);

            let short_buffer = jvm.instantiate_array("B", 3).await?;
            let short_result: JvmResult<()> = jvm
                .invoke_virtual(&database, "selectRecord", "(I[BI)V", (record_id, short_buffer, 1))
                .await;
            let Err(JavaError::JavaException(exception)) = short_result else {
                panic!("selectRecord accepted a buffer smaller than the record");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let missing_result: JvmResult<()> = jvm.invoke_virtual(&database, "selectRecord", "(I[BI)V", (99, buffer.clone(), 0)).await;
            let Err(JavaError::JavaException(exception)) = missing_result else {
                panic!("selectRecord accepted an unknown record ID");
            };
            assert!(jvm.is_instance(&*exception, "org/kwis/msp/db/DataBaseRecordException"));

            let sorted: ClassInstanceRef<Array<i32>> = jvm
                .invoke_virtual(
                    &database,
                    "sortRecord",
                    "(Lorg/kwis/msp/db/DataFilter;Lorg/kwis/msp/db/DataComparator;)[I",
                    (ClassInstanceRef::<()>::new(None), ClassInstanceRef::<()>::new(None)),
                )
                .await?;
            assert!(sorted.is_null());

            let databases: ClassInstanceRef<Array<String>> = jvm
                .invoke_static("org/kwis/msp/db/DataBase", "listDataBases", "()[Ljava/lang/String;", ())
                .await?;
            assert_eq!(jvm.array_length(&databases).await?, 0);

            let access_mode: i32 = jvm
                .invoke_static("org/kwis/msp/db/DataBase", "getAccessMode", "(Ljava/lang/String;)I", (name.clone(),))
                .await?;
            let database_size: i32 = jvm.invoke_virtual(&database, "getDataBaseSize", "()I", ()).await?;
            let size_available: i32 = jvm.invoke_virtual(&database, "getSizeAvailable", "()I", ()).await?;
            let last_modified: i64 = jvm.invoke_virtual(&database, "getLastModified", "()J", ()).await?;
            assert_eq!(access_mode, 0);
            assert_eq!(database_size, 0);
            assert_eq!(size_available, 1_000_000);
            assert_eq!(last_modified, 0);

            let _: () = jvm
                .invoke_static("org/kwis/msp/db/DataBase", "deleteDataBase", "(Ljava/lang/String;I)V", (name, 1))
                .await?;

            Ok(())
        })
    }

    #[test]
    fn test_delete_record_updates_count_and_translates_missing_ids() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "delete-records").await?.into();
            let database: ClassInstanceRef<DataBase> = jvm
                .invoke_static(
                    "org/kwis/msp/db/DataBase",
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    (name, 32, true),
                )
                .await?;

            let mut first = jvm.instantiate_array("B", 2).await?;
            jvm.store_array(&mut first, 0, [1i8, 2]).await?;
            let first_id: i32 = jvm.invoke_virtual(&database, "insertRecord", "([B)I", (first,)).await?;

            let mut second = jvm.instantiate_array("B", 2).await?;
            jvm.store_array(&mut second, 0, [3i8, 4]).await?;
            let second_id: i32 = jvm.invoke_virtual(&database, "insertRecord", "([B)I", (second,)).await?;
            assert_eq!((first_id, second_id), (0, 1));

            let count: i32 = jvm.invoke_virtual(&database, "getNumberOfRecords", "()I", ()).await?;
            assert_eq!(count, 2);

            let _: () = jvm.invoke_virtual(&database, "deleteRecord", "(I)V", (first_id,)).await?;
            let count: i32 = jvm.invoke_virtual(&database, "getNumberOfRecords", "()I", ()).await?;
            assert_eq!(count, 1);

            let deleted: JvmResult<ClassInstanceRef<Array<i8>>> = jvm.invoke_virtual(&database, "selectRecord", "(I)[B", (first_id,)).await;
            let Err(JavaError::JavaException(exception)) = deleted else {
                panic!("deleted WIPI record lookup succeeded");
            };
            assert!(jvm.is_instance(&*exception, "org/kwis/msp/db/DataBaseRecordException"));

            let remaining: ClassInstanceRef<Array<i8>> = jvm.invoke_virtual(&database, "selectRecord", "(I)[B", (second_id,)).await?;
            assert_eq!(jvm.load_array::<i8>(&remaining, 0, 2).await?, [3i8, 4]);

            let unknown: JvmResult<()> = jvm.invoke_virtual(&database, "deleteRecord", "(I)V", (99,)).await;
            let Err(JavaError::JavaException(exception)) = unknown else {
                panic!("unknown WIPI record deletion succeeded");
            };
            assert!(jvm.is_instance(&*exception, "org/kwis/msp/db/DataBaseRecordException"));

            let count: i32 = jvm.invoke_virtual(&database, "getNumberOfRecords", "()I", ()).await?;
            assert_eq!(count, 1);

            Ok(())
        })
    }
}
