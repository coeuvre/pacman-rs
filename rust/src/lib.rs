#[macro_use]
extern crate log;
extern crate time;
extern crate gl;

mod logger;

use std::ffi::{CStr, CString};
use std::os::raw::*;

const PLATFORM_EVENT_CLOSE: c_int = 0;

static mut PLATFORM: *mut PlatformApi = 0 as *mut PlatformApi;

struct LibState {
    last_counter: u64,
    count: f32
}

impl LibState {
    pub fn new() -> LibState {
        LibState {
            last_counter: 0,
            count: 0.0,
        }
    }
}

#[repr(C)]
pub struct PlatformApi {
    pub log: unsafe extern "C" fn(message: *const c_char),
    pub get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
    pub swap_gl_buffer: unsafe extern "C" fn(),
    pub get_performance_counter: unsafe extern "C" fn() -> u64,
    pub get_performance_frequency: unsafe extern "C" fn() -> u64
}

unsafe fn get_performance_counter() -> u64 {
    ((*PLATFORM).get_performance_counter)()
}

unsafe fn get_performance_frequency() -> u64 {
    ((*PLATFORM).get_performance_frequency)()
}

unsafe fn swap_gl_buffer() {
    ((*PLATFORM).swap_gl_buffer)()
}

unsafe fn render(state: &mut LibState) {
    let delta;
    let current_counter = get_performance_counter();

    if state.last_counter == 0 {
        // first render
        delta = 0.0;
    } else {
        delta = ((current_counter - state.last_counter) as f64 / get_performance_frequency() as f64) as f32;
    }

    state.count = state.count + delta;

    gl::ClearColor(state.count.min(1.0), 0.0, 0.0, 1.0);
    //gl::ClearColor(255.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

    info!("Delta {}, Count {}", delta, state.count);

    state.last_counter = current_counter;

    swap_gl_buffer();
}

#[no_mangle]
pub unsafe extern "C" fn pacman_start(platform: *mut PlatformApi) {
    PLATFORM = platform;

    logger::init().unwrap();

    info!("init at rust side");

    gl::load_with(|s| {
        let cstring = CString::new(s).unwrap();
        ((*platform).get_gl_proc_address)(cstring.as_ptr())
    });

    let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
    info!("OpenGL Version {}", glversion.to_str().unwrap());

    let mut state = LibState::new();

    'game: loop {
        render(&mut state);
    }
}