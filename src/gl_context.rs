use std::ffi::{CStr, CString};

use failure::Error;
use gl;

use bridge;

pub static mut CURRENT_GL_CONTEXT: *mut GlContext = 0 as *mut GlContext;

#[repr(u32)]
pub enum GlCapability {
    FramebufferSrgb = gl::FRAMEBUFFER_SRGB
}

pub struct GlContext {
}

impl GlContext {
    pub fn init() -> Result<(), Error> {
        let gl_context = Box::new(GlContext::new()?);
        unsafe { CURRENT_GL_CONTEXT = Box::into_raw(gl_context); }
        Ok(())
    }

    fn new() -> Result<GlContext, Error> {
        gl::load_with(|s| {
            let cstring = CString::new(s).unwrap();
            unsafe { bridge::get_gl_proc_address(cstring.as_ptr()) }
        });

        let gl_version =
            unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char) };
        info!("OpenGL Version {}", gl_version.to_str().unwrap());

        Ok(GlContext {})
    }

    pub fn current() -> &'static GlContext {
        unsafe {
            if CURRENT_GL_CONTEXT == 0 as *mut GlContext {
                panic!("Failed to get current gl context. Call GlContext::init() first.")
            }

            &*CURRENT_GL_CONTEXT
        }
    }

    pub fn current_mut() -> &'static mut GlContext {
        unsafe {
            if CURRENT_GL_CONTEXT == 0 as *mut GlContext {
                panic!("Failed to get current gl context. Call GlContext::init() first.")
            }

            &mut *CURRENT_GL_CONTEXT
        }
    }

    pub fn enable(&mut self, capability: GlCapability) -> Result<(), Error> {
        unsafe { gl::Enable(capability as u32) }
        Ok(())
    }
}
