use wie_core_arm::SvcId;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum InitSvcId {
    GetImportTable = 0,
    GetImportFunction = 1,
    Unk0 = 2,
    JavaUnk7 = 3,
    JavaUnk1 = 4,
    JavaUnk2 = 5,
    JavaUnk3 = 6,
    JavaInterfaceUnk0 = 7,
    JavaInterfaceUnk12 = 8,
    JavaInterfaceUnk5 = 9,
    JavaLoadClasses = 10,
    JavaUnk9 = 11,
    JavaUnk11 = 12,
}

impl TryFrom<SvcId> for InitSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0 => Self::GetImportTable,
            1 => Self::GetImportFunction,
            2 => Self::Unk0,
            3 => Self::JavaUnk7,
            4 => Self::JavaUnk1,
            5 => Self::JavaUnk2,
            6 => Self::JavaUnk3,
            7 => Self::JavaInterfaceUnk0,
            8 => Self::JavaInterfaceUnk12,
            9 => Self::JavaInterfaceUnk5,
            10 => Self::JavaLoadClasses,
            11 => Self::JavaUnk9,
            12 => Self::JavaUnk11,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown LGT init SVC id {}", value.0))),
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum StdlibSvcId {
    Unk2 = 0x3f6,
    Atoi = 0x3fb,
    Strcpy = 0x405,
    Strncpy = 0x406,
    Strcat = 0x407,
    Strcmp = 0x409,
    Unk4 = 0x40a,
    Unk5 = 0x410,
    Strlen = 0x411,
    Memcpy = 0x414,
    Memset = 0x418,
    Localtime = 0x420,
    Unk3 = 0x424,
}
