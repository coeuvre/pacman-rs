#[macro_use] extern crate log;
extern crate failure;
extern crate env_logger;

extern crate gl;
extern crate bridge;
extern crate stb;

pub mod game;
pub mod image;

use game::*;
use bridge::PlatformEvent;

pub trait Game {
    fn load() -> Self;
    fn render(&mut self);
    fn on_platform_event(&mut self, event: &PlatformEvent);
}

static mut GAME: *mut PacMan = 0 as *mut PacMan;

#[no_mangle]
pub unsafe extern "C" fn game_load(platform: *mut bridge::Platform) {
    env_logger::init();
    bridge::init(platform);
    GAME = Box::into_raw(Box::new(PacMan::load()));
}

#[no_mangle]
pub unsafe extern "C" fn game_quit() {
    Box::from_raw(GAME);
    GAME = 0 as *mut PacMan;
}

#[no_mangle]
pub unsafe extern "C" fn game_render() {
    let game = &mut *GAME;
    game.render();
}

#[no_mangle]
pub unsafe extern "C" fn game_on_platform_event(event: *mut bridge::PlatformEvent) {
    let game = &mut *GAME;
    let event = &*event;
    game.on_platform_event(event);
}