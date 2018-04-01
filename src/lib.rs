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
use std::os::raw::*;
use gl::types::*;
use bridge::PlatformEvent;
use failure::{err_msg, Error};

static VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec2 a_tex_coord;

out vec2 tex_coord;

void main() {
    gl_Position = vec4(a_pos.x, a_pos.y, a_pos.z, 1.0);
    tex_coord = a_tex_coord;
}
"#;

static FRAGMENT_SHADER: &str = r#"
#version 330 core

uniform sampler2D texture0;

in vec2 tex_coord;
out vec4 frag_color;

void main() {
    frag_color = texture(texture0, tex_coord);
}
"#;

#[derive(Debug)]
pub enum GlShader {
    VertexShader(GLuint),
    FragmentShader(GLuint),
}

impl GlShader {
    unsafe fn compile_shader(kind: GLenum, source: *const GLchar) -> Result<GLuint, Error> {
        let shader = gl::CreateShader(kind);
        gl::ShaderSource(shader, 1, &source, 0 as *const GLint);
        gl::CompileShader(shader);
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 1 {
            Ok(shader)
        } else {
            gl::DeleteShader(shader);
            let mut log = ['\0'; 512];
            let ptr = (&mut log[..]).as_mut_ptr() as *mut GLchar;
            gl::GetShaderInfoLog(shader, 512, 0 as *mut GLsizei, ptr);
            Err(err_msg(format!(
                "Failed to compile shader {}",
                CStr::from_ptr(ptr).to_str().unwrap()
            )))
        }
    }

    pub fn compile_vertex_shader<T: Into<Vec<u8>>>(source: T) -> Result<GlShader, Error> {
        let cstring = CString::new(source)?;
        let id =
            unsafe { Self::compile_shader(gl::VERTEX_SHADER, cstring.as_ptr() as *const GLchar)? };
        Ok(GlShader::VertexShader(id))
    }

    pub fn compile_fragment_shader<T: Into<Vec<u8>>>(source: T) -> Result<GlShader, Error> {
        let cstring = CString::new(source)?;
        let id = unsafe {
            Self::compile_shader(gl::FRAGMENT_SHADER, cstring.as_ptr() as *const GLchar)?
        };
        Ok(GlShader::FragmentShader(id))
    }

    pub fn id(&self) -> GLuint {
        match *self {
            GlShader::VertexShader(id) => id,
            GlShader::FragmentShader(id) => id,
        }
    }
}

impl Drop for GlShader {
    fn drop(&mut self) {
        trace!("Drop GlShader::{:?}", self);
        unsafe {
            gl::DeleteShader(self.id());
        }
    }
}

#[derive(Debug)]
pub struct GlProgram(GLuint);

impl GlProgram {
    pub fn link(shaders: &[GlShader]) -> Result<GlProgram, Error> {
        unsafe {
            let program = gl::CreateProgram();

            for shader in shaders.iter() {
                gl::AttachShader(program, shader.id());
            }

            gl::LinkProgram(program);

            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 1 {
                Ok(GlProgram(program))
            } else {
                let mut log = ['\0'; 512];
                let ptr = (&mut log[..]).as_mut_ptr() as *mut GLchar;
                gl::GetProgramInfoLog(program, 512, 0 as *mut GLsizei, ptr);
                Err(err_msg(format!(
                    "Failed to compile shader {}",
                    CStr::from_ptr(ptr).to_str().unwrap()
                )))
            }
        }
    }

    pub fn active(&mut self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }
}

impl Drop for GlProgram {
    fn drop(&mut self) {
        trace!("Drop {:?}", self);
        unsafe { gl::DeleteProgram(self.0) }
    }
}

pub struct PacMan {
    frame: u64,
    frequency: u64,
    last_counter: u64,
    program: GlProgram,
    texture: GLuint,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

impl PacMan {
    pub fn new() -> PacMan {
        let vertices = [
            // Top Right
            1.0 as GLfloat,
            1.0,
            0.0,
            1.0,
            1.0,
            // Bottom Right
            1.0,
            -1.0,
            0.0,
            1.0,
            0.0,
            // Bottom Left
            -1.0,
            -1.0,
            0.0,
            0.0,
            0.0,
            // Top Left
            -1.0,
            1.0,
            0.0,
            0.0,
            1.0,
        ];
        let indices = [
            0 as GLuint,
            1,
            3, // First Triangle
            1,
            2,
            3, // Second Triangle
        ];
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        let shaders = [
            GlShader::compile_vertex_shader(VERTEX_SHADER).unwrap(),
            GlShader::compile_fragment_shader(FRAGMENT_SHADER).unwrap(),
        ];
        let mut program = GlProgram::link(&shaders).unwrap();
        program.active();

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of_val(&vertices)) as GLsizeiptr,
                vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (std::mem::size_of_val(&indices)) as GLsizeiptr,
                indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (5 * std::mem::size_of::<GLfloat>()) as i32,
                0 as *const c_void,
            );
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * std::mem::size_of::<GLfloat>()) as i32,
                (3 * std::mem::size_of::<GLfloat>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);

            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        }

        let image = image::Image::load_and_flip("pacman.png").unwrap();
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::SRGB_ALPHA as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.date().as_ptr() as *const c_void,
            );
        }

        PacMan {
            frame: 0,
            frequency: bridge::get_performance_frequency(),
            last_counter: bridge::get_performance_counter(),
            program,
            texture,
            vao,
            vbo,
            ebo,
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

        unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB); }

        PacMan::new()
    }

    fn update(&mut self, _dt: f32) {
        let current_counter = bridge::get_performance_counter();
        let delta = ((current_counter - self.last_counter) as f64 / self.frequency as f64) as f32;
        self.last_counter = current_counter;
        self.frame = self.frame + 1;
        trace!("Update for frame {}, delta {}", self.frame, delta);
    }

    fn render(&self) {
        trace!("Rendering frame {}", self.frame);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
            gl::BindVertexArray(0);

            bridge::swap_gl_buffers();
        }
    }

    fn on_platform_event(&mut self, event: &PlatformEvent) {
        match *event {
            PlatformEvent::Close => bridge::quit(),

            PlatformEvent::Resized { width, height } => unsafe {
                trace!("Resizing {}x{}", width, height);
                gl::Viewport(0, 0, width, height);
            },
        }
    }
}

entry!(PacMan);
