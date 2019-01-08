use failure::Error;
use profiler::*;

use crate::renderer::*;
use crate::bitmap::*;
use crate::math::*;
use crate::asset;

pub struct Input {
    pub dt: f32
}

pub struct GameState {
    face: freetype::Face,
    count: f32,
    test_texture: TextureHandle,
}


impl GameState {
    pub fn load(renderer: &mut Renderer) -> Result<Self, Error> {
        let bitmap = Bitmap::from_url("assets://test.png")?;
        let texture = renderer.load_texture(&bitmap);
        let ft_library = freetype::Library::init()?;
        let face = ft_library.new_face(asset::url_to_path("assets://test_font.otf"), 0)?;
        Ok(GameState {
            face,
            count: 0.0,
            test_texture: texture,
        })
    }

    #[profile]
    pub fn update(&mut self, input: &Input, renderer: &mut Renderer, render_command_buffer: &mut Vec<RenderCommand>) {
        let buffer = render_command_buffer;

        self.count += input.dt;

        {
            let texture = self.test_texture.clone();
            let texture_size = texture.size().as_vec2();
            buffer.push(RenderCommand::RenderTexturedRect2(TexturedRect2 {
                src: Rect2::with_min_size(Vec2::new(0.0, 0.0), texture_size),
                texture,
                dst: Rect2::with_min_size(Vec2::new(200.0, 10.0 + 10.0 * self.count), texture_size),
            }));
        }

        let font_pixel_size = 16;
        let pos = Vec2::new(10.0, 20.0);

        let last_frame = PROFILER.lock().unwrap().last_frame().cloned();

        if let Some(last_frame) = last_frame {
            render_profile_info(renderer, buffer, &mut self.face, &last_frame, pos, font_pixel_size);
        }
    }
}

#[profile]
fn render_profile_info(
    renderer: &mut Renderer,
    buffer: &mut Vec<RenderCommand>,
    face: &mut freetype::Face,
    last_frame: &Frame,
    mut pos: Vec2,
    font_pixel_size: u32,
) {
    for block in last_frame.dfs_block_iter() {
        let data = block.data();
        let delta_ms = data.delta() * 1000.0;
        let block_pos = pos + Vec2::new(20.0 * (block.level() - 1) as f32, 0.0);
        let text = if block.level() > 1 {
            format!("{:.2} ms {} ({}:{})", delta_ms, data.name(), data.file(), data.line())
        } else {
            format!("{:.2} ms", delta_ms)
        };
        render_text_line(renderer, buffer, face, font_pixel_size, block_pos, text).unwrap();
        pos.y = pos.y + font_pixel_size as f32;
    }
}

fn render_text_line<S>(
    renderer: &mut Renderer,
    buffer: &mut Vec<RenderCommand>,
    face: &mut freetype::Face,
    font_pixel_size: u32,
    pos: Vec2,
    text: S
) -> Result<(), Error>
where
    S: AsRef<str>
{
    face.set_pixel_sizes(0, font_pixel_size)?;

    let mut pen_pos = pos;
    for char in text.as_ref().chars() {
        face.load_char(char as usize, freetype::face::LoadFlag::RENDER).unwrap();
        let glyph = face.glyph();
        let bitmap = Bitmap::from_glyph(&glyph.bitmap());
        if bitmap.stride > 0 {
            let texture = renderer.load_texture(&bitmap);
            let texture_size = texture.size().as_vec2();
            buffer.push(RenderCommand::RenderTexturedRect2(TexturedRect2 {
                src: Rect2::with_min_size(Vec2::new(0.0, 0.0), texture_size),
                texture,
                dst: Rect2::with_min_size(pen_pos + Vec2::new(glyph.bitmap_left() as Scalar, -glyph.bitmap_top() as Scalar), texture_size),
            }));
        }

        let advance = glyph.advance();
        pen_pos = pen_pos + Vec2::new((advance.x >> 6) as Scalar, -(advance.y >> 6) as Scalar);
    }

    Ok(())
}