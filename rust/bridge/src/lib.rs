pub mod bridge;

pub use bridge::*;

pub trait Game {
    fn load() -> Self;
    fn on_platform_event(&mut self, event: &bridge::PlatformEvent);
}

#[macro_export]
macro_rules! entry {
    ($t:ty) => {
        static mut GAME: *mut $t = 0 as *mut $t;

        #[no_mangle]
        pub unsafe extern "C" fn game_load(platform: *mut $crate::bridge::Platform) {
            use $crate::Game;
            $crate::bridge::init(platform);
            GAME = Box::into_raw(Box::new(<$t>::load()))
        }

        #[no_mangle]
        pub unsafe extern "C" fn game_quit() {
            Box::from_raw(GAME);
            GAME = 0 as *mut $t;
        }

        #[no_mangle]
        pub unsafe extern "C" fn game_on_platform_event(event: *mut $crate::bridge::PlatformEvent) {
            use $crate::Game;
            let game = &mut *GAME;
            let event = &*event;
            game.on_platform_event(event);
        }
    }
}