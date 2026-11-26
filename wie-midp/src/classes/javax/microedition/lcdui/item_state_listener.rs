use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::Item;

// interface javax.microedition.lcdui.ItemStateListener
#[java_class(name = "javax/microedition/lcdui/ItemStateListener", public, interface, abstract)]
pub struct ItemStateListener;

#[java_methods]
impl ItemStateListener {
    #[method(public)]
    async fn item_state_changed(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, item: ClassInstanceRef<Item>) -> Result<()>;
}
