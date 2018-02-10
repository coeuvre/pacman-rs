extern crate gl;
#[macro_use]
extern crate lazy_static;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::ffi::{CStr, CString};
use std::os::raw::*;
use std::sync::{Once, ONCE_INIT, RwLock};

static mut LIB: *mut PacManLib = 0 as *mut PacManLib;

lazy_static! {
    static ref STATE_LOCK: RwLock<State> = RwLock::new(State::new());
}

const PLATFORM_EVENT_CLOSE: c_int = 0;

struct State {
    count: i32
}

impl State {
    pub fn new() -> State {
        State {
            count: 0,
        }
    }
}

#[repr(C)]
pub struct Platform {
    pub quit: unsafe extern "C" fn(),
    pub get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
}

#[repr(C)]
pub struct PacManLib {
    pub on_platform_event: unsafe extern "C" fn(*mut Platform, c_int, *const c_void),
    pub update: unsafe extern "C" fn(*mut Platform),
    pub render: unsafe extern "C" fn(*mut Platform),
}

#[no_mangle]
pub unsafe extern "C" fn pacman_load(platform: *mut Platform) -> *mut PacManLib {
    println!("init at rust side");

    lazy_static::initialize(&STATE_LOCK);

    LIB = Box::into_raw(Box::new(PacManLib  {
        on_platform_event,
        update,
        render,
    }));

    LIB
}

unsafe extern "C" fn update(platform: *mut Platform) {
    let mut state = STATE_LOCK.write().unwrap();

    state.count = state.count + 1;
}

static INIT_RENDER: Once = ONCE_INIT;

unsafe extern "C" fn render(platform: *mut Platform) {
    INIT_RENDER.call_once(|| {
        gl::load_with(|s| {
            let cstring = CString::new(s).unwrap();
            ((*platform).get_gl_proc_address)(cstring.as_ptr())
        });

        let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
        println!("OpenGL Version {}", glversion.to_str().unwrap());

    });

    let state = STATE_LOCK.read().unwrap();

    gl::ClearColor((state.count as f32 / 255.0).min(1.0), 0.0, 0.0, 1.0);
    // gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

    println!("{}", state.count);
}

unsafe extern "C" fn on_platform_event(platform: *mut Platform, event_id: c_int, _data: *const c_void) {
    match event_id {
        PLATFORM_EVENT_CLOSE => ((*platform).quit)(),
        _ => {}
    }
}