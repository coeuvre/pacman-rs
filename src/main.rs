use std::{
    ffi::*,
    mem::*,
};
use failure::{Error, format_err};
use sdl2_sys::{
    *,
    SDL_GLattr::*,
    SDL_GLprofile::*,
    SDL_WindowFlags::*,
    SDL_EventType::*,
};
use profiler::*;

macro_rules! offset_of {
    ($ty:ty, $field:tt) => ({
        let base = std::ptr::null::<$ty>();
        let field = &(*base).$field as *const _;
        field as usize - base as usize
    });
}

pub mod asset;
pub mod renderer;
pub mod math;
pub mod bitmap;
pub mod game;

use crate::renderer::*;
use crate::math::*;
use crate::game::*;

fn main() -> Result<(), Error> {
    unsafe { sdl_main() }
}

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

macro_rules! sdl_error {
    () => {{
        use std::ffi::CStr;
        Err(format_err!("{}", CStr::from_ptr(SDL_GetError()).to_string_lossy()))
    }};
}

unsafe fn sdl_main() -> Result<(), Error> {
    if SDL_Init(SDL_INIT_EVERYTHING) != 0 {
        return sdl_error!();
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE as i32);
    SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_GL_SetAttribute(SDL_GL_FRAMEBUFFER_SRGB_CAPABLE, 1);
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLEBUFFERS, 1);
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLESAMPLES, 4);

    let title = CString::new("PacMan").unwrap();
    let window = SDL_CreateWindow(
        title.as_ptr(),
        SDL_WINDOWPOS_CENTERED_MASK as i32,
        SDL_WINDOWPOS_CENTERED_MASK as i32,
        WINDOW_WIDTH, WINDOW_HEIGHT,
        SDL_WINDOW_SHOWN as u32 | SDL_WINDOW_RESIZABLE as u32 | SDL_WINDOW_OPENGL as u32
    );
    if window.is_null() {
        return sdl_error!();
    }

    let gl_context = SDL_GL_CreateContext(window);
    if gl_context.is_null() {
        return sdl_error!();
    }

    if SDL_GL_SetSwapInterval(1) != 0 {
        return sdl_error!();
    }

    run_sdl_game_loop(window)
}

#[profile]
fn poll_event() -> bool {
    let mut event = unsafe { zeroed::<SDL_Event>() };
    while unsafe { SDL_PollEvent(&mut event) } != 0 {
        match unsafe { transmute::<_, SDL_EventType>(event.type_) } {
            SDL_QUIT => {
                return true
            }
            _ => {}
        }
    }

    false
}

#[profile]
fn swap_buffer(window: *mut SDL_Window) {
    unsafe { SDL_GL_SwapWindow(window); }
}

#[profile]
fn sleep(counter_per_frame: u64, frequency: u64) {
    reduce_profile_events();
    let current_counter = get_performance_counter();
    let last_counter = last_counter().unwrap();
    let frame_delta_counter = current_counter - last_counter;
    if frame_delta_counter < counter_per_frame {
        let sleep_ms = ((counter_per_frame - frame_delta_counter) * 1000 / frequency) as u32;
        if sleep_ms > 0 {
            unsafe { SDL_Delay(sleep_ms); }
        }
    }
}

fn prepare_frame(window: *mut SDL_Window, dl: &mut DisplayList) {
    let mut window_width = 0;
    let mut window_height = 0;
    unsafe { SDL_GetWindowSize(window, &mut window_width, &mut window_height); }
    let viewport_size = Vec2::new(window_width as Scalar, window_height as Scalar);
    dl.set_viewport(Rect2::with_min_size(Vec2::zero(), viewport_size));
    dl.clear();
}

fn run_sdl_game_loop(window: *mut SDL_Window) -> Result<(), Error> {
    begin_frame();
    let mut renderer = Renderer::load(load_gl_fn)?;
    let mut game_state = GameState::load(&mut renderer)?;

    let input = Input {
        dt: 0.016,
    };
    let mut dl = DisplayList::new();

    let frequency = get_performance_frequency();
    let counter_per_frame = (input.dt as f64 * frequency as f64) as u64;
    end_frame();

    'game: loop {
        begin_frame();

        if poll_event() {
            break 'game;
        }

        prepare_frame(window, &mut dl);
        game_state.update(&input, &mut renderer, &mut dl);
        renderer.render(&dl);
        swap_buffer(window);
        sleep(counter_per_frame, frequency);

        end_frame();
        reduce_profile_events();
    }

    Ok(())
}

fn load_gl_fn(symbol: &str) -> *const c_void {
    let symbol = CString::new(symbol).unwrap();
    unsafe { SDL_GL_GetProcAddress(symbol.as_ptr()) }
}
