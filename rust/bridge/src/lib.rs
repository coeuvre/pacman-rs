pub mod bridge;

pub use bridge::get_gl_proc_address;

pub enum PlatformEvent {
    Close,
    Render,
    Resized { width: i32, height: i32 }
}

pub trait Game {
    fn load() -> Self;
    fn on_platform_event(&mut self, event: &PlatformEvent);
}

pub fn get_performance_counter() -> u64 {
    unsafe { bridge::get_performance_counter() }
}

pub fn get_performance_frequency() -> u64 {
    unsafe { bridge::get_performance_frequency() }
}

pub fn quit() {
    unsafe { bridge::quit() }
}

pub fn to_platform_event(event: &bridge::PlatformEvent) -> Option<PlatformEvent> {
    match event.kind {
        bridge::PLATFORM_EVENT_CLOSE => Some(PlatformEvent::Close),
        bridge::PLATFORM_EVENT_RENDER => Some(PlatformEvent::Render),
        bridge::PLATFORM_EVENT_RESIZE => Some(PlatformEvent::Resized {
            width: unsafe { event.data.resize.width },
            height: unsafe { event.data.resize.height },
        }),
        _ => None
    }
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
            if let Some(event) = $crate::to_platform_event(&*event) {
                game.on_platform_event(&event);
            }
        }
    }
}
