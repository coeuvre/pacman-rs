use failure::Error;

use crate::renderer::*;
use crate::bitmap::*;
use crate::math::*;
use crate::asset;

pub struct Input {
    pub dt: f32
}

pub struct GameState {
    face: freetype::Face,
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
            test_texture: texture,
        })
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        let mut buffer = Vec::new();
//        {
//            let texture = self.test_texture.clone();
//            let texture_size = texture.size().as_vec2();
//            buffer.push(RenderCommand::RenderTexturedRect2(TexturedRect2 {
//                src: Rect2::with_min_size(Vec2::new(0.0, 0.0), texture_size),
//                texture,
//                dst: Rect2::with_min_size(Vec2::new(10.0, 10.0), texture_size),
//            }));
//        }

        let font_pixel_size = 16;
        render_text_line(renderer, &mut buffer, &mut self.face, font_pixel_size, "Hello World", Vec2::new(10.0, 20.0)).unwrap();

        renderer.render(&buffer);
    }
}

fn render_text_line(renderer: &mut Renderer, buffer: &mut Vec<RenderCommand>,
                    face: &mut freetype::Face, font_pixel_size: u32, text: &str,
                    pos: Vec2) -> Result<(), Error> {
    let mut pen_pos = pos;
    face.set_pixel_sizes(0, font_pixel_size)?;

    for char in text.chars() {
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