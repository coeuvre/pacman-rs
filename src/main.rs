extern crate gl;
extern crate platform;

use std::ffi::CStr;

use platform::WindowEvent;

fn update(_dt: f32) {
    println!("Updating at {}", platform::performance_counter());
}

fn render() {}

fn main() {
    platform::init().unwrap();

    let mut window = platform::Window::new().unwrap();
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

        update(0.0);

        render();

        current_gl_ctx.swap_buffers().unwrap();
    }
}
