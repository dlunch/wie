use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::{Form, Item};

// class net.wie.ItemStateEvent
#[java_class(name = "net/wie/ItemStateEvent", implements("java/lang/Runnable"), public)]
pub struct ItemStateEvent {
    #[field(private)]
    form: ClassInstanceRef<Form>,
    #[field(private)]
    item: ClassInstanceRef<Item>,
}

#[java_methods]
impl ItemStateEvent {
    #[method(constructor, public)]
    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        form: ClassInstanceRef<Form>,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "net/wie/ItemStateEvent", "form", "Ljavax/microedition/lcdui/Form;", form)
            .await?;
        jvm.put_field(&mut this, "net/wie/ItemStateEvent", "item", "Ljavax/microedition/lcdui/Item;", item)
            .await
    }

    #[method(public)]
    async fn run(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let form: ClassInstanceRef<Form> = jvm
            .get_field(&this, "net/wie/ItemStateEvent", "form", "Ljavax/microedition/lcdui/Form;")
            .await?;
        let item: ClassInstanceRef<Item> = jvm
            .get_field(&this, "net/wie/ItemStateEvent", "item", "Ljavax/microedition/lcdui/Item;")
            .await?;
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
