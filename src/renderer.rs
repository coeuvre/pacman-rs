use std::{
    mem::*,
    ffi::*,
    ptr::*
};
use failure::{Error, format_err};
use gl::types::*;

use crate::math::*;
use crate::bitmap::*;

#[derive(Clone)]
pub struct Texture {
    pub handle: GLuint,
    pub width: u32,
    pub height: u32,
}

pub struct TexturedRect2 {
    pub texture: Texture,
    pub src: Rect2,
    pub dst: Rect2,
}

pub enum RenderCommand {
    RenderTexturedRect2(TexturedRect2),
}

pub struct Renderer {
    viewport_size: Vec2,
    render_textured_rect2_program: RenderTexturedRect2Program
}

impl Renderer {
    pub fn new<F>(load_fn: F) -> Result<Renderer, Error> where F: FnMut(&str) -> *const c_void  {
        gl::load_with(load_fn);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Enable(gl::TEXTURE_2D);
            gl::Enable(gl::FRAMEBUFFER_SRGB);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);

            gl::Enable(gl::MULTISAMPLE);
        }

        Ok(Renderer {
            viewport_size: Vec2::zero(),
            render_textured_rect2_program: RenderTexturedRect2Program::new()?
        })
    }

    pub fn load_texture(&mut self, bitmap: &Bitmap) -> Texture {
        const GAMMA: f32 = 2.2;
        const INV_GAMMA: f32 = 1.0 / GAMMA;

        let mut pixels = match bitmap.pixels {
            Pixels::SRGBA8(ref pixels) => {
                pixels.chunks(bitmap.stride as usize).map(|row| {
                    row.into_iter().take(bitmap.width as usize).map(|pixel| {
                        let sc = pixel;
                        let scf = Vec4::new(sc.r as f32, sc.g as f32, sc.b as f32, sc.a as f32) / 255.0;
                        // sRGB to lRGB
                        let lc = Vec4::new(scf.x.powf(GAMMA), scf.y.powf(GAMMA), scf.z.powf(GAMMA), scf.w);
                        // Premultiply alpha
                        let plc = Vec4::from_xyz(lc.xyz() * lc.w, lc.w);
                        // lRGB to sRGB
                        let pscf = Vec4::new(plc.x.powf(INV_GAMMA), plc.y.powf(INV_GAMMA), plc.z.powf(INV_GAMMA), plc.w);
                        let psc = pscf * 255.0;
                        vec![psc.x.round() as u8, psc.y.round() as u8, psc.z.round() as u8, psc.w.round() as u8]
                    }).flatten().collect::<Vec<u8>>()
                }).collect::<Vec<Vec<u8>>>()
            }
        };

        // Flip bitmap
        pixels.reverse();

        let bytes = pixels.into_iter().flatten().collect::<Vec<u8>>();

        unsafe {
            let mut texture_id = zeroed();
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::SRGB8_ALPHA8 as i32,
                bitmap.width as i32, bitmap.height as i32, 0,
                gl::RGBA, gl::UNSIGNED_BYTE, bytes.as_ptr() as *const _,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            Texture {
                handle: texture_id,
                width: bitmap.width,
                height: bitmap.height,
            }
        }
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vec2) {
        self.viewport_size = viewport_size;
    }

    pub fn render(&mut self, buffer: &[RenderCommand]) {
        for command in buffer.iter() {
            match command {
                RenderCommand::RenderTexturedRect2(textured_rect2) => {
                    let data = [textured_rect2];
                    self.render_textured_rect2_program.render(self.viewport_size, &data);
                }
            }
        }
    }
}

macro_rules! offset_of {
    ($ty:ty, $field:tt) => ({
        let base = 0 as *const $ty;
        let field = &(*base).$field as *const _;
        field as usize - base as usize
    });
}

#[repr(C)]
struct RenderTexturedRect2VertexAttrib {
    pos: [GLfloat; 2],
    tex_coord: [GLfloat; 2],
    color: [GLfloat; 4],
}

pub struct RenderTexturedRect2Program {
    program: Program,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

static RENDER_TEXTURED_RECT2_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec2 attrib_pos;
layout (location = 1) in vec2 attrib_tex_coord;
layout (location = 2) in vec4 attrib_color;

out vec2 vertex_tex_coord;
out vec4 vertex_color;

void main()
{
    gl_Position = vec4(attrib_pos, 0, 1);
    vertex_tex_coord = attrib_tex_coord;
    vertex_color = attrib_color;
}
"#;

static RENDER_TEXTURED_RECT2_FRAGMENT_SHADER: &str = r#"
#version 330 core

uniform sampler2D uniform_texture0;

in vec2 vertex_tex_coord;
in vec4 vertex_color;

out vec4 frag_color;

void main()
{
    vec4 texel = texture(uniform_texture0, vertex_tex_coord);
    frag_color = texel * vertex_color;
}
"#;

impl RenderTexturedRect2Program {
    pub fn new() -> Result<RenderTexturedRect2Program, Error> {
        let mut program = Program::new();
        let mut vs = Shader::new(gl::VERTEX_SHADER);
        vs.compile(RENDER_TEXTURED_RECT2_VERTEX_SHADER)?;
        program.attach(&vs);

        let mut fs = Shader::new(gl::FRAGMENT_SHADER);
        fs.compile(RENDER_TEXTURED_RECT2_FRAGMENT_SHADER)?;
        program.attach(&fs);

        program.link()?;

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            gl::VertexAttribPointer(
                0, 2, gl::FLOAT, gl::FALSE,
                size_of::<RenderTexturedRect2VertexAttrib>() as i32,
                offset_of!(RenderTexturedRect2VertexAttrib, pos) as *const _
            );
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(
                1, 2, gl::FLOAT, gl::FALSE,
                size_of::<RenderTexturedRect2VertexAttrib>() as i32,
                offset_of!(RenderTexturedRect2VertexAttrib, tex_coord) as *const _
            );
            gl::EnableVertexAttribArray(1);

            gl::VertexAttribPointer(
                2, 4, gl::FLOAT, gl::FALSE,
                size_of::<RenderTexturedRect2VertexAttrib>() as i32,
                offset_of!(RenderTexturedRect2VertexAttrib, color) as *const _
            );
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);

