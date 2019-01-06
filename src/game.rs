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
    test_texture: Texture,
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
        {
            let texture = self.test_texture.clone();
            let texture_size = Vec2::new(texture.width as Scalar, texture.height as Scalar);
            buffer.push(RenderCommand::RenderTexturedRect2(TexturedRect2 {
                src: Rect2::with_min_size(Vec2::new(0.0, 0.0), texture_size),
                texture,
                dst: Rect2::with_min_size(Vec2::new(10.0, 10.0), texture_size),
            }));
        }

        {
            self.face.set_pixel_sizes(0, 16).unwrap();
            self.face.load_char('B' as usize, freetype::face::LoadFlag::RENDER).unwrap();
            let glyph = self.face.glyph();
            let bitmap = Bitmap::from_glyph(&glyph.bitmap());
            let texture = renderer.load_texture(&bitmap);
            let texture_size = Vec2::new(texture.width as Scalar, texture.height as Scalar);
            buffer.push(RenderCommand::RenderTexturedRect2(TexturedRect2 {
                src: Rect2::with_min_size(Vec2::new(0.0, 0.0), texture_size),
                texture,
                dst: Rect2::with_min_size(Vec2::new(10.0, 10.0), texture_size),
            }));
        }

        renderer.render(&buffer);
    }
}