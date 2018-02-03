use std::os::raw::c_void;
use std::ffi::CStr;
use gl;

pub trait Platform {
    fn performance_counter() -> u64;
    fn performance_fraquency() -> u64;
}

pub trait Desktop: Platform {
    type Window: Window;

    fn create_window(&mut self) -> Result<Self::Window, String>;
}

#[derive(Debug)]
pub enum WindowEvent {
    CloseRequested,
}

pub trait Window {
    type GlContext: GlContext;

    fn create_gl_context(&mut self) -> Result<Self::GlContext, String>;

    // FIXME(coeuvre): Use generic associated types when it is landed at stable channel
    fn poll_events<'a>(&'a mut self) -> Box<'a + Iterator<Item=WindowEvent>>;
}

pub trait GlContext: Send {
    // FIXME(coeuvre): Use generic associated types when it is landed at stable channel
    fn make_current<'a>(&'a mut self) -> Result<Box<CurrentGlContext + 'a>, String>;
}

pub trait CurrentGlContext<'a> {
    unsafe fn proc_address(&self, name: &str) -> Result<*const c_void, String>;
    fn swap_buffers(&mut self) -> Result<(), String>;
}

fn update<D>(_dt: f32) where D: Desktop {
    println!("Updating at {}", D::performance_counter());
}

fn render() {}

pub fn start_desktop<D>(desktop: &mut D)
where
    D: Desktop,
{
    let mut window = desktop.create_window().unwrap();
    let mut gl_ctx = window.create_gl_context().unwrap();
    let mut current_gl_ctx = gl_ctx.make_current().unwrap();

    unsafe {
        gl::load_with(|s| current_gl_ctx.proc_address(s).unwrap());
    }

    // gl::ClearColor(0.0, 0.0, 0.0, 0.0);
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    let glversion =
        unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char) };
    println!("OpenGL Version {}", glversion.to_str().unwrap());

    'game: loop {
        for event in window.poll_events() {
            match event {
                WindowEvent::CloseRequested => break 'game,
            }
        }

        update::<D>(0.0);

        render();

        current_gl_ctx.swap_buffers().unwrap();
    }
}
