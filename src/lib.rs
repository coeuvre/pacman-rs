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
    vbo: gl::types::GLuint,
}

impl PacMan {
    pub fn new() -> PacMan {
        let mut vbo = 0;

        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        }

        PacMan {
            frame: 0,
            frequency: bridge::get_performance_frequency(),
            last_counter: bridge::get_performance_counter(),
            vbo,
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

    fn update(&mut self, _dt: f32) {
        let current_counter = bridge::get_performance_counter();
        let delta =
            ((current_counter - self.last_counter) as f64 / self.frequency as f64) as f32;
        self.last_counter = current_counter;
        self.frame = self.frame + 1;
        info!("Update for frame {}, delta {}", self.frame, delta);
    }

    fn render(&self) {
        info!("Rendering frame {}", self.frame);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            bridge::swap_gl_buffers();
        }
    }

    fn on_platform_event(&mut self, event: &PlatformEvent) {
        match *event {
            PlatformEvent::Close => bridge::quit(),

            PlatformEvent::Resized { width, height } => unsafe {
                info!("Resizing {}x{}", width, height);
                gl::Viewport(0, 0, width, height);
            }
        }
    }
}

entry!(PacMan);