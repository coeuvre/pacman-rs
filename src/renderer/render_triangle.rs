use super::*;

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub pos: Vec2,
    pub tex_coord: Vec2,
    pub color: Vec4,
}

pub type Index = u32;

pub struct RenderTriangleShader {
    program: Program,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

static RENDER_TRIANGLE_VERTEX_SHADER: &str = r#"
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

static RENDER_TRIANGLE_FRAGMENT_SHADER: &str = r#"
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

impl RenderTriangleShader {
    pub fn load() -> Result<RenderTriangleShader, Error> {
        let mut program = Program::new();
        let mut vs = Shader::new(gl::VERTEX_SHADER);
        vs.compile(RENDER_TRIANGLE_VERTEX_SHADER)?;
        program.attach(&vs);

        let mut fs = Shader::new(gl::FRAGMENT_SHADER);
        fs.compile(RENDER_TRIANGLE_FRAGMENT_SHADER)?;
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

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0, 2, gl::FLOAT, gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, pos) as *const _
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1, 2, gl::FLOAT, gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, tex_coord) as *const _
            );

            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2, 4, gl::FLOAT, gl::FALSE,
                size_of::<Vertex>() as i32,
                offset_of!(Vertex, color) as *const _
            );

            gl::BindVertexArray(0);

            program.active();
            {
                let uniform_name = CString::new("uniform_texture0").unwrap();
                gl::Uniform1i(gl::GetUniformLocation(program.id, uniform_name.as_ptr() as *const _), 0);
            }
        }

        Ok(RenderTriangleShader {
            program,
            vao,
            vbo,
            ebo,
        })
    }

    pub fn upload_data(&mut self, vertices: &[Vertex], indices: &[Index]) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<Vertex>() * vertices.len()) as isize,
                vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (size_of::<Index>() * indices.len()) as isize,
                indices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
        }
    }

    pub fn render(&mut self, texture_id: GLuint, start: usize, len: usize) {
        unsafe {
//            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            self.program.active();
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, len as i32, gl::UNSIGNED_INT, start as *const _);
            gl::BindVertexArray(0);
        }
    }
}
