use std::{
    ffi::*,
    os::raw::*,
};

pub fn init_gl<F>(load_fn: F) where F: FnMut(&str) -> *const c_void  {
    gl::load_with(load_fn);

    unsafe {
        let version = gl::GetString(gl::VERSION);
        println!("OpenGL version: {}", CStr::from_ptr(version as *const c_char).to_string_lossy());

        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        gl::Enable(gl::TEXTURE_2D);
        gl::Enable(gl::FRAMEBUFFER_SRGB);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);

        gl::Enable(gl::MULTISAMPLE);
    }
}
