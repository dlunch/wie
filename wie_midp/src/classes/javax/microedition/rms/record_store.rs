use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

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
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("addRecord", "([BII)I", Self::add_record, Default::default()),
                JavaMethodProto::new("setRecord", "(I[BII)V", Self::set_record, Default::default()),
                JavaMethodProto::new("getNumRecords", "()I", Self::get_num_records, Default::default()),
                JavaMethodProto::new("closeRecordStore", "()V", Self::close_record_store, Default::default()),
                JavaMethodProto::new(
                    "openRecordStore",
                    "(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;",
                    Self::open_record_store,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn add_record(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::warn!(
            "stub javax.microedition.rms.RecordStore::addRecord({:?}, {:?}, {}, {})",
            &this,
            &data,
            offset,
            length
        );

        Ok(0)
    }

    async fn set_record(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record_id: i32,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub javax.microedition.rms.RecordStore::setRecord({:?}, {}, {:?}, {}, {})",
            &this,
            record_id,
            &data,
            offset,
            length
        );

        Ok(())
    }

    async fn get_num_records(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.rms.RecordStore::getNumRecords({:?})", &this);

        Ok(0)
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
        tracing::warn!("stub javax.microedition.rms.RecordStore::openRecordStore({:?}, {:?})", name, create);

        let store = jvm.new_class("javax/microedition/rms/RecordStore", "()V", ()).await?;

        Ok(store.into())
    }
}
