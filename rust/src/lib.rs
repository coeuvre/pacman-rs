#[macro_use] extern crate log;
extern crate failure;

extern crate gl;
extern crate bridge;
extern crate stb;

pub mod game;
pub mod image;

use std::ffi::{CStr, CString};
use game::*;

#[no_mangle]
pub unsafe extern "C" fn game_main(platform: *mut bridge::Platform) {
    bridge::init(platform);

    gl::load_with(|s| {
        let cstring = CString::new(s).unwrap();
        bridge::get_gl_proc_address(cstring.as_ptr())
    });

    let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
    info!("OpenGL Version {}", glversion.to_str().unwrap());

    if let Err(e) = image::Image::load("example.png") {
        error!("An error occurs: {}", e);
        return;
    }

    let mut game = Game::new();

    static FRAMETIME: f32 = 0.016;
    let frequency = bridge::get_performance_frequency() as f64;
    let mut last_counter = bridge::get_performance_counter();
    let mut frame = 0;

    game.render();
    bridge::swap_gl_buffer();

    let mut event = std::mem::uninitialized();
    'game: loop {
        while bridge::poll_event(&mut event) {
            info!("{:?}", event);
            match event.kind {
                bridge::PLATFORM_EVENT_CLOSE => break 'game,
                _ => {}
            }
        }

        let current_counter = bridge::get_performance_counter();
        let delta = ((current_counter - last_counter) as f64 / frequency) as f32;
        if delta >= FRAMETIME {
            last_counter = current_counter;
            game.update(FRAMETIME);
            game.render();
            bridge::swap_gl_buffer();

            info!("Render frame {}, delta {}", frame, delta);
            frame = frame + 1;
        }
    }
}