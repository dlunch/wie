use wie_backend::System;
use wie_core_jvm::JvmCore;

use test_utils::TestPlatform;

pub fn test_core() -> JvmCore {
    let system_handle = System::new(Box::new(TestPlatform)).handle();

    JvmCore::new(&system_handle)
}
