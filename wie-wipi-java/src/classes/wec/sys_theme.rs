use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class wec.SYSTheme
pub struct SYSTheme;

impl SYSTheme {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/SYSTheme",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::clinit, MethodAccessFlags::STATIC),
                JavaMethodProto::new("saveItem", "(IILjava/lang/String;[BI)I", Self::save_item, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("ITEM_GROUP_SOUND_DEFAULT", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                // Native imports link this field eagerly; no format is exposed without a theme provider.
                JavaFieldProto::new("FORMAT_SOUND_MA3", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn clinit(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("wec.SYSTheme::<clinit>()");
        jvm.put_static_field("wec/SYSTheme", "ITEM_GROUP_SOUND_DEFAULT", "I", 1).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn save_item(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        group: i32,
        format: i32,
        name: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub wec.SYSTheme::saveItem({this:?}, {group}, {format}, {name:?}, {data:?}, {length})");
        Ok(-1)
    }
}
