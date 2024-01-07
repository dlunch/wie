use alloc::boxed::Box;

use wie_core_arm::ArmCore;

use crate::TestPlatform;

pub fn test_arm_core() -> ArmCore {
    ArmCore::new(wie_backend::System::new(Box::new(TestPlatform)).handle()).unwrap()
}
