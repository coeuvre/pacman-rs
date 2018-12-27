use std::{
    ffi::CString,
    ptr::null_mut,
    mem::{
        uninitialized,
        transmute
    }
};

use failure::{Error, format_err};
use sdl2_sys::{
    *,
    SDL_GLattr::*,
    SDL_GLprofile::*,
    SDL_WindowFlags::*,
};

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
        (SDL_WINDOWPOS_CENTERED_MASK | 0) as i32,
        (SDL_WINDOWPOS_CENTERED_MASK | 0) as i32,
        WINDOW_WIDTH, WINDOW_HEIGHT,
        SDL_WINDOW_SHOWN as u32 | SDL_WINDOW_RESIZABLE as u32 | SDL_WINDOW_OPENGL as u32
    );
    if window == null_mut() {
        return sdl_error!();
    }

    let gl_context = SDL_GL_CreateContext(window);
    if gl_context == null_mut() {
        return sdl_error!();
    }

    if SDL_GL_SetSwapInterval(1) != 0 {
        return sdl_error!();
    }

    sdl_game_loop(window)
}

unsafe fn sdl_game_loop(window: *mut SDL_Window) -> Result<(), Error> {
    'game: loop {
        let mut event = uninitialized::<SDL_Event>();
        while SDL_PollEvent(&mut event) != 0 {
            let event_type: SDL_EventType = transmute(event.type_);
            match event_type {
                SDL_EventType::SDL_QUIT => {
                    break 'game;
                }
                _ => {}
            }
        }

        SDL_GL_SwapWindow(window);
    }

    Ok(())
}
