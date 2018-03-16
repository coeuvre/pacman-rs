use std::ffi::{CStr, CString};
use bridge;
use bridge::PlatformEvent;
use gl;

use super::Game;
use super::image;

pub struct PacMan {
    frame: u64,
    frequency: u64,
    last_counter: u64
}

impl PacMan {
    pub fn update(&mut self, _dt: f32) {

    }

    pub fn render(&self) {
        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

impl Game for PacMan {
    fn load() -> PacMan {
        gl::load_with(|s| {
            let cstring = CString::new(s).unwrap();
            unsafe { bridge::get_gl_proc_address(cstring.as_ptr()) }
        });

        let glversion = unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char) };
        info!("OpenGL Version {}", glversion.to_str().unwrap());

        if let Err(e) = image::Image::load("example.png") {
            error!("An error occurs: {}", e);
        }

        PacMan {
            frame: 0,
            frequency: unsafe { bridge::get_performance_frequency() },
            last_counter: unsafe { bridge::get_performance_counter() },
        }
    }

    fn on_platform_event(&mut self, event: &PlatformEvent) {
        info!("{:?}", event);
        match event.kind {
            bridge::PLATFORM_EVENT_RENDER => {
                static FRAMETIME: f32 = 0.016;

                let current_counter = unsafe { bridge::get_performance_counter() };
                let delta = ((current_counter - self.last_counter) as f64 / self.frequency as f64) as f32;
                if delta + 0.001 >= FRAMETIME {
                    self.last_counter = current_counter;
                    self.update(FRAMETIME);
                    self.frame = self.frame + 1;
                    info!("Update game state for frame {}, delta {}", self.frame, delta);
                }
                info!("Render frame {}", self.frame);
                self.render();
            },
            bridge::PLATFORM_EVENT_CLOSE => unsafe { bridge::quit() },
            _ => {}
        }
    }
}