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
    count: f32
}

impl LibState {
    pub fn new(api: LibApi) -> LibState {
        LibState {
            api,
            count: 0.0,
        }
    }
}

#[repr(C)]
pub struct PlatformApi {
    pub quit: unsafe extern "C" fn(),
    pub log: unsafe extern "C" fn(message: *const c_char),
    pub get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
    pub get_delta_time: unsafe extern "C" fn() -> f32,
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

    state.count = state.count + ((*PLATFORM).get_delta_time)();

    gl::ClearColor(state.count.min(1.0), 0.0, 0.0, 1.0);
    // gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

    info!("{}", state.count);
}

unsafe extern "C" fn on_platform_event(event_id: c_int, _data: *const c_void) {
    match event_id {
        PLATFORM_EVENT_CLOSE => ((*PLATFORM).quit)(),
        _ => {}
    }
}