use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface org.kwis.msp.db.DataFilter
#[java_class(name = "org/kwis/msp/db/DataFilter", public, interface, abstract)]
pub struct DataFilter;

#[java_methods]
impl DataFilter {
    #[method(public)]
    async fn filter(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, data: ClassInstanceRef<Array<i8>>) -> JvmResult<bool>;
}
