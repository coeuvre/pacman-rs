extern crate gl;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::ffi::{CStr, CString};
use std::os::raw::*;

static mut PLATFORM: *mut Platform = 0 as *mut Platform;
static mut LIB: *mut PacManLib = 0 as *mut PacManLib;
static mut STATE: *mut State = 0 as *mut State;

enum PlatformEvent {
    Count
}

struct State {
    platform_event_sender: Sender<PlatformEvent>,
    platform_event_receiver: Receiver<PlatformEvent>,
    count: i32
}

#[repr(C)]
pub struct Platform {
    pub get_gl_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
}

#[repr(C)]
pub struct PacManLib {
    pub on_platform_event: unsafe extern "C" fn(c_int, *const c_void),
    pub update: unsafe extern "C" fn(),
    pub render: unsafe extern "C" fn(),
}

#[no_mangle]
pub unsafe extern "C" fn pacman_init(platform: *mut Platform) -> *mut PacManLib {
    PLATFORM = platform;

    println!("init at rust side");
    gl::load_with(|s| {
        let cstring = CString::new(s).unwrap();
        ((*PLATFORM).get_gl_proc_address)(cstring.as_ptr())
    });

    // println!("{}", add(1, 2));
    // // ::std::thread::sleep_ms(1000);
    let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
    println!("OpenGL Version {}", glversion.to_str().unwrap());

    let (sender, receiver) = channel();
    let state = Box::new(State {
        count: 0,
        platform_event_sender: sender,
        platform_event_receiver: receiver,
    });
    STATE = Box::into_raw(state);

    LIB = Box::into_raw(Box::new(PacManLib  {
        on_platform_event,
        update,
        render,
    }));

    LIB
}

unsafe extern "C" fn update() {
    while let Ok(platform_event) = (*STATE).platform_event_receiver.try_recv() {
        match platform_event {
            PlatformEvent::Count => (*STATE).count += 1,
        }
    }
}

unsafe extern "C" fn render() {
    let state = &*STATE;

    gl::ClearColor((state.count as f32 / 255.0).min(1.0), 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

    println!("{}", state.count);
}

unsafe extern "C" fn on_platform_event(_event_id: c_int, _data: *const c_void) {
    (*STATE).platform_event_sender.send(PlatformEvent::Count).unwrap();
}