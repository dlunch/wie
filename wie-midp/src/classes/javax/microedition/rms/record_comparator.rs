use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface javax.microedition.rms.RecordComparator
#[java_class(name = "javax/microedition/rms/RecordComparator", public, interface, abstract)]
pub struct RecordComparator {
    #[field(public, static, final)]
    PRECEDES: i32,
    #[field(public, static, final)]
    EQUIVALENT: i32,
    #[field(public, static, final)]
    FOLLOWS: i32,
}

#[java_methods]
impl RecordComparator {
    #[method(public)]
    async fn compare(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        record1: ClassInstanceRef<Array<i8>>,
        record2: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<i32>;

    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.rms.RecordComparator::<clinit>()");
        for (name, value) in [("PRECEDES", -1), ("EQUIVALENT", 0), ("FOLLOWS", 1)] {
            jvm.put_static_field("javax/microedition/rms/RecordComparator", name, "I", value).await?;
        }
        Ok(())
    }
}