            program.active();
            {
                let uniform_name = CString::new("uniform_texture0").unwrap();
                gl::Uniform1i(gl::GetUniformLocation(program.id, uniform_name.as_ptr() as *const _), 0);
            }
        }

        Ok(RenderTexturedRect2Program {
            program,
            vao,
            vbo,
            ebo,
        })
    }

    pub fn render(&mut self, viewport_size: Vec2, data_array: &[&TexturedRect2]) {
        if data_array.is_empty() {
            return
        }

        let texture_id = data_array[0].texture.handle;
        let inv_viewport_size = 1.0 / viewport_size;

        let mut vertices = Vec::new();
        let mut indices = Vec::<GLuint>::new();
        for data in data_array {
            let ref texture = data.texture;
            assert_eq!(texture_id, texture.handle);

            let dst_rect_size = data.dst.size();
            let dst_rect_min = Vec2::new(data.dst.min.x, viewport_size.y - data.dst.min.y - dst_rect_size.y);
            let dst_rect_max = Vec2::new(data.dst.max.x, viewport_size.y - data.dst.min.y);
            let min = dst_rect_min.hadamard(inv_viewport_size) * 2.0 - Vec2::new(1.0, 1.0);
            let max = dst_rect_max.hadamard(inv_viewport_size) * 2.0 - Vec2::new(1.0, 1.0);

            let src_rect_size = data.src.size();
            let src_rect_min = Vec2::new(data.src.min.x, texture.height as f32 - data.src.min.y - src_rect_size.y);
            let src_rect_max = Vec2::new(data.src.max.x, texture.height as f32 - data.src.min.y);
            let inv_texture_size = 1.0 / Vec2::new(texture.width as Scalar, texture.height as Scalar);
            let tex_min = src_rect_min.hadamard(inv_texture_size);
            let tex_max = src_rect_max.hadamard(inv_texture_size);

            let vertex_index = vertices.len() as u32;

            // Bottom Left
            vertices.push(RenderTexturedRect2VertexAttrib {
                pos: [min.x, min.y],
                tex_coord: [tex_min.x, tex_min.y],
                color: [1.0, 1.0, 1.0, 1.0],
            });

            // Bottom Right
            vertices.push(RenderTexturedRect2VertexAttrib {
                pos: [max.x, min.y],
                tex_coord: [tex_max.x, tex_min.y],
                color: [1.0, 1.0, 1.0, 1.0],
            });

            // Top Right
            vertices.push(RenderTexturedRect2VertexAttrib {
                pos: [max.x, max.y],
                tex_coord: [tex_max.x, tex_max.y],
                color: [1.0, 1.0, 1.0, 1.0],
            });

            // Top Left
            vertices.push(RenderTexturedRect2VertexAttrib {
                pos: [min.x, max.y],
                tex_coord: [tex_min.x, tex_max.y],
                color: [1.0, 1.0, 1.0, 1.0],
            });

            indices.push(vertex_index + 0);
            indices.push(vertex_index + 1);
            indices.push(vertex_index + 2);
            indices.push(vertex_index + 0);
            indices.push(vertex_index + 2);
            indices.push(vertex_index + 3);
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of_val(vertices.get_unchecked(0)) * vertices.len()) as isize,
                vertices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (size_of_val(indices.get_unchecked(0)) * indices.len()) as isize,
                indices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            self.program.active();

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_INT, null());
        }
    }
}

struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn new(ty: GLenum) -> Shader {
        Shader {
            id: unsafe { gl::CreateShader(ty) },
        }
    }

    pub fn compile<T: Into<Vec<u8>>>(&mut self, source: T) -> Result<(), Error> {
        let source = CString::new(source)?;
        unsafe {
            gl::ShaderSource(self.id, 1, &source.as_ptr(), null());
            gl::CompileShader(self.id);

            let mut success = 0;
            gl::GetShaderiv(self.id, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as i32 {
                let mut buffer = Vec::with_capacity(512);
                gl::GetShaderInfoLog(self.id, buffer.len() as i32, null_mut(), buffer.as_mut_ptr() as *mut i8);
                Err(format_err!("{}", CString::new(buffer).unwrap().to_string_lossy()))
            } else {
                Ok(())
            }
        }

    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}

struct Program {
    id: GLuint,
}

impl Program {
    pub fn new() -> Program {
        Program {
            id: unsafe { gl::CreateProgram() },
        }
    }

    pub fn attach(&mut self, shader: &Shader) {
        unsafe {
            gl::AttachShader(self.id, shader.id);
        }
    }

    pub fn link(&mut self) -> Result<(), Error> {
        unsafe {
            gl::LinkProgram(self.id);

            let mut success = 0;
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as i32 {
                let mut buffer = Vec::with_capacity(512);
                gl::GetProgramInfoLog(self.id, buffer.len() as i32, null_mut(), buffer.as_mut_ptr() as *mut i8);
                Err(format_err!("{}", CString::new(buffer).unwrap().to_string_lossy()))
            } else {
                Ok(())
            }
        }
    }

    pub fn active(&mut self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}
