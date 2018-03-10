#[macro_use]
extern crate log;
extern crate time;
extern crate gl;

mod logger;

use std::ffi::{CStr, CString};
use std::os::raw::*;

const PLATFORM_EVENT_CLOSE: c_int = 1;

static mut PLATFORM: *mut PlatformApi = 0 as *mut PlatformApi;

struct LibState {
    count: f32
}

impl LibState {
    pub fn new() -> LibState {
        LibState {
            count: 0.0,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PlatformEvent {
    kind: c_int,
}

#[repr(C)]
pub struct PlatformApi {
    pub poll_event: unsafe extern "C" fn(event: *mut PlatformEvent) -> c_int,

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

unsafe fn poll_event(event: &mut PlatformEvent) -> bool {
    ((*PLATFORM).poll_event)(event as *mut PlatformEvent) != 0
}


unsafe fn update(state: &mut LibState, dt: f32) {
    state.count = state.count + dt;
}

unsafe fn render(state: &LibState) {
    gl::ClearColor(state.count.min(1.0), 0.0, 0.0, 1.0);
    //gl::ClearColor(255.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

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

    static FRAMETIME: f32 = 0.016;
    let frequency = get_performance_frequency() as f64;
    let mut last_counter = get_performance_counter();
    let mut frame = 0;

    render(&state);

    let mut event = std::mem::uninitialized();
    'game: loop {
        while poll_event(&mut event) {
            info!("{:?}", event);
            match event.kind {
                PLATFORM_EVENT_CLOSE => break 'game,
                _ => {}
            }
        }

        let current_counter = get_performance_counter();
        let delta = ((current_counter - last_counter) as f64 / frequency) as f32;
        if delta >= FRAMETIME {
            last_counter = current_counter;
            update(&mut state, FRAMETIME);
            render(&state);
            info!("Render frame {}, delta {}", frame, delta);
            frame = frame + 1;
        }
    }
}