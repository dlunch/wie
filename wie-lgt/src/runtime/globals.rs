// Keep these slots disjoint from those in wie-wipi-c/src/globals.rs.
pub const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000; // 8 bytes: JavaSupportContext.
pub const GRAPHICS_STATE_ROOT: u32 = 0x7fff1008; // 4 bytes: LgtGraphicsState pointer.
pub const DISPLAY_PROPERTIES_ROOT: u32 = 0x7fff1010; // 12 bytes: LgtDisplayProperties.
pub const STRTOK_STATE: u32 = 0x7fff101c; // 4 bytes: strtok continuation pointer.
