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

impl From<InitSvcId> for u32 {
    fn from(value: InitSvcId) -> Self {
        value as u32
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

impl From<JavaSvcId> for u32 {
    fn from(value: JavaSvcId) -> Self {
        value as u32
    }
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum WIPICKernelMethodId {
    Printk = 0,
    Sprintk = 1,
    GetExecNames = 2,
    Execute = 3,
    Mexecute = 4,
    Load = 5,
    Mload = 6,
    Exit = 7,
    ProgramStop = 8,
    GetCurProgramId = 9,
    GetParentProgramId = 10,
    GetAppManagerId = 11,
    GetProgramInfo = 12,
    GetAccessLevel = 13,
    GetProgramName = 14,
    CreateSharedBuf = 15,
    DestroySharedBuf = 16,
    GetSharedBuf = 17,
    GetSharedBufSize = 18,
    ResizeSharedBuf = 19,
    Alloc = 20,
    Calloc = 21,
    Free = 22,
    GetTotalMemory = 23,
    GetFreeMemory = 24,
    DefTimer = 25,
    SetTimer = 26,
    UnsetTimer = 27,
    CurrentTime = 28,
    GetSystemProperty = 29,
    SetSystemProperty = 30,
    GetResourceId = 31,
    GetResource = 32,
    Reserved1 = 33,
    Reserved2 = 34,
    Reserved3 = 35,
    Reserved4 = 36,
    Reserved5 = 37,
    Reserved6 = 38,
    Reserved7 = 39,
    Reserved8 = 40,
    Reserved9 = 41,
    Reserved10 = 42,
    Reserved11 = 43,
    SendMessage = 44,
    SetTimerEx = 45,
    GetSystemState = 46,
    CreateSystemProgressBar = 47,
    SetSystemProgressBar = 48,
    DestroySystemProgressBar = 49,
    ExecuteEx = 50,
    GetProcAddress = 51,
    Unload = 52,
    CreateSysMessageBox = 53,
    DestroySysMessageBox = 54,
    GetProgramIdList = 55,
    GetProgramInfo2 = 56,
    Reserved12 = 57,
    Reserved13 = 58,
    CreateAppPrivateArea = 59,
    GetAppPrivateArea = 60,
    CreateLibPrivateArea = 61,
    GetLibPrivateArea = 62,
    GetPlatformVersion = 63,
    GetToken = 64,
}

impl From<WIPICKernelMethodId> for u16 {
    fn from(value: WIPICKernelMethodId) -> Self {
        value as u16
    }
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum WIPICGraphicsMethodId {
    GetImageProperty = 0,
    GetImageFramebuffer = 1,
    GetScreenFramebuffer = 2,
    DestroyOffscreenFramebuffer = 3,
    CreateOffscreenFramebuffer = 4,
    InitContext = 5,
    SetContext = 6,
    GetContext = 7,
    PutPixel = 8,
    DrawLine = 9,
    DrawRect = 10,
    FillRect = 11,
    CopyFrameBuffer = 12,
    DrawImage = 13,
    CopyArea = 14,
    DrawArc = 15,
    FillArc = 16,
    DrawString = 17,
    DrawUnicodeString = 18,
    GetRgbPixels = 19,
    SetRgbPixels = 20,
    FlushLcd = 21,
    GetPixelFromRgb = 22,
    GetRgbFromPixel = 23,
    GetDisplayInfo = 24,
    Repaint = 25,
    GetFont = 26,
    GetFontHeight = 27,
    GetFontAscent = 28,
    GetFontDescent = 29,
    GetStringWidth = 30,
    GetUnicodeStringWidth = 31,
    CreateImage = 32,
    DestroyImage = 33,
    DecodeNextImage = 34,
    EncodeImage = 35,
    PostEvent = 36,
    HandleInput = 37,
    SetCurrentMode = 38,
    GetCurrentMode = 39,
    GetSupportModeCount = 40,
    GetSupportedModes = 41,
    FillPolygon = 42,
    DrawPolygon = 43,
    ShowAnnunciator = 44,
    GetAnnunciatorInfo = 45,
    SetAnnunciatorIcon = 46,
    GetIdleHelpLineInfo = 47,
    ShowHelpLine = 48,
    GetCharGlyph = 49,
    CreateImageEx = 50,
    HideHelpLine = 51,
    SetCloneScreenFramebuffer = 52,
    GetFontEx = 53,
    GetFontLists = 54,
    GetFontInfo = 55,
    SetFontHelpLine = 56,
    GetFontHelpLine = 57,
    EncodeImageEx = 58,
    GetImageInfo = 59,
}

impl From<WIPICGraphicsMethodId> for u16 {
    fn from(value: WIPICGraphicsMethodId) -> Self {
        value as u16
    }
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum WIPICDatabaseMethodId {
    OpenDatabase = 0,
    ReadRecordSingle = 1,
    WriteRecordSingle = 2,
    CloseDatabase = 3,
    SelectRecord = 4,
    UpdateRecord = 5,
    DeleteRecord = 6,
    ListRecord = 7,
    SortRecords = 8,
    GetAccessMode = 9,
    GetNumberOfRecords = 10,
    GetRecordSize = 11,
    ListDatabases = 12,
    Unk13 = 13,
    Unk14 = 14,
    Unk15 = 15,
    Unk16 = 16,
}

impl From<WIPICDatabaseMethodId> for u16 {
    fn from(value: WIPICDatabaseMethodId) -> Self {
        value as u16
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
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
    pub fn function_id(self, method_id: impl Into<u16>) -> u32 {
        ((self as u32) << 16) | method_id.into() as u32
    }
}

impl TryFrom<u32> for WIPICTableId {
    type Error = wie_util::WieError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Kernel,
            1 => Self::Util,
            2 => Self::Misc,
            3 => Self::Graphics,
            4 => Self::Interface3,
            5 => Self::Interface4,
            6 => Self::Interface5,
            7 => Self::Database,
            8 => Self::Interface7,
            9 => Self::Uic,
            10 => Self::Media,
            11 => Self::Net,
            12 => Self::Interface11,
            13 => Self::Interface12,
            14 => Self::Interface13,
            15 => Self::Interface14,
            16 => Self::Interface15,
            17 => Self::Interface16,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown KTF WIPIC table id {value}"))),
        })
    }
}
