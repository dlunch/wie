use jvm::{Array, ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface javax.microedition.rms.RecordEnumeration
#[java_class(name = "javax/microedition/rms/RecordEnumeration", public, interface, abstract)]
pub struct RecordEnumeration;

#[java_methods]
impl RecordEnumeration {
    #[method(public)]
    async fn num_records(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32>;

    #[method(public)]
    async fn next_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<Array<i8>>>;

    #[method(public)]
    async fn next_record_id(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32>;

    #[method(public)]
    async fn previous_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<Array<i8>>>;

    #[method(public)]
    async fn previous_record_id(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32>;

    #[method(public)]
    async fn has_next_element(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<bool>;

    #[method(public)]
    async fn has_previous_element(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<bool>;

    #[method(public)]
    async fn reset(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn rebuild(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn keep_updated(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, keep_updated: bool) -> Result<()>;

    #[method(public)]
    async fn is_kept_updated(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<bool>;

    #[method(public)]
    async fn destroy(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;
}
