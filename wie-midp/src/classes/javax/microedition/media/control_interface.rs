use jvm_class_proto::{java_class, java_methods};

// interface javax.microedition.media.Control
#[java_class(name = "javax/microedition/media/Control", public, interface, abstract)]
pub struct Control;

#[java_methods]
impl Control {}
