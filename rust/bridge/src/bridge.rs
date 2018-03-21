use std::os::raw::*;

pub const PLATFORM_EVENT_RENDER: c_int = 1;
pub const PLATFORM_EVENT_CLOSE: c_int = 2;
pub const PLATFORM_EVENT_RESIZE: c_int = 3;

static mut PLATFORM: *mut Platform = 0 as *mut Platform;

#[repr(C)]
pub struct PlatformEvent {
    pub kind: c_int,
    pub data: PlatformEventData,
}

#[repr(C)]
pub union PlatformEventData {
    pub resize: PlatformEventDataResize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PlatformEventDataResize {
    pub width: c_int,
    pub height: c_int,
}

#[repr(C)]
pub struct Platform {
    quit: unsafe extern "C" fn(),

    get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
    swap_gl_buffers: unsafe extern "C" fn(),

    get_performance_counter: unsafe extern "C" fn() -> u64,
    get_performance_frequency: unsafe extern "C" fn() -> u64,
}

pub unsafe fn quit() {
    ((*PLATFORM).quit)()
}

pub unsafe fn get_gl_proc_address(symbol: *const c_char) -> *const c_void {
    ((*PLATFORM).get_gl_proc_address)(symbol)
}

pub unsafe fn swap_gl_buffers() {
    ((*PLATFORM).swap_gl_buffers)()
}

pub unsafe fn get_performance_counter() -> u64 {
    ((*PLATFORM).get_performance_counter)()
}

pub unsafe fn get_performance_frequency() -> u64 {
    ((*PLATFORM).get_performance_frequency)()
}

pub unsafe fn init(platform: *mut Platform) {
    PLATFORM = platform;
}
