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

unsafe fn run_sdl_game_loop(window: *mut SDL_Window) -> Result<(), Error> {
    let mut renderer = Renderer::load(load_gl_fn)?;
    let mut game_state = GameState::load(&mut renderer)?;

    'game: loop {
        let mut event = zeroed::<SDL_Event>();
        while SDL_PollEvent(&mut event) != 0 {
            match transmute::<_, SDL_EventType>(event.type_) {
                SDL_QUIT => {
                    break 'game;
                }
                _ => {}
            }
        }

        let mut window_width = 0;
        let mut window_height = 0;
        SDL_GetWindowSize(window, &mut window_width, &mut window_height);
        renderer.set_viewport_size(Vec2::new(window_width as Scalar, window_height as Scalar));

        game_state.update(&mut renderer);

        SDL_GL_SwapWindow(window);
    }

    Ok(())
}

fn load_gl_fn(symbol: &str) -> *const c_void {
    let symbol = CString::new(symbol).unwrap();
    unsafe { SDL_GL_GetProcAddress(symbol.as_ptr()) }
}
