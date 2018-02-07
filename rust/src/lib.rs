extern crate gl;

use std::ffi::{CStr, CString};
use std::os::raw::*;

struct GameState {
    count: i32
}

static mut GAME_STATE: GameState = GameState {
    count: 0
};

#[no_mangle]
pub unsafe fn pacman_init(get_gl_proc_address: *const c_void) {
    println!("init at rust side");
    let get_gl_proc_address: extern "C" fn(*const c_char) -> *const c_void = ::std::mem::transmute(get_gl_proc_address);

    gl::load_with(|s| {
        let cstring = CString::new(s).unwrap();
        get_gl_proc_address(cstring.as_ptr())
    });

    // println!("{}", add(1, 2));
    // // ::std::thread::sleep_ms(1000);
    let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
    println!("OpenGL Version {}", glversion.to_str().unwrap());
}

#[no_mangle]
pub unsafe fn pacman_update() {
    GAME_STATE.count += 1;
}

#[no_mangle]
pub unsafe fn pacman_render() {
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

    println!("{}", GAME_STATE.count);
}

#[no_mangle]
pub unsafe fn pacman_start() {

}
