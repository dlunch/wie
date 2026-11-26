use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::{Command, CommandListener, Displayable, Item, ItemCommandListener};

// class net.wie.CommandEvent
#[java_class(name = "net/wie/CommandEvent", implements("java/lang/Runnable"), public)]
pub struct CommandEvent {
    #[field(private)]
    command: ClassInstanceRef<Command>,
    #[field(private)]
    listener: ClassInstanceRef<CommandListener>,
    #[field(private)]
    displayable: ClassInstanceRef<Displayable>,
    #[field(private)]
    itemListener: ClassInstanceRef<ItemCommandListener>,
    #[field(private)]
    item: ClassInstanceRef<Item>,
}

#[java_methods]
impl CommandEvent {
    #[method(constructor, public)]
    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<CommandListener>,
        command: ClassInstanceRef<Command>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "net/wie/CommandEvent",
            "command",
            "Ljavax/microedition/lcdui/Command;",
            command,
        )
        .await?;
        jvm.put_field(
            &mut this,
            "net/wie/CommandEvent",
            "listener",
            "Ljavax/microedition/lcdui/CommandListener;",
            listener,
        )
        .await?;
        jvm.put_field(
            &mut this,
            "net/wie/CommandEvent",
            "displayable",
            "Ljavax/microedition/lcdui/Displayable;",
            displayable,
        )
        .await
    }

    #[method(constructor, public)]
    async fn init_item(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ItemCommandListener>,
        command: ClassInstanceRef<Command>,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "net/wie/CommandEvent",
            "command",
            "Ljavax/microedition/lcdui/Command;",
            command,
        )
        .await?;
        jvm.put_field(
            &mut this,
            "net/wie/CommandEvent",
            "itemListener",
            "Ljavax/microedition/lcdui/ItemCommandListener;",
            listener,
        )
        .await?;
        jvm.put_field(&mut this, "net/wie/CommandEvent", "item", "Ljavax/microedition/lcdui/Item;", item)
            .await
    }

    #[method(public)]
    async fn run(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let command: ClassInstanceRef<Command> = jvm
            .get_field(&this, "net/wie/CommandEvent", "command", "Ljavax/microedition/lcdui/Command;")
            .await?;
        let item: ClassInstanceRef<Item> = jvm
            .get_field(&this, "net/wie/CommandEvent", "item", "Ljavax/microedition/lcdui/Item;")
            .await?;
        if item.is_null() {
            let listener: ClassInstanceRef<CommandListener> = jvm
                .get_field(&this, "net/wie/CommandEvent", "listener", "Ljavax/microedition/lcdui/CommandListener;")
                .await?;
            let displayable: ClassInstanceRef<Displayable> = jvm
                .get_field(&this, "net/wie/CommandEvent", "displayable", "Ljavax/microedition/lcdui/Displayable;")
                .await?;
            jvm.invoke_virtual(
                &listener,
                "javax/microedition/lcdui/CommandListener",
                "commandAction",
                "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                (command, displayable),
            )
            .await
        } else {
            let listener: ClassInstanceRef<ItemCommandListener> = jvm
                .get_field(
                    &this,
                    "net/wie/CommandEvent",
                    "itemListener",
                    "Ljavax/microedition/lcdui/ItemCommandListener;",
                )
                .await?;
            jvm.invoke_virtual(
                &listener,
                "javax/microedition/lcdui/ItemCommandListener",
                "commandAction",
                "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Item;)V",
                (command, item),
            )
            .await
        }
    }
}
