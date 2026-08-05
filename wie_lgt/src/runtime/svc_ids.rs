use wie_core_arm::SvcId;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum InitSvcId {
    GetImportTable = 0,
    GetImportFunction = 1,
    Unk0 = 2,
    GetApplicationJarPath = 3,
}

impl TryFrom<SvcId> for InitSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0 => Self::GetImportTable,
            1 => Self::GetImportFunction,
            2 => Self::Unk0,
            3 => Self::GetApplicationJarPath,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown LGT init SVC id {}", value.0))),
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum JavaSystemSvcId {
    InterfaceUnk0 = 0,
    DestroyRuntimeContext = 1,
    CreateRuntimeContext = 2,
    LinkImportedClasses = 3,
    SetJarPath = 4,
    StartApplication = 5,
    RegisterClass = 6,
    ResolveClass = 7,
    InitializeClass = 8,
    GetArrayType = 9,
    Instantiate = 10,
    InstantiateArray = 11,
    Unk54 = 12,
    Unk55 = 13,
    StringLiteral = 14,
    PushExceptionFrame = 15,
    PopExceptionFrame = 16,
    StoreReferenceArray = 17,
    GetStringClass = 18,
    GetStringArrayClass = 19,
    PendingException = 20,
    StoreReferenceArrayUnchecked = 21,
    InstantiateMultiArray = 22,
    LinkPublicClass = 23,
    ExceptionMatchesClass = 24,
    RethrowException = 25,
    RaiseNullPointerException = 26,
    RaiseArrayIndexException = 27,
    RaiseArithmeticException = 28,
    Unk1 = 29,
    Unk2 = 30,
    Unk3 = 31,
    GetInterfaceDispatchTable = 32,
    MonitorEnter = 33,
    MonitorExit = 34,
}

impl TryFrom<SvcId> for JavaSystemSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0 => Self::InterfaceUnk0,
            1 => Self::DestroyRuntimeContext,
            2 => Self::CreateRuntimeContext,
            3 => Self::LinkImportedClasses,
            4 => Self::SetJarPath,
            5 => Self::StartApplication,
            6 => Self::RegisterClass,
            7 => Self::ResolveClass,
            8 => Self::InitializeClass,
            9 => Self::GetArrayType,
            10 => Self::Instantiate,
            11 => Self::InstantiateArray,
            12 => Self::Unk54,
            13 => Self::Unk55,
            14 => Self::StringLiteral,
            15 => Self::PushExceptionFrame,
            16 => Self::PopExceptionFrame,
            17 => Self::StoreReferenceArray,
            18 => Self::GetStringClass,
            19 => Self::GetStringArrayClass,
            20 => Self::PendingException,
            21 => Self::StoreReferenceArrayUnchecked,
            22 => Self::InstantiateMultiArray,
            23 => Self::LinkPublicClass,
            24 => Self::ExceptionMatchesClass,
            25 => Self::RethrowException,
            26 => Self::RaiseNullPointerException,
            27 => Self::RaiseArrayIndexException,
            28 => Self::RaiseArithmeticException,
            29 => Self::Unk1,
            30 => Self::Unk2,
            31 => Self::Unk3,
            32 => Self::GetInterfaceDispatchTable,
            33 => Self::MonitorEnter,
            34 => Self::MonitorExit,
            _ => {
                return Err(wie_util::WieError::FatalError(alloc::format!(
                    "Unknown LGT Java system SVC id {}",
                    value.0
                )));
            }
        })
    }
}

