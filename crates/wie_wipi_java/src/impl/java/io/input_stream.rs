use alloc::vec;

use crate::base::JavaClassProto;

// class java.io.InputStream
pub struct InputStream {}

impl InputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
