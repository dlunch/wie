pub(crate) mod init {
    pub const IMMEDIATE: u32 = 1;
    pub const GET_INTERFACE: u32 = 1;
    pub const JAVA_THROW: u32 = 2;
    pub const JAVA_CHECK_TYPE: u32 = 3;
    pub const JAVA_NEW: u32 = 4;
    pub const JAVA_ARRAY_NEW: u32 = 5;
    pub const JAVA_CLASS_LOAD: u32 = 6;
    pub const ALLOC: u32 = 7;
    pub const JAVA_JUMP_1: u32 = 8;
    pub const JAVA_JUMP_2: u32 = 9;
    pub const JAVA_JUMP_3: u32 = 10;
    pub const GET_JAVA_METHOD: u32 = 11;
    pub const GET_FIELD: u32 = 12;
    pub const JB_UNK4: u32 = 13;
    pub const JB_UNK5: u32 = 14;
    pub const JB_UNK7: u32 = 15;
    pub const JB_UNK8: u32 = 16;
    pub const REGISTER_CLASS: u32 = 17;
    pub const REGISTER_JAVA_STRING: u32 = 18;
    pub const CALL_NATIVE: u32 = 19;
}
