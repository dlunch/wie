use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::rms::{RecordComparator, RecordFilter, RecordStore};

// class net.wie.RecordEnumeration
pub struct RecordEnumeration;

impl RecordEnumeration {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/RecordEnumeration",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["javax/microedition/rms/RecordEnumeration"],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/rms/RecordStore;Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("numRecords", "()I", Self::num_records, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("nextRecord", "()[B", Self::next_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("nextRecordId", "()I", Self::next_record_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("previousRecord", "()[B", Self::previous_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("previousRecordId", "()I", Self::previous_record_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasNextElement", "()Z", Self::has_next_element, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasPreviousElement", "()Z", Self::has_previous_element, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("reset", "()V", Self::reset, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("rebuild", "()V", Self::rebuild, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keepUpdated", "(Z)V", Self::keep_updated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isKeptUpdated", "()Z", Self::is_kept_updated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("destroy", "()V", Self::destroy, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("store", "Ljavax/microedition/rms/RecordStore;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("filter", "Ljavax/microedition/rms/RecordFilter;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("comparator", "Ljavax/microedition/rms/RecordComparator;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("ids", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("currentId", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("nextIndex", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("previousIndex", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("keepUpdated", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        store: ClassInstanceRef<RecordStore>,
        filter: ClassInstanceRef<RecordFilter>,
        comparator: ClassInstanceRef<RecordComparator>,
        keep_updated: bool,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumeration::<init>({this:?}, {store:?}, {filter:?}, {comparator:?}, {keep_updated})");
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "net/wie/RecordEnumeration",
            "store",
            "Ljavax/microedition/rms/RecordStore;",
            store,
        )
        .await?;
        jvm.put_field(
            &mut this,
            "net/wie/RecordEnumeration",
            "filter",
            "Ljavax/microedition/rms/RecordFilter;",
            filter,
        )
        .await?;
        jvm.put_field(
            &mut this,
            "net/wie/RecordEnumeration",
            "comparator",
            "Ljavax/microedition/rms/RecordComparator;",
            comparator,
        )
        .await?;
        jvm.put_field(&mut this, "net/wie/RecordEnumeration", "keepUpdated", "Z", keep_updated)
            .await?;
        let ids = jvm.instantiate_array("I", 0).await?;
        jvm.put_field(&mut this, "net/wie/RecordEnumeration", "ids", "[I", ids).await?;
        Self::refresh(jvm, context, &mut this, true).await?;
        Ok(())
    }

    async fn refresh(jvm: &Jvm, context: &mut WieJvmContext, this: &mut ClassInstanceRef<Self>, force: bool) -> JvmResult<()> {
        let store: ClassInstanceRef<RecordStore> = jvm
            .get_field(this, "net/wie/RecordEnumeration", "store", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        if store.is_null() {
            return Err(jvm.exception("java/lang/IllegalStateException", "Enumeration is destroyed").await);
        }
        let keep_updated: bool = jvm.get_field(this, "net/wie/RecordEnumeration", "keepUpdated", "Z").await?;
        if !force && !keep_updated {
            return Ok(());
        }

        let filter: ClassInstanceRef<RecordFilter> = jvm
            .get_field(this, "net/wie/RecordEnumeration", "filter", "Ljavax/microedition/rms/RecordFilter;")
            .await?;
        let comparator: ClassInstanceRef<RecordComparator> = jvm
            .get_field(
                this,
                "net/wie/RecordEnumeration",
                "comparator",
                "Ljavax/microedition/rms/RecordComparator;",
            )
            .await?;
        let database = RecordStore::get_database(jvm, context, &store).await?;
        let mut records: Vec<(i32, ClassInstanceRef<Array<i8>>)> = Vec::new();
        for id in database.get_record_ids().await {
            let data: ClassInstanceRef<Array<i8>> = if filter.is_null() && comparator.is_null() {
                None.into()
            } else {
                jvm.invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (id as i32,))
                    .await?
            };
            if !filter.is_null()
                && !jvm
                    .invoke_virtual::<_, bool>(&filter, "javax/microedition/rms/RecordFilter", "matches", "([B)Z", (data.clone(),))
                    .await?
            {
                continue;
            }
            records.push((id as i32, data));
            if !comparator.is_null() {
                let mut index = records.len() - 1;
                while index > 0 {
                    let order: i32 = jvm
                        .invoke_virtual(
                            &comparator,
                            "javax/microedition/rms/RecordComparator",
                            "compare",
                            "([B[B)I",
                            (records[index - 1].1.clone(), records[index].1.clone()),
                        )
                        .await?;
                    if order <= 0 {
                        break;
                    }
                    records.swap(index - 1, index);
                    index -= 1;
                }
            }
        }
        let ids: Vec<i32> = records.into_iter().map(|(id, _)| id).collect();
        let old_array: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "net/wie/RecordEnumeration", "ids", "[I").await?;
        let old_ids: Vec<i32> = jvm.load_array(&old_array, 0, jvm.array_length(&old_array).await?).await?;
        let current: i32 = jvm.get_field(this, "net/wie/RecordEnumeration", "currentId", "I").await?;
        let (next, previous) = if current == 0 {
            (0, ids.len() as i32 - 1)
        } else if let Some(index) = ids.iter().position(|id| *id == current) {
            (index as i32 + 1, index as i32 - 1)
        } else {
            let next: i32 = jvm.get_field(this, "net/wie/RecordEnumeration", "nextIndex", "I").await?;
            let previous: i32 = jvm.get_field(this, "net/wie/RecordEnumeration", "previousIndex", "I").await?;
            let after = old_ids.iter().skip(next as usize).find_map(|old| ids.iter().position(|id| id == old));
            let before = old_ids
                .iter()
                .take((previous + 1) as usize)
                .rev()
                .find_map(|old| ids.iter().position(|id| id == old));
            // A deleted current record leaves a gap; retain both neighboring directions.
            (
                after.map_or_else(|| before.map_or(0, |index| index as i32 + 1), |index| index as i32),
                before.map_or_else(|| after.map_or(-1, |index| index as i32 - 1), |index| index as i32),
            )
        };
        if ids != old_ids {
            let mut array = jvm.instantiate_array("I", ids.len()).await?;
            jvm.store_array(&mut array, 0, ids).await?;
            jvm.put_field(this, "net/wie/RecordEnumeration", "ids", "[I", array).await?;
        }
        jvm.put_field(this, "net/wie/RecordEnumeration", "nextIndex", "I", next).await?;
        jvm.put_field(this, "net/wie/RecordEnumeration", "previousIndex", "I", previous).await?;
        Ok(())
    }

    async fn num_records(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("net.wie.RecordEnumeration::numRecords({this:?})");
        Self::refresh(jvm, context, &mut this, false).await?;
        let ids: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "net/wie/RecordEnumeration", "ids", "[I").await?;
        Ok(jvm.array_length(&ids).await? as i32)
    }

    async fn step(jvm: &Jvm, context: &mut WieJvmContext, this: &mut ClassInstanceRef<Self>, forward: bool) -> JvmResult<i32> {
        Self::refresh(jvm, context, this, false).await?;
        let ids: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "net/wie/RecordEnumeration", "ids", "[I").await?;
        let index: i32 = jvm
            .get_field(
                this,
                "net/wie/RecordEnumeration",
                if forward { "nextIndex" } else { "previousIndex" },
                "I",
            )
            .await?;
        if index < 0 || index as usize >= jvm.array_length(&ids).await? {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "No more records").await);
        }
        let id = jvm.load_array::<i32>(&ids, index as usize, 1).await?[0];
        jvm.put_field(this, "net/wie/RecordEnumeration", "currentId", "I", id).await?;
        jvm.put_field(this, "net/wie/RecordEnumeration", "nextIndex", "I", index + 1).await?;
        jvm.put_field(this, "net/wie/RecordEnumeration", "previousIndex", "I", index - 1).await?;
        Ok(id)
    }

    async fn next_record_id(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("net.wie.RecordEnumeration::nextRecordId({this:?})");
        Self::step(jvm, context, &mut this, true).await
    }

    async fn previous_record_id(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("net.wie.RecordEnumeration::previousRecordId({this:?})");
        Self::step(jvm, context, &mut this, false).await
    }

    async fn next_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("net.wie.RecordEnumeration::nextRecord({this:?})");
        let id: i32 = jvm.invoke_virtual(&this, "net/wie/RecordEnumeration", "nextRecordId", "()I", ()).await?;
        let store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "net/wie/RecordEnumeration", "store", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        jvm.invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (id,))
            .await
    }

    async fn previous_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("net.wie.RecordEnumeration::previousRecord({this:?})");
        let id: i32 = jvm
            .invoke_virtual(&this, "net/wie/RecordEnumeration", "previousRecordId", "()I", ())
            .await?;
        let store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "net/wie/RecordEnumeration", "store", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        jvm.invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (id,))
            .await
    }

    async fn has_next_element(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("net.wie.RecordEnumeration::hasNextElement({this:?})");
        Self::refresh(jvm, context, &mut this, false).await?;
        let ids: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "net/wie/RecordEnumeration", "ids", "[I").await?;
        let next: i32 = jvm.get_field(&this, "net/wie/RecordEnumeration", "nextIndex", "I").await?;
        Ok((next as usize) < jvm.array_length(&ids).await?)
    }

    async fn has_previous_element(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("net.wie.RecordEnumeration::hasPreviousElement({this:?})");
        Self::refresh(jvm, context, &mut this, false).await?;
        let previous: i32 = jvm.get_field(&this, "net/wie/RecordEnumeration", "previousIndex", "I").await?;
        Ok(previous >= 0)
    }

    async fn reset(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumeration::reset({this:?})");
        Self::refresh(jvm, context, &mut this, false).await?;
        let ids: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "net/wie/RecordEnumeration", "ids", "[I").await?;
        jvm.put_field(&mut this, "net/wie/RecordEnumeration", "currentId", "I", 0).await?;
        jvm.put_field(&mut this, "net/wie/RecordEnumeration", "nextIndex", "I", 0).await?;
        jvm.put_field(
            &mut this,
            "net/wie/RecordEnumeration",
            "previousIndex",
            "I",
            jvm.array_length(&ids).await? as i32 - 1,
        )
        .await
    }

    async fn rebuild(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumeration::rebuild({this:?})");
        jvm.put_field(&mut this, "net/wie/RecordEnumeration", "currentId", "I", 0).await?;
        Self::refresh(jvm, context, &mut this, true).await?;
        Ok(())
    }

    async fn keep_updated(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, keep_updated: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumeration::keepUpdated({this:?}, {keep_updated})");
        if keep_updated {
            let _: () = jvm.invoke_virtual(&this, "net/wie/RecordEnumeration", "rebuild", "()V", ()).await?;
        } else {
            Self::refresh(jvm, context, &mut this, false).await?;
        }
        jvm.put_field(&mut this, "net/wie/RecordEnumeration", "keepUpdated", "Z", keep_updated)
            .await
    }

    async fn is_kept_updated(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("net.wie.RecordEnumeration::isKeptUpdated({this:?})");
        let store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "net/wie/RecordEnumeration", "store", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        if store.is_null() {
            return Err(jvm.exception("java/lang/IllegalStateException", "Enumeration is destroyed").await);
        }
        jvm.get_field(&this, "net/wie/RecordEnumeration", "keepUpdated", "Z").await
    }

    async fn destroy(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumeration::destroy({this:?})");
        let store: ClassInstanceRef<RecordStore> = jvm
            .get_field(&this, "net/wie/RecordEnumeration", "store", "Ljavax/microedition/rms/RecordStore;")
            .await?;
        if store.is_null() {
            return Err(jvm.exception("java/lang/IllegalStateException", "Enumeration is destroyed").await);
        }
        for (field, descriptor) in [
            ("store", "Ljavax/microedition/rms/RecordStore;"),
            ("filter", "Ljavax/microedition/rms/RecordFilter;"),
            ("comparator", "Ljavax/microedition/rms/RecordComparator;"),
            ("ids", "[I"),
        ] {
            jvm.put_field(&mut this, "net/wie/RecordEnumeration", field, descriptor, None).await?;
        }
        Ok(())
    }
}
