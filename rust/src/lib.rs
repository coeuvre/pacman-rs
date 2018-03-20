extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
extern crate bridge;
extern crate gl;
extern crate stb;

pub mod image;

use std::ffi::{CStr, CString};
use bridge::PlatformEvent;

pub struct PacMan {
    frame: u64,
    frequency: u64,
    last_counter: u64,
    render_count: u64,
}

impl PacMan {
    pub fn new() -> PacMan {
        PacMan {
            frame: 0,
            frequency: bridge::get_performance_frequency(),
            last_counter: bridge::get_performance_counter(),
            render_count: 0,
        }
    }

    pub fn update(&mut self, _dt: f32) {}

    pub fn render(&self) {
        unsafe {
            gl::ClearColor((self.frame as f32 / 255.0).min(1.0), 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

impl bridge::Game for PacMan {
    fn load() -> PacMan {
        env_logger::init();

        gl::load_with(|s| {
            let cstring = CString::new(s).unwrap();
            unsafe { bridge::get_gl_proc_address(cstring.as_ptr()) }
        });

        let glversion =
            unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char) };
        info!("OpenGL Version {}", glversion.to_str().unwrap());

        if let Err(e) = image::Image::load("example.png") {
            error!("An error occurs: {}", e);
        }

        PacMan::new()
    }

    fn on_platform_event(&mut self, event: &PlatformEvent) {
        match *event {
            bridge::PlatformEvent::Render => {
                static FRAMETIME: f32 = 0.016;

                let current_counter = bridge::get_performance_counter();
                let delta =
                    ((current_counter - self.last_counter) as f64 / self.frequency as f64) as f32;
                if delta + 0.001 >= FRAMETIME {
                    self.last_counter = current_counter;
                    self.update(FRAMETIME);
                    self.frame = self.frame + 1;
                    //info!("Update game state for frame {}, delta {}", self.frame, delta);
                }
                // info!("Render frame {}", self.frame);
                self.render();
                self.render_count = self.render_count + 1;
            }
            bridge::PlatformEvent::Close => bridge::quit(),
            bridge::PlatformEvent::Resized { width, height } => unsafe {
                info!("Resizing {}x{}", width, height);
                gl::Enable(gl::SCISSOR_TEST);
                gl::Scissor(0, 0, width / 2, height / 2);
            },
        }
    }
}

entry!(PacMan);
