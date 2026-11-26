use jvm::{Array, ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface javax.microedition.rms.RecordFilter
#[java_class(name = "javax/microedition/rms/RecordFilter", public, interface, abstract)]
pub struct RecordFilter;

#[java_methods]
impl RecordFilter {
    #[method(public)]
    async fn matches(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, candidate: ClassInstanceRef<Array<i8>>) -> Result<bool>;
}
