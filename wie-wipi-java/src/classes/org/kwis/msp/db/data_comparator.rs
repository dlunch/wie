use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface org.kwis.msp.db.DataComparator
#[java_class(name = "org/kwis/msp/db/DataComparator", public, interface, abstract)]
pub struct DataComparator;

#[java_methods]
impl DataComparator {
    #[method(public)]
    async fn compare(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data1: ClassInstanceRef<Array<i8>>,
        data2: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<i32>;
}
