use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::{Command, Item};

// interface javax.microedition.lcdui.ItemCommandListener
#[java_class(name = "javax/microedition/lcdui/ItemCommandListener", public, interface, abstract)]
pub struct ItemCommandListener;

#[java_methods]
impl ItemCommandListener {
    #[method(public)]
    async fn command_action(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
        item: ClassInstanceRef<Item>,
    ) -> Result<()>;
}
