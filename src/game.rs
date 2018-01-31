use std::os::raw::c_void;
use std::ffi::CStr;
use gl;

pub trait GlContext<'a, C>: Send
where
    C: CurrentGlContext<'a>,
{
    fn make_current(&'a mut self) -> Result<C, String>;
}

pub trait CurrentGlContext<'a> {
    unsafe fn get_proc_address(&self, name: &str) -> Result<*const c_void, String>;
    fn swap_buffers(&mut self) -> Result<(), String>;
}

pub fn start<'a, G, C>(gl_ctx: &'a mut G)
where
    G: GlContext<'a, C>,
    C: CurrentGlContext<'a>,
{
    let mut current_gl_ctx = gl_ctx.make_current().unwrap();

    unsafe {
        gl::load_with(|s| current_gl_ctx.get_proc_address(s).unwrap());
    }

    // gl::ClearColor(0.0, 0.0, 0.0, 0.0);
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    let glversion =
        unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char) };
    println!("OpenGL Version {}", glversion.to_str().unwrap());

    'game: loop {
        current_gl_ctx.swap_buffers().unwrap();
    }
}
