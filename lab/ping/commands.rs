pub const DELAY: u32 = 1 << 31;
pub const COUNT: u32 = u32::MAX - 1;
pub const PANIC: u32 = u32::MAX;
#[allow(dead_code)]
pub const STOP: u32 = u32::MAX - 2;

#[allow(dead_code)]
pub const FAULT: u32 = u32::MAX - 3;
#[allow(dead_code)]
pub const PROC0_PANIC: u32 = u32::MAX - 4;
#[allow(dead_code)]
pub const IDLE_BEGIN: u32 = u32::MAX - 5;
#[allow(dead_code)]
pub const IDLE_END: u32 = u32::MAX - 6;

// Channel probes use a separate pair so a service fence cannot free a reply slot.
#[allow(dead_code)]
pub const SERVICE_FENCE: u32 = u32::MAX - 7;
#[allow(dead_code)]
pub const CREDIT_RESUMED: u32 = u32::MAX - 8;
#[allow(dead_code)]
pub const CONTROL_NO_IPC: u32 = u32::MAX - 9;
#[allow(dead_code)]
pub const CONTROL_IPC: u32 = u32::MAX - 10;
