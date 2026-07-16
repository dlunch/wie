use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.db.DataComparator
pub struct DataComparator;

impl DataComparator {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataComparator",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract("compare", "([B[B)I", MethodAccessFlags::ABSTRACT)],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
