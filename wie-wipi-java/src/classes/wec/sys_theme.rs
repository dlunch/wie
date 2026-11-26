use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class wec.SYSTheme
#[java_class(name = "wec/SYSTheme", public)]
pub struct SYSTheme {
    #[field(public, static)]
    ITEM_GROUP_SOUND_DEFAULT: i32,
    #[field(public, static)]
    FORMAT_SOUND_MA3: i32,
}

#[java_methods]
impl SYSTheme {
    #[method(static_initializer)]
    async fn clinit(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("wec.SYSTheme::<clinit>()");
        jvm.put_static_field("wec/SYSTheme", "ITEM_GROUP_SOUND_DEFAULT", "I", 1).await
    }

    #[method(public)]
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
