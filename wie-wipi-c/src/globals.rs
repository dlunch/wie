// Keep these slots disjoint from those in wie-lgt/src/runtime/globals.rs.
pub const SCREEN_FRAMEBUFFER_PTR: u32 = 0x7fff1000; // 4 bytes: framebuffer pointer.
pub const INPUT_MODES_PTR: u32 = 0x7fff1020; // 4 bytes: input-mode table pointer.
pub const PENDING_TIMERS_HEAD: u32 = 0x7fff1024; // 4 bytes: pending timer list head.
pub const UIC_TEXT_CLASS: u32 = 0x7fff1028; // 28-byte class record followed by the NUL-terminated class name.
pub const UIC_INPUT_COMPONENT: u32 = 0x7fff1054; // 4 bytes: custom handler input target (0 outside a handler, u32::MAX for a non-typing event).
pub const UIC_LAST_DESTROYED: u32 = 0x7fff105c; // Most recently destroyed component handle.
