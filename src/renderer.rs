use std::{
    mem::*,
    ffi::*,
    ptr::*,
    rc::Rc,
};
use failure::{Error, format_err};
use gl::types::*;
use profiler::profile;

mod render_triangle;

use crate::math::*;
use crate::bitmap::*;
use self::render_triangle::*;

pub use self::render_triangle::{Vertex, Index};

pub struct DisplayList {
    pub viewport_size: Vec2,
    commands: Vec<RenderCommand>,
}

impl DisplayList {
    pub fn new() -> DisplayList {
        DisplayList {
            viewport_size: Vec2::zero(),
            commands: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn render_triangles(&mut self, vertices: Vec<Vertex>, indices: Vec<Index>, texture: Option<TextureHandle>) {
        self.commands.push(RenderCommand::RenderTriangles(Triangles {
            vertices,
            indices,
            texture,
        }))
    }
}

#[derive(Clone)]
pub struct Quad {
    pub texture: Option<TextureHandle>,
    pub src: Rect2,
    pub dst: Rect2,
    pub color: Vec4,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextureHandle {
    inner: Rc<OpenGLTexture>,
}

impl TextureHandle {
    #[inline]
    pub fn size(&self) -> Vec2i {
        Vec2i::new(self.inner.width as i32, self.inner.height as i32)
    }

    pub fn whole_texture(self) -> SubTexture {
        SubTexture {
            texture: self.clone(),
            region: Rect2::with_min_size(Vec2::zero(), self.size().as_vec2())
        }
    }
}

#[derive(Clone)]
pub struct SubTexture {
    pub texture: TextureHandle,
    pub region: Rect2,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct OpenGLTexture {
    id: GLuint,
    width: i32,
    height: i32,
}

impl Drop for OpenGLTexture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id); }
    }
}

enum RenderCommand {
//    RenderQuad(Quad),
    RenderTriangles(Triangles)
}

struct Triangles {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
    pub texture: Option<TextureHandle>,
}


pub struct Renderer {
    viewport_size: Vec2,
    white1x1_texture: TextureHandle,
    render_triangle_program: RenderTriangleShader
}

impl Renderer {
    pub fn load<F>(load_fn: F) -> Result<Self, Error> where F: FnMut(&str) -> *const c_void  {
        gl::load_with(load_fn);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Enable(gl::TEXTURE_2D);
            gl::Enable(gl::FRAMEBUFFER_SRGB);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);

            gl::Enable(gl::MULTISAMPLE);
        }

        let white1x1_bitmap = Bitmap {
            width: 1,
            height: 1,
            stride: 1,
            pixels: Pixels::A8(vec![255]),
        };

        let white1x1_texture = load_texture(&white1x1_bitmap);

        Ok(Renderer {
            viewport_size: Vec2::zero(),
            white1x1_texture,
            render_triangle_program: RenderTriangleShader::load()?
        })
    }

    pub fn load_texture(&mut self, bitmap: &Bitmap) -> TextureHandle {
        load_texture(bitmap)
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vec2) {
        self.viewport_size = viewport_size;
    }

    #[profile]
    pub fn render(&mut self, display_list: &DisplayList) {
        unsafe {
            gl::Viewport(0, 0, self.viewport_size.x as i32, self.viewport_size.y as i32);
            gl::Scissor(0, 0, self.viewport_size.x as i32, self.viewport_size.y as i32);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        for command in display_list.commands.iter() {
            match *command {
                RenderCommand::RenderTriangles(ref triangles) => {
                    render_triangles(&mut self.render_triangle_program, &self.white1x1_texture, triangles);
                }
            }
        }
    }
}

fn load_texture(bitmap: &Bitmap) -> TextureHandle {
    const GAMMA: f32 = 2.2;
    const INV_GAMMA: f32 = 1.0 / GAMMA;

    let mut pixels = match bitmap.pixels {
        Pixels::SRGBA8(ref pixels) => {
            pixels.chunks(bitmap.stride as usize).map(|row| {
                row.into_iter().take(bitmap.width as usize).map(|pixel| {
                    let sc = pixel;
                    let scf = Vec4::new(f32::from(sc.r), f32::from(sc.g), f32::from(sc.b), f32::from(sc.a)) / 255.0;
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
        Pixels::A8(ref pixels) => {
            pixels.chunks(bitmap.stride as usize).map(|row| {
                row.into_iter().map(|&a| {
                    let af = f32::from(a) / 255.0;
                    // Premultiply alpha
                    let plc = Vec4::new(1.0 * af, 1.0 * af, 1.0 * af, af);
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

        TextureHandle {
            inner: Rc::new(OpenGLTexture {
                id: texture_id,
                width: bitmap.width,
                height: bitmap.height,
            })
        }
    }
}

fn render_triangles(program: &mut RenderTriangleShader, white1x1_texture: &TextureHandle, triangles: &Triangles) {
    let texture = triangles.texture.as_ref().unwrap_or(white1x1_texture);
    program.set_vertices(&triangles.vertices, &triangles.indices);
    program.render(texture.inner.id, 0, triangles.indices.len());
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
            if success as u8 != gl::TRUE {
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
            if success as u8 != gl::TRUE {
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
