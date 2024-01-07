use alloc::boxed::Box;

use wie_backend::System;
use wie_core_jvm::JvmCore;

use crate::TestPlatform;

pub fn test_jvm_core() -> JvmCore {
    let system_handle = System::new(Box::new(TestPlatform)).handle();

    JvmCore::new(&system_handle)
}
