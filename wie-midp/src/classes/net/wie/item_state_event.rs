use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Form, Item};

// class net.wie.ItemStateEvent
pub struct ItemStateEvent;

impl ItemStateEvent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/ItemStateEvent",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["java/lang/Runnable"],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/Form;Ljavax/microedition/lcdui/Item;)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("run", "()V", Self::run, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("form", "Ljavax/microedition/lcdui/Form;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("item", "Ljavax/microedition/lcdui/Item;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        form: ClassInstanceRef<Form>,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "form", "Ljavax/microedition/lcdui/Form;", form).await?;
        jvm.put_field(&mut this, "item", "Ljavax/microedition/lcdui/Item;", item).await
    }

    async fn run(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let form: ClassInstanceRef<Form> = jvm.get_field(&this, "form", "Ljavax/microedition/lcdui/Form;").await?;
        let item: ClassInstanceRef<Item> = jvm.get_field(&this, "item", "Ljavax/microedition/lcdui/Item;").await?;
        jvm.invoke_virtual(
            &form,
            "javax/microedition/lcdui/Form",
            "dispatchItemStateChanged",
            "(Ljavax/microedition/lcdui/Item;)V",
            (item,),
        )
        .await
    }
}
