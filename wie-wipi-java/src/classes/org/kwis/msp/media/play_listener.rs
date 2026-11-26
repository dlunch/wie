use jvm_class_proto::{java_class, java_methods};

// interface org.kwis.msp.media.PlayListener
#[java_class(name = "org/kwis/msp/media/PlayListener", public, interface, abstract)]
pub struct PlayListener;

#[java_methods]
impl PlayListener {}
