use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::Image;

// class javax.microedition.lcdui.List
pub struct List;

impl List {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/List",
            parent_class: Some("javax/microedition/lcdui/Screen"),
            interfaces: vec!["javax/microedition/lcdui/Choice"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                    Self::init_with_elements,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>, list_type: i32) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.List::<init>({this:?}, {title:?}, {list_type})");

        jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await
    }

    async fn init_with_elements(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        list_type: i32,
        strings: ClassInstanceRef<Array<String>>,
        images: ClassInstanceRef<Array<Image>>,
    ) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.List::<init>({this:?}, {title:?}, {list_type}, {strings:?}, {images:?})");

        jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await
    }
}