impl From<JavaSystemSvcId> for u32 {
    fn from(value: JavaSystemSvcId) -> Self {
        value as u32
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
    GetImageFramebuffer = 0xc9,
    GetScreenFramebuffer = 0xca,
    DestroyOffscreenFramebuffer = 0xcb,
    CreateOffscreenFramebuffer = 0xcc,
    InitContext = 0xcd,
    SetContext = 0xce,
    PutPixel = 0xd0,
    DrawLine = 0xd1,
    DrawRect = 0xd2,
    FillRect = 0xd3,
    CopyFrameBuffer = 0xd4,
    DrawImage = 0xd5,
    CopyArea = 0xd7,
    DrawString = 0xda,
    GetRgbPixels = 0xdc,
    SetRgbPixels = 0xdd,
    FlushLcd = 0xde,
    GetPixelFromRgb = 0xdf,
    GetRgbFromPixel = 0xe0,
    GetDisplayInfo = 0xe1,
    Repaint = 0xe2,
    GetFont = 0xe3,
    GetFontHeight = 0xe4,
    GetFontAscent = 0xe5,
    GetFontDescent = 0xe6,
    GetStringWidth = 0xe7,
    CreateImage = 0xe9,
    Unk0 = 0xeb,
    Unk11 = 0xee,
    Unk3 = 0x12c,
    Unk4 = 0x12d,
    Unk7 = 0x12e,
    Unk6 = 0x12f,
    TimeNow = 0x320,
    TimeComponent = 0x321,
    TimeConvert = 0x322,
    TimeToTm = 0x323,
    DateTimeToTm = 0x338,
    OpenDatabase = 0x190,
    ReadRecordSingle = 0x191,
    WriteRecordSingle = 0x192,
    CloseDatabase = 0x193,
    Unk12 = 0x194,
    Unk9 = 0x195,
    DeleteRecord = 0x196,
    ListRecord = 0x197,
    UpdateRecord = 0x198,
    SelectRecord = 0x199,
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
            0xc9 => Self::GetImageFramebuffer,
            0xca => Self::GetScreenFramebuffer,
            0xcb => Self::DestroyOffscreenFramebuffer,
            0xcc => Self::CreateOffscreenFramebuffer,
            0xcd => Self::InitContext,
            0xce => Self::SetContext,
            0xd0 => Self::PutPixel,
            0xd1 => Self::DrawLine,
            0xd2 => Self::DrawRect,
            0xd3 => Self::FillRect,
            0xd4 => Self::CopyFrameBuffer,
            0xd5 => Self::DrawImage,
            0xd7 => Self::CopyArea,
            0xda => Self::DrawString,
            0xdc => Self::GetRgbPixels,
            0xdd => Self::SetRgbPixels,
            0xde => Self::FlushLcd,
            0xdf => Self::GetPixelFromRgb,
            0xe0 => Self::GetRgbFromPixel,
            0xe1 => Self::GetDisplayInfo,
            0xe2 => Self::Repaint,
            0xe3 => Self::GetFont,
            0xe4 => Self::GetFontHeight,
            0xe5 => Self::GetFontAscent,
            0xe6 => Self::GetFontDescent,
            0xe7 => Self::GetStringWidth,
            0xe9 => Self::CreateImage,
            0xeb => Self::Unk0,
            0xee => Self::Unk11,
            0x12c => Self::Unk3,
            0x12d => Self::Unk4,
            0x12e => Self::Unk7,
            0x12f => Self::Unk6,
            0x320 => Self::TimeNow,
            0x321 => Self::TimeComponent,
            0x322 => Self::TimeConvert,
            0x323 => Self::TimeToTm,
            0x338 => Self::DateTimeToTm,
            0x190 => Self::OpenDatabase,
            0x191 => Self::ReadRecordSingle,
            0x192 => Self::WriteRecordSingle,
            0x193 => Self::CloseDatabase,
            0x194 => Self::Unk12,
            0x195 => Self::Unk9,
            0x196 => Self::DeleteRecord,
            0x197 => Self::ListRecord,
            0x198 => Self::UpdateRecord,
            0x199 => Self::SelectRecord,
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
    Time = 0x41a,
    Localtime = 0x420,
    Unk3 = 0x424,
}

impl From<StdlibSvcId> for u32 {
    fn from(value: StdlibSvcId) -> Self {
        value as u32
    }
}
