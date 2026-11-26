use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::{Command, Displayable};

// interface javax.microedition.lcdui.CommandListener
#[java_class(name = "javax/microedition/lcdui/CommandListener", public, interface, abstract)]
pub struct CommandListener;

#[java_methods]
impl CommandListener {
    #[method(public)]
    async fn command_action(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> Result<()>;
}
