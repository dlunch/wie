use wie_core_arm::SvcId;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum InitSvcId {
    GetInterface = 0,
    JavaThrow = 1,
    JavaCheckType = 2,
    JavaNew = 3,
    JavaArrayNew = 4,
    JavaClassLoad = 5,
    Alloc = 6,
}

impl TryFrom<SvcId> for InitSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0 => Self::GetInterface,
            1 => Self::JavaThrow,
            2 => Self::JavaCheckType,
            3 => Self::JavaNew,
            4 => Self::JavaArrayNew,
            5 => Self::JavaClassLoad,
            6 => Self::Alloc,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown KTF init SVC id {}", value.0))),
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum JavaSvcId {
    JavaJump1 = 7,
    JavaJump2 = 8,
    JavaJump3 = 9,
    GetJavaMethod = 10,
    GetField = 11,
    JbUnk4 = 12,
    JbUnk5 = 13,
    JbUnk7 = 14,
    JbUnk8 = 15,
    RegisterClass = 16,
    RegisterJavaString = 17,
    CallNative = 18,
}

impl TryFrom<SvcId> for JavaSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            7 => Self::JavaJump1,
            8 => Self::JavaJump2,
            9 => Self::JavaJump3,
            10 => Self::GetJavaMethod,
            11 => Self::GetField,
            12 => Self::JbUnk4,
            13 => Self::JbUnk5,
            14 => Self::JbUnk7,
            15 => Self::JbUnk8,
            16 => Self::RegisterClass,
            17 => Self::RegisterJavaString,
            18 => Self::CallNative,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown KTF Java SVC id {}", value.0))),
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum WIPICTableId {
    Kernel = 0,
    Util = 1,
    Misc = 2,
    Graphics = 3,
    Interface3 = 4,
    Interface4 = 5,
    Interface5 = 6,
    Database = 7,
    Interface7 = 8,
    Uic = 9,
    Media = 10,
    Net = 11,
    Interface11 = 12,
    Interface12 = 13,
    Interface13 = 14,
    Interface14 = 15,
    Interface15 = 16,
    Interface16 = 17,
}

impl WIPICTableId {
    pub const fn function_id(self, method_id: u16) -> u32 {
        ((self as u32) << 16) | method_id as u32
    }
}
