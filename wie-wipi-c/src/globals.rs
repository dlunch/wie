// Keep these slots disjoint from those in wie-lgt/src/runtime/globals.rs.
pub const SCREEN_FRAMEBUFFER_PTR: u32 = 0x7fff1000; // 4 bytes: framebuffer pointer.
pub const INPUT_MODES_PTR: u32 = 0x7fff1020; // 4 bytes: input-mode table pointer.
pub const PENDING_TIMERS_HEAD: u32 = 0x7fff1024; // 4 bytes: pending timer list head.
