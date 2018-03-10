#[macro_use]
extern crate log;
extern crate time;
extern crate gl;

mod logger;

use std::ffi::{CStr, CString};
use std::os::raw::*;

const PLATFORM_EVENT_CLOSE: c_int = 0;

static mut PLATFORM: *mut PlatformApi = 0 as *mut PlatformApi;
static mut LIB: *mut LibState = 0 as *mut LibState;

struct LibState {
    api: LibApi,
    last_counter: u64,
    count: f32
}

impl LibState {
    pub fn new(api: LibApi) -> LibState {
        LibState {
            api,
            last_counter: 0,
            count: 0.0,
        }
    }
}

#[repr(C)]
pub struct PlatformApi {
    pub quit: unsafe extern "C" fn(),
    pub log: unsafe extern "C" fn(message: *const c_char),
    pub get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
    pub get_performance_counter: unsafe extern "C" fn() -> u64,
    pub get_performance_frequency: unsafe extern "C" fn() -> u64
}

#[repr(C)]
pub struct LibApi {
    pub on_platform_event: unsafe extern "C" fn(c_int, *const c_void),
    pub render: unsafe extern "C" fn(),
}

#[no_mangle]
pub unsafe extern "C" fn pacman_load(platform: *mut PlatformApi) -> *mut LibApi {
    PLATFORM = platform;

    logger::init().unwrap();

    info!("init at rust side");

    gl::load_with(|s| {
        let cstring = CString::new(s).unwrap();
        ((*platform).get_gl_proc_address)(cstring.as_ptr())
    });

    let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
    info!("OpenGL Version {}", glversion.to_str().unwrap());

    LIB = Box::into_raw(Box::new(LibState::new(LibApi {
        on_platform_event,
        render,
    })));

    &mut (*LIB).api
}

unsafe extern "C" fn render() {
    let state = &mut *LIB;

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
}

unsafe extern "C" fn on_platform_event(event_id: c_int, _data: *const c_void) {
    match event_id {
        PLATFORM_EVENT_CLOSE => ((*PLATFORM).quit)(),
        _ => {}
    }
}

unsafe fn get_performance_counter() -> u64 {
    ((*PLATFORM).get_performance_counter)()
}

unsafe fn get_performance_frequency() -> u64 {
    ((*PLATFORM).get_performance_frequency)()
}