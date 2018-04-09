pub mod bridge;

pub use bridge::{get_gl_proc_address, swap_gl_buffers};

pub enum PlatformEvent {
    Update { dt: f32 },
    Render,
    Close,
    Resized { width: i32, height: i32 }
}

pub trait Runner {
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
    static FRAMETIME: f32 = 0.016;

    match event.kind {
        bridge::PLATFORM_EVENT_UPDATE => Some(PlatformEvent::Update { dt: FRAMETIME }),
        bridge::PLATFORM_EVENT_RENDER => Some(PlatformEvent::Render),
        bridge::PLATFORM_EVENT_CLOSE => Some(PlatformEvent::Close),
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
        static mut RUNNER: *mut $t = 0 as *mut $t;

        #[no_mangle]
        pub unsafe extern "C" fn game_load(platform: *mut $crate::bridge::Platform) {
            use $crate::Runner;
            $crate::bridge::init(platform);
            RUNNER = Box::into_raw(Box::new(<$t>::load()))
        }

        #[no_mangle]
        pub unsafe extern "C" fn game_quit() {
            Box::from_raw(RUNNER);
            RUNNER = 0 as *mut $t;
        }

        #[no_mangle]
        pub unsafe extern "C" fn game_on_platform_event(event: *mut $crate::bridge::PlatformEvent) {
            use $crate::Runner;

            let runner = &mut *RUNNER;
            let event = &*event;
            match event.kind {
                _ => if let Some(event) = $crate::to_platform_event(event) {
                    runner.on_platform_event(&event);
                }
            }
        }
    }
}
