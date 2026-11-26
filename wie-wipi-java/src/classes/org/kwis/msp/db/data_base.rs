use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::rms::RecordStore;

use super::{DataComparator, DataFilter};

// class org.kwis.msp.db.DataBase
#[java_class(name = "org/kwis/msp/db/DataBase", public)]
pub struct DataBase {
    #[field(private)]
    recordStore: ClassInstanceRef<RecordStore>,
    #[field(private)]
    recordSize: i32,
}

#[java_methods]
impl DataBase {
    #[method(constructor, private)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, record_store: ClassInstanceRef<RecordStore>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::<init>({this:?}, {record_store:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(
            &mut this,
            "org/kwis/msp/db/DataBase",
            "recordStore",
            "Ljavax/microedition/rms/RecordStore;",
            record_store,
        )
        .await?;
        jvm.put_field(&mut this, "org/kwis/msp/db/DataBase", "recordSize", "I", 0).await?;

        Ok(())
    }

    #[method(public, static)]
    async fn open_data_base(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data_base_name: ClassInstanceRef<String>,
        record_size: i32,
        create: bool,
    ) -> JvmResult<ClassInstanceRef<DataBase>> {
        tracing::debug!("org.kwis.msp.db.DataBase::openDataBase({data_base_name:?}, {record_size}, {create})");

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

    #[method(name = "openDataBase", public, static)]
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
        jvm.put_field(&mut instance, "org/kwis/msp/db/DataBase", "recordSize", "I", record_size)
            .await?;

        Ok(instance)
    }

    #[method(public)]
    async fn get_number_of_records(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getNumberOfRecords({this:?})");

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        jvm.invoke_virtual(&record_store, "javax/microedition/rms/RecordStore", "getNumRecords", "()I", ())
            .await
    }

    #[method(public)]
    async fn close_data_base(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<DataBase>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::closeDataBase({this:?})");

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        jvm.invoke_virtual(&record_store, "javax/microedition/rms/RecordStore", "closeRecordStore", "()V", ())
            .await
    }

    #[method(public)]
    async fn insert_record(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::insertRecord({this:?}, {data:?})");

        let length = jvm.array_length(&data).await? as i32;
        let result = jvm
            .invoke_virtual(&this, "org/kwis/msp/db/DataBase", "insertRecord", "([BII)I", (data, 0, length))
            .await?;

        Ok(result)
    }

    #[method(name = "insertRecord", public)]
    async fn insert_record_with_offset(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        num_bytes: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::insertRecord({this:?}, {data:?}, {offset}, {num_bytes})");

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        let record_id = jvm
            .invoke_virtual(
                &record_store,
                "javax/microedition/rms/RecordStore",
                "addRecord",
                "([BII)I",
                (data, offset, num_bytes),
            )
            .await?;

        Ok(DataBase::to_wipi_record_id(record_id))
    }

    #[method(public)]
    async fn select_record(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
    ) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("org.kwis.msp.db.DataBase::selectRecord({this:?}, {record_id})");

        let record_id = DataBase::to_midp_record_id(record_id);

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        let result = jvm
            .invoke_virtual(&record_store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (record_id,))
            .await;

        // TODO check exception type
        if result.is_err() {
            return Err(jvm.exception("org/kwis/msp/db/DataBaseRecordException", "Record not found").await);
        }

        Ok(result.unwrap())
    }

    #[method(public)]
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
            .invoke_virtual(
                &this,
                "org/kwis/msp/db/DataBase",
                "updateRecord",
                "(I[BII)V",
                (record_id, data, 0, length),
            )
            .await?;

        Ok(())
    }

    #[method(name = "updateRecord", public)]
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

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &record_store,
                "javax/microedition/rms/RecordStore",
                "setRecord",
                "(I[BII)V",
                (record_id, data, offset, num_bytes),
            )
            .await?;

        Ok(())
    }

    #[method(public, static)]
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

    #[method(name = "deleteDataBase", public, static)]
    async fn delete_data_base_with_flag(_: &Jvm, _: &mut WieJvmContext, data_base_name: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::deleteDataBase({data_base_name:?}, {flag})");

        Ok(())
    }

    #[method(public)]
    async fn delete_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, record_id: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.db.DataBase::deleteRecord({this:?}, {record_id})");

        let record_id = DataBase::to_midp_record_id(record_id);
        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        let result: JvmResult<()> = jvm
            .invoke_virtual(&record_store, "javax/microedition/rms/RecordStore", "deleteRecord", "(I)V", (record_id,))
            .await;
        if result.is_err() {
            return Err(jvm.exception("org/kwis/msp/db/DataBaseRecordException", "Record not found").await);
        }

        Ok(())
    }

    #[method(name = "selectRecord", public)]
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
        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        let record_size: JvmResult<i32> = jvm
            .invoke_virtual(&record_store, "javax/microedition/rms/RecordStore", "getRecordSize", "(I)I", (record_id,))
            .await;
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
            .invoke_virtual(
                &record_store,
                "javax/microedition/rms/RecordStore",
                "getRecord",
                "(I[BI)I",
                (record_id, buffer, offset),
            )
            .await?;

        Ok(())
    }

    #[method(public)]
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

    #[method(public, static)]
    async fn list_data_bases(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::debug!("org.kwis.msp.db.DataBase::listDataBases()");

        jvm.invoke_static("javax/microedition/rms/RecordStore", "listRecordStores", "()[Ljava/lang/String;", ())
            .await
    }

    #[method(public, static)]
    async fn get_access_mode(_: &Jvm, _: &mut WieJvmContext, data_base_name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::getAccessMode({data_base_name:?})");

        Ok(0)
    }

    #[method(public)]
    async fn get_data_base_name(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.db.DataBase::getDataBaseName({this:?})");

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        jvm.get_field(&record_store, "javax/microedition/rms/RecordStore", "dbName", "Ljava/lang/String;")
            .await
    }

    #[method(public)]
    async fn get_data_base_size(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::getDataBaseSize({this:?})");

        Ok(0)
    }

    #[method(public)]
    async fn get_record_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getRecordSize({this:?})");

        jvm.get_field(&this, "org/kwis/msp/db/DataBase", "recordSize", "I").await
    }

    #[method(public)]
    async fn get_size_available(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.db.DataBase::getSizeAvailable({this:?})");

        let record_store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "org/kwis/msp/db/DataBase", "recordStore", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        jvm.invoke_virtual(&record_store, "javax/microedition/rms/RecordStore", "getSizeAvailable", "()I", ())
            .await
    }

    #[method(public)]
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

    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::DataBase;

    #[test]
    fn test_database_metadata_and_record_selection() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "storage-handset").await?.into();
            let database: ClassInstanceRef<DataBase> = jvm
                .invoke_static(
                    "org/kwis/msp/db/DataBase",
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    (name, 32, true),
                )
                .await?;

            let database_name: ClassInstanceRef<String> = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "getDataBaseName", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &database_name).await?.as_str(), "storage-handset");

            let record_size: i32 = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "getRecordSize", "()I", ())
                .await?;
            assert_eq!(record_size, 32);

            let mut record = jvm.instantiate_array("B", 3).await?;
            jvm.store_array(&mut record, 0, [1i8, 2, 3]).await?;
            let record_id: i32 = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "insertRecord", "([B)I", (record,))
                .await?;

            let mut buffer = jvm.instantiate_array("B", 5).await?;
            jvm.store_array(&mut buffer, 0, [9i8; 5]).await?;
            let _: () = jvm
                .invoke_virtual(
                    &database,
                    "org/kwis/msp/db/DataBase",
                    "selectRecord",
                    "(I[BI)V",
                    (record_id, buffer.clone(), 1),
                )
                .await?;
            assert_eq!(jvm.load_array::<i8>(&buffer, 0, 5).await?, [9i8, 1, 2, 3, 9]);

            let short_buffer = jvm.instantiate_array("B", 3).await?;
            let short_result: JvmResult<()> = jvm
                .invoke_virtual(
                    &database,
                    "org/kwis/msp/db/DataBase",
                    "selectRecord",
                    "(I[BI)V",
                    (record_id, short_buffer, 1),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = short_result else {
                panic!("selectRecord accepted a buffer smaller than the record");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let missing_result: JvmResult<()> = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "selectRecord", "(I[BI)V", (99, buffer, 0))
                .await;
            let Err(JavaError::JavaException(exception)) = missing_result else {
                panic!("selectRecord accepted an unknown record ID");
            };
            assert!(jvm.is_instance(&*exception, "org/kwis/msp/db/DataBaseRecordException"));

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
            let first_id: i32 = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "insertRecord", "([B)I", (first,))
                .await?;

            let mut second = jvm.instantiate_array("B", 2).await?;
            jvm.store_array(&mut second, 0, [3i8, 4]).await?;
            let second_id: i32 = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "insertRecord", "([B)I", (second,))
                .await?;
            assert_eq!((first_id, second_id), (0, 1));

            let count: i32 = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "getNumberOfRecords", "()I", ())
                .await?;
            assert_eq!(count, 2);

            let _: () = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "deleteRecord", "(I)V", (first_id,))
                .await?;
            let count: i32 = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "getNumberOfRecords", "()I", ())
                .await?;
            assert_eq!(count, 1);

            let deleted: JvmResult<ClassInstanceRef<Array<i8>>> = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "selectRecord", "(I)[B", (first_id,))
                .await;
            let Err(JavaError::JavaException(exception)) = deleted else {
                panic!("deleted WIPI record lookup succeeded");
            };
            assert!(jvm.is_instance(&*exception, "org/kwis/msp/db/DataBaseRecordException"));

            let remaining: ClassInstanceRef<Array<i8>> = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "selectRecord", "(I)[B", (second_id,))
                .await?;
            assert_eq!(jvm.load_array::<i8>(&remaining, 0, 2).await?, [3i8, 4]);

            let unknown: JvmResult<()> = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "deleteRecord", "(I)V", (99,))
                .await;
            let Err(JavaError::JavaException(exception)) = unknown else {
                panic!("unknown WIPI record deletion succeeded");
            };
            assert!(jvm.is_instance(&*exception, "org/kwis/msp/db/DataBaseRecordException"));

            let count: i32 = jvm
                .invoke_virtual(&database, "org/kwis/msp/db/DataBase", "getNumberOfRecords", "()I", ())
                .await?;
            assert_eq!(count, 1);

            Ok(())
        })
    }
}
