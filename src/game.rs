use failure::Error;

use crate::renderer::*;
use crate::bitmap::*;
use crate::math::*;

pub struct Input {
    pub dt: f32
}

pub struct GameState {
    test_texture: Texture,
}

impl GameState {
    pub fn new(renderer: &mut Renderer) -> Result<Self, Error> {
        let bitmap = Bitmap::from_url("assets://test.png")?;
        let texture = renderer.load_texture(&bitmap);

        Ok(GameState {
            test_texture: texture
        })
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        let mut buffer = Vec::new();
        {
            let texture = self.test_texture.clone();
            buffer.push(RenderCommand::RenderTexturedRect2(TexturedRect2 {
                src: Rect2::with_min_size(Vec2::new(0.0, 0.0), Vec2::new(texture.width as Scalar, texture.height as Scalar)),
                texture,
                dst: Rect2::with_min_size(Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0)),
            }));
        }
        renderer.render(&buffer);
    }
}