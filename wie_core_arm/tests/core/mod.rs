use wie_backend::System;
use wie_core_arm::ArmCore;

use test_utils::TestPlatform;

pub fn test_core() -> ArmCore {
    ArmCore::new(System::new(Box::new(TestPlatform)).handle()).unwrap()
}
