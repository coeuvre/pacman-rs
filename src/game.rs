use std::os::raw::c_void;
use std::ffi::CStr;
use gl;

pub trait GlContext: Send {
    // FIXME(coeuvre): Use generic associated types when it is landed at stable channel
    fn make_current<'a>(&'a mut self) -> Result<Box<CurrentGlContext + 'a>, String>;
}

pub trait CurrentGlContext<'a> {
    unsafe fn proc_address(&self, name: &str) -> Result<*const c_void, String>;
}

pub trait Platform {
    fn performance_counter() -> u64;
    fn performance_fraquency() -> u64;
}

pub trait GlDesktop: Platform {
    type GlWindow: GlWindow;

    fn create_window(&mut self) -> Result<Self::GlWindow, String>;
}

pub trait GlWindow {
    type GlContext: GlContext;

    fn create_gl_context(&mut self) -> Result<Self::GlContext, String>;
    fn swap_buffers(&mut self) -> Result<(), String>;
}

fn update(_dt: f32) {}

fn render() {}

pub fn start_desktop<D>(desktop: &mut D)
where
    D: GlDesktop,
{
    println!("Starting desktop");

    let mut window = desktop.create_window().unwrap();
    let mut gl_ctx = window.create_gl_context().unwrap();
    let current_gl_ctx = gl_ctx.make_current().unwrap();

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

    // 'game: loop {
    //     update(0.0);

    //     render();

    //     window.swap_buffers().unwrap();
    // }
}
