#[macro_use] extern crate log;
extern crate failure;

extern crate gl;
extern crate bridge;
extern crate stb;

pub mod game;
pub mod image;

use std::ffi::{CStr, CString};
use game::*;

struct State {
    game: Game,
    frame: u64,
    frequency: u64,
    last_counter: u64
}

impl State {
    pub fn new() -> State {
        State {
            game: Game::new(),
            frame: 0,
            frequency: unsafe { bridge::get_performance_frequency() },
            last_counter: unsafe { bridge::get_performance_counter() },
        }
    }
}

static mut STATE: *mut State = 0 as *mut State;

#[no_mangle]
pub unsafe extern "C" fn game_load(platform: *mut bridge::Platform) {
    bridge::init(platform);

    gl::load_with(|s| {
        let cstring = CString::new(s).unwrap();
        bridge::get_gl_proc_address(cstring.as_ptr())
    });

    let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
    info!("OpenGL Version {}", glversion.to_str().unwrap());

    if let Err(e) = image::Image::load("example.png") {
        error!("An error occurs: {}", e);
    }

    STATE = Box::into_raw(Box::new(State::new()));
}

#[no_mangle]
pub unsafe extern "C" fn game_render() {
    let state = &mut *STATE;

    static FRAMETIME: f32 = 0.016;
    let current_counter = bridge::get_performance_counter();
    let delta = ((current_counter - state.last_counter) as f64 / state.frequency as f64) as f32;
    if delta >= FRAMETIME {
        state.last_counter = current_counter;
        state.game.update(FRAMETIME);
        info!("Render frame {}, delta {}", state.frame, delta);
        state.frame = state.frame + 1;
    }
    state.game.render();
}

pub unsafe extern "C" fn game_on_platform_event(event: bridge::PlatformEvent) {
    match event.kind {
        // bridge::PLATFORM_EVENT_CLOSE => break 'game,
        _ => {}
    }
}

// #[no_mangle]
// pub unsafe extern "C" fn game_main(platform: *mut bridge::Platform) {
//     static FRAMETIME: f32 = 0.016;
//     let frequency = bridge::get_performance_frequency() as f64;
//     let mut last_counter = bridge::get_performance_counter();
//     let mut frame = 0;

//     game.render();
//     bridge::swap_gl_buffer();

//     let mut event = std::mem::uninitialized();
//     'game: loop {
//         while bridge::poll_event(&mut event) {
//             info!("{:?}", event);
//             match event.kind {
//                 bridge::PLATFORM_EVENT_CLOSE => break 'game,
//                 _ => {}
//             }
//         }

//         let current_counter = bridge::get_performance_counter();
//         let delta = ((current_counter - last_counter) as f64 / frequency) as f32;
//         if delta >= FRAMETIME {
//             last_counter = current_counter;
//             game.update(FRAMETIME);
//             game.render();
//             bridge::swap_gl_buffer();

//             info!("Render frame {}, delta {}", frame, delta);
//             frame = frame + 1;
//         }
//     }
// }