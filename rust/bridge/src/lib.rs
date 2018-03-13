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
    quit: unsafe extern "C" fn(),
    log: unsafe extern "C" fn(message: *const c_char),

    get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,

    get_performance_counter: unsafe extern "C" fn() -> u64,
    get_performance_frequency: unsafe extern "C" fn() -> u64
}

pub unsafe fn quit() {
    ((*PLATFORM).quit)()
}

pub unsafe fn log(message: *const c_char) {
    ((*PLATFORM).log)(message)
}

pub unsafe fn get_gl_proc_address(symbol: *const c_char) -> *const c_void {
    ((*PLATFORM).get_gl_proc_address)(symbol)
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