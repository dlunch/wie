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

impl From<InitSvcId> for u32 {
    fn from(value: InitSvcId) -> Self {
        value as u32
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum WIPICSvcId {
    CletRegister = 0x03,
    GetFramebufferPointer = 0x32,
    GetFramebufferWidth = 0x33,
    GetFramebufferHeight = 0x34,
    GetFramebufferBpl = 0x35,
    GetFramebufferBpp = 0x36,
    Printk = 0x64,
    Sprintk = 0x65,
    Unk13 = 0x68,
    Unk1 = 0x6a,
    Exit = 0x6b,
    Alloc = 0x75,
    Calloc = 0x76,
    Free = 0x77,
    GetTotalMemory = 0x78,
    GetFreeMemory = 0x79,
    DefTimer = 0x7a,
    SetTimer = 0x7b,
    UnsetTimer = 0x7c,
    CurrentTime = 0x7d,
    GetSystemProperty = 0x7e,
    SetSystemProperty = 0x7f,
    GetResourceId = 0x80,
    GetResource = 0x81,
    Unk2 = 0x97,
    GetImageProperty = 0xc8,
    GetScreenFramebuffer = 0xca,
    DestroyOffscreenFramebuffer = 0xcb,
    CreateOffscreenFramebuffer = 0xcc,
    InitContext = 0xcd,
    SetContext = 0xce,
    PutPixel = 0xd0,
    DrawRect = 0xd2,
    FillRect = 0xd3,
    CopyFrameBuffer = 0xd4,
    DrawImage = 0xd5,
    DrawString = 0xda,
    FlushLcd = 0xde,
    GetPixelFromRgb = 0xdf,
    GetRgbFromPixel = 0xe0,
    GetDisplayInfo = 0xe1,
    Repaint = 0xe2,
    GetFont = 0xe3,
    GetFontHeight = 0xe4,
    CreateImage = 0xe9,
    Unk0 = 0xeb,
    Unk11 = 0xee,
    Unk3 = 0x12c,
    Unk4 = 0x12d,
    Unk7 = 0x12e,
    Unk6 = 0x12f,
    OpenDatabase = 0x190,
    ReadRecordSingle = 0x191,
    WriteRecordSingle = 0x192,
    CloseDatabase = 0x193,
    Unk12 = 0x194,
    Unk9 = 0x195,
    Unk8 = 0x1a0,
    Connect = 0x258,
    Close = 0x259,
    SocketClose = 0x25e,
    ClipCreate = 0x4b0,
    ClipFree = 0x4b1,
    ClipPutData = 0x4b3,
    Unk15 = 0x4b6,
    ClipGetVolume = 0x4b8,
    ClipSetVolume = 0x4b9,
    Play = 0x4ba,
    Stop = 0x4bd,
    Unk5 = 0x4c0,
    Vibrator = 0x4c1,
    Unk14 = 0x4c2,
    ClipAllocPlayer = 0x4c5,
    ClipFreePlayer = 0x4c6,
    Unk10 = 0x4ce,
    SetMuteState = 0x4d1,
    GetMuteState = 0x4d2,
    BackLight = 0x578,
}

impl TryFrom<SvcId> for WIPICSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0x03 => Self::CletRegister,
            0x32 => Self::GetFramebufferPointer,
            0x33 => Self::GetFramebufferWidth,
            0x34 => Self::GetFramebufferHeight,
            0x35 => Self::GetFramebufferBpl,
            0x36 => Self::GetFramebufferBpp,
            0x64 => Self::Printk,
            0x65 => Self::Sprintk,
            0x68 => Self::Unk13,
            0x6a => Self::Unk1,
            0x6b => Self::Exit,
            0x75 => Self::Alloc,
            0x76 => Self::Calloc,
            0x77 => Self::Free,
            0x78 => Self::GetTotalMemory,
            0x79 => Self::GetFreeMemory,
            0x7a => Self::DefTimer,
            0x7b => Self::SetTimer,
            0x7c => Self::UnsetTimer,
            0x7d => Self::CurrentTime,
            0x7e => Self::GetSystemProperty,
            0x7f => Self::SetSystemProperty,
            0x80 => Self::GetResourceId,
            0x81 => Self::GetResource,
            0x97 => Self::Unk2,
            0xc8 => Self::GetImageProperty,
            0xca => Self::GetScreenFramebuffer,
            0xcb => Self::DestroyOffscreenFramebuffer,
            0xcc => Self::CreateOffscreenFramebuffer,
            0xcd => Self::InitContext,
            0xce => Self::SetContext,
            0xd0 => Self::PutPixel,
            0xd2 => Self::DrawRect,
            0xd3 => Self::FillRect,
            0xd4 => Self::CopyFrameBuffer,
            0xd5 => Self::DrawImage,
            0xda => Self::DrawString,
            0xde => Self::FlushLcd,
            0xdf => Self::GetPixelFromRgb,
            0xe0 => Self::GetRgbFromPixel,
            0xe1 => Self::GetDisplayInfo,
            0xe2 => Self::Repaint,
            0xe3 => Self::GetFont,
            0xe4 => Self::GetFontHeight,
            0xe9 => Self::CreateImage,
            0xeb => Self::Unk0,
            0xee => Self::Unk11,
            0x12c => Self::Unk3,
            0x12d => Self::Unk4,
            0x12e => Self::Unk7,
            0x12f => Self::Unk6,
            0x190 => Self::OpenDatabase,
            0x191 => Self::ReadRecordSingle,
            0x192 => Self::WriteRecordSingle,
            0x193 => Self::CloseDatabase,
            0x194 => Self::Unk12,
            0x195 => Self::Unk9,
            0x1a0 => Self::Unk8,
            0x258 => Self::Connect,
            0x259 => Self::Close,
            0x25e => Self::SocketClose,
            0x4b0 => Self::ClipCreate,
            0x4b1 => Self::ClipFree,
            0x4b3 => Self::ClipPutData,
            0x4b6 => Self::Unk15,
            0x4b8 => Self::ClipGetVolume,
            0x4b9 => Self::ClipSetVolume,
            0x4ba => Self::Play,
            0x4bd => Self::Stop,
            0x4c0 => Self::Unk5,
            0x4c1 => Self::Vibrator,
            0x4c2 => Self::Unk14,
            0x4c5 => Self::ClipAllocPlayer,
            0x4c6 => Self::ClipFreePlayer,
            0x4ce => Self::Unk10,
            0x4d1 => Self::SetMuteState,
            0x4d2 => Self::GetMuteState,
            0x578 => Self::BackLight,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown LGT WIPIC SVC id {}", value.0))),
        })
    }
}

impl From<WIPICSvcId> for u32 {
    fn from(value: WIPICSvcId) -> Self {
        value as u32
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

impl From<StdlibSvcId> for u32 {
    fn from(value: StdlibSvcId) -> Self {
        value as u32
    }
}
