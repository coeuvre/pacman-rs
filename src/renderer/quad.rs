use super::*;

#[derive(Clone)]
pub struct Quad {
    pub texture: Option<TextureHandle>,
    pub src: Rect2,
    pub dst: Rect2,
    pub color: Vec4,
}

pub struct RenderQuadProgram {
    program: Program,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    white_texture: TextureHandle,
}

#[repr(C)]
struct RenderQuadVertexAttrib {
    pos: [GLfloat; 2],
    tex_coord: [GLfloat; 2],
    color: [GLfloat; 4],
}

static RENDER_QUAD_VERTEX_SHADER: &str = r#"
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

static RENDER_QUAD_FRAGMENT_SHADER: &str = r#"
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

impl RenderQuadProgram {
    pub fn load() -> Result<RenderQuadProgram, Error> {
        let mut program = Program::new();
        let mut vs = Shader::new(gl::VERTEX_SHADER);
        vs.compile(RENDER_QUAD_VERTEX_SHADER)?;
        program.attach(&vs);

        let mut fs = Shader::new(gl::FRAGMENT_SHADER);
        fs.compile(RENDER_QUAD_FRAGMENT_SHADER)?;
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
                size_of::<RenderQuadVertexAttrib>() as i32,
                offset_of!(RenderQuadVertexAttrib, pos) as *const _
            );
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(
                1, 2, gl::FLOAT, gl::FALSE,
                size_of::<RenderQuadVertexAttrib>() as i32,
                offset_of!(RenderQuadVertexAttrib, tex_coord) as *const _
            );
            gl::EnableVertexAttribArray(1);

            gl::VertexAttribPointer(
                2, 4, gl::FLOAT, gl::FALSE,
                size_of::<RenderQuadVertexAttrib>() as i32,
                offset_of!(RenderQuadVertexAttrib, color) as *const _
            );
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);

            program.active();
            {
                let uniform_name = CString::new("uniform_texture0").unwrap();
                gl::Uniform1i(gl::GetUniformLocation(program.id, uniform_name.as_ptr() as *const _), 0);
            }
        }

        let white_bitmap = Bitmap {
            width: 1,
            height: 1,
            stride: 1,
            pixels: Pixels::A8(vec![255]),
        };
        let white_texture = load_texture(&white_bitmap);

        Ok(RenderQuadProgram {
            program,
            vao,
            vbo,
            ebo,
            white_texture,
        })
    }

    pub fn render(&mut self, viewport_size: Vec2, data_array: &[&Quad]) {
        if data_array.is_empty() {
            return
        }

        let first_data_texture = &data_array[0].texture;
        let texture = if let Some(texture) = first_data_texture {
            texture
        } else {
            &self.white_texture
        };

        let inv_viewport_size = 1.0 / viewport_size;

        let mut vertices = Vec::new();
        let mut indices = Vec::<GLuint>::new();
        for data in data_array {
            assert_eq!(first_data_texture, &data.texture);

            let dst_rect_size = data.dst.size();
            let dst_rect_min = Vec2::new(data.dst.min.x, viewport_size.y - data.dst.min.y - dst_rect_size.y);
            let dst_rect_max = Vec2::new(data.dst.max.x, viewport_size.y - data.dst.min.y);
            let min = dst_rect_min.hadamard(inv_viewport_size) * 2.0 - Vec2::new(1.0, 1.0);
            let max = dst_rect_max.hadamard(inv_viewport_size) * 2.0 - Vec2::new(1.0, 1.0);

            let src_rect_size = data.src.size();
            let texture_size = texture.size().as_vec2();
            let inv_texture_size = 1.0 / texture_size;
            let src_rect_min = Vec2::new(data.src.min.x, texture_size.y - data.src.min.y - src_rect_size.y);
            let src_rect_max = Vec2::new(data.src.max.x, texture_size.y - data.src.min.y);
            let tex_min = src_rect_min.hadamard(inv_texture_size);
            let tex_max = src_rect_max.hadamard(inv_texture_size);

            let vertex_index = vertices.len() as u32;

            // Bottom Left
            vertices.push(RenderQuadVertexAttrib {
                pos: [min.x, min.y],
                tex_coord: [tex_min.x, tex_min.y],
                color: [data.color.x, data.color.y, data.color.z, data.color.w],
            });

            // Bottom Right
            vertices.push(RenderQuadVertexAttrib {
                pos: [max.x, min.y],
                tex_coord: [tex_max.x, tex_min.y],
                color: [data.color.x, data.color.y, data.color.z, data.color.w],
            });

            // Top Right
            vertices.push(RenderQuadVertexAttrib {
                pos: [max.x, max.y],
                tex_coord: [tex_max.x, tex_max.y],
                color: [data.color.x, data.color.y, data.color.z, data.color.w],
            });

            // Top Left
            vertices.push(RenderQuadVertexAttrib {
                pos: [min.x, max.y],
                tex_coord: [tex_min.x, tex_max.y],
                color: [data.color.x, data.color.y, data.color.z, data.color.w],
            });

            indices.push(vertex_index);
            indices.push(vertex_index + 1);
            indices.push(vertex_index + 2);
            indices.push(vertex_index);
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
            gl::BindTexture(gl::TEXTURE_2D, texture.inner.id);

            self.program.active();

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_INT, null());
        }
    }
}
