use jvm_class_proto::{java_class, java_methods};

// abstract class org.kwis.msp.lcdui.JletWrapper
#[java_class(name = "org/kwis/msp/lcdui/JletWrapper", extends = "org/kwis/msp/lcdui/Jlet", public, abstract)]
pub struct JletWrapper;

#[java_methods]
impl JletWrapper {}
