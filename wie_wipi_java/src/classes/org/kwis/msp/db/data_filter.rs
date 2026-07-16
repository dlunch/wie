use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.db.DataFilter
pub struct DataFilter;

impl DataFilter {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataFilter",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract("filter", "([B)Z", MethodAccessFlags::ABSTRACT)],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
