//! Fixed core/ROM boot register addresses (STATIC: reference amp_proc1.rs).
pub const CORE_ID: *const u32 = 0xe00f_f01c as *const u32;
pub const RESET_CTRL0: *const u32 = 0x4001_4000 as *const u32;
pub const RESET_DONE0: *const u32 = 0x4001_4018 as *const u32;
pub const RESET_CLEAR0: *mut u32 = 0x4001_7000 as *mut u32;
pub const BOOT_MAGIC: *mut u32 = 0x4015_400c as *mut u32;
pub const PROC1_ENTRY: *mut u32 = 0x4015_4014 as *mut u32;
pub const PROC1_STACK: *mut u32 = 0x4015_401c as *mut u32;
pub const PROC1_RESET_MASK: u32 = 1 << 31;
pub const HOST_EVENTS_SET: *mut u32 = 0x4000_a00c as *mut u32;
pub const HOST_EVENTS_CLEAR: *mut u32 = 0x4000_b00c as *mut u32;
pub const PROC_EVENTS: *const u32 = 0x4000_8008 as *const u32;
pub const PROC_EVENTS_CLEAR: *mut u32 = 0x4000_b008 as *mut u32;
