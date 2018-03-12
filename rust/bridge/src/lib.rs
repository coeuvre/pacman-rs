extern crate log;
extern crate time;

mod logger;

use std::os::raw::*;

pub const PLATFORM_EVENT_CLOSE: c_int = 1;

static mut PLATFORM: *mut Platform = 0 as *mut Platform;

#[derive(Debug)]
#[repr(C)]
pub struct PlatformEvent {
    pub kind: c_int,
}

#[repr(C)]
pub struct Platform {
    poll_event: unsafe extern "C" fn(event: *mut PlatformEvent) -> c_int,

    log: unsafe extern "C" fn(message: *const c_char),

    get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
    swap_gl_buffer: unsafe extern "C" fn(),

    get_performance_counter: unsafe extern "C" fn() -> u64,
    get_performance_frequency: unsafe extern "C" fn() -> u64
}

pub unsafe fn poll_event(event: &mut PlatformEvent) -> bool {
    ((*PLATFORM).poll_event)(event as *mut PlatformEvent) != 0
}

pub unsafe fn log(message: *const c_char) {
    ((*PLATFORM).log)(message)
}

pub unsafe fn get_gl_proc_address(symbol: *const c_char) -> *const c_void {
    ((*PLATFORM).get_gl_proc_address)(symbol)
}

pub unsafe fn swap_gl_buffer() {
    ((*PLATFORM).swap_gl_buffer)()
}

pub unsafe fn get_performance_counter() -> u64 {
    ((*PLATFORM).get_performance_counter)()
}

pub unsafe fn get_performance_frequency() -> u64 {
    ((*PLATFORM).get_performance_frequency)()
}

pub unsafe fn init(platform: *mut Platform) {
    PLATFORM = platform;
    logger::init().unwrap();
}