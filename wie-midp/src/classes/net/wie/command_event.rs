use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Command, CommandListener, Displayable, Item, ItemCommandListener};

// class net.wie.CommandEvent
pub struct CommandEvent;

impl CommandEvent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/CommandEvent",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["java/lang/Runnable"],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/CommandListener;Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/ItemCommandListener;Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Item;)V",
                    Self::init_item,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("run", "()V", Self::run, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("command", "Ljavax/microedition/lcdui/Command;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("listener", "Ljavax/microedition/lcdui/CommandListener;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("displayable", "Ljavax/microedition/lcdui/Displayable;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(
                    "itemListener",
                    "Ljavax/microedition/lcdui/ItemCommandListener;",
                    FieldAccessFlags::PRIVATE,
                ),
                JavaFieldProto::new("item", "Ljavax/microedition/lcdui/Item;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<CommandListener>,
        command: ClassInstanceRef<Command>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "command", "Ljavax/microedition/lcdui/Command;", command).await?;
        jvm.put_field(&mut this, "listener", "Ljavax/microedition/lcdui/CommandListener;", listener)
            .await?;
        jvm.put_field(&mut this, "displayable", "Ljavax/microedition/lcdui/Displayable;", displayable)
            .await
    }

    async fn init_item(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ItemCommandListener>,
        command: ClassInstanceRef<Command>,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "command", "Ljavax/microedition/lcdui/Command;", command).await?;
        jvm.put_field(&mut this, "itemListener", "Ljavax/microedition/lcdui/ItemCommandListener;", listener)
            .await?;
        jvm.put_field(&mut this, "item", "Ljavax/microedition/lcdui/Item;", item).await
    }

    async fn run(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let command: ClassInstanceRef<Command> = jvm.get_field(&this, "command", "Ljavax/microedition/lcdui/Command;").await?;
        let item: ClassInstanceRef<Item> = jvm.get_field(&this, "item", "Ljavax/microedition/lcdui/Item;").await?;
        if item.is_null() {
            let listener: ClassInstanceRef<CommandListener> = jvm.get_field(&this, "listener", "Ljavax/microedition/lcdui/CommandListener;").await?;
            let displayable: ClassInstanceRef<Displayable> = jvm.get_field(&this, "displayable", "Ljavax/microedition/lcdui/Displayable;").await?;
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
                .get_field(&this, "itemListener", "Ljavax/microedition/lcdui/ItemCommandListener;")
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
