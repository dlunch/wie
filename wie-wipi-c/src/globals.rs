// Keep these slots disjoint from those in wie-lgt/src/runtime/globals.rs.
pub const SCREEN_FRAMEBUFFER_PTR: u32 = 0x7fff1000; // 4 bytes: framebuffer pointer.
pub const INPUT_MODES_PTR: u32 = 0x7fff1020; // 4 bytes: input-mode table pointer.
pub const CURRENT_INPUT_MODE: u32 = 0x7fff1024; // 4 bytes: input-mode index.
pub const UIC_TEXT_CLASS: u32 = 0x7fff1028; // 4 bytes: text-component class descriptor.
