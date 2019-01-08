use std::collections::{
    HashMap,
    hash_map,
};

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
    face: Face,
    count: f32,
    test_texture: TextureHandle,
}

#[derive(Eq, PartialEq, Hash)]
pub struct GlyphKey {
    font_pixel_size: u32,
    ch: usize,
}

pub struct Glyph {
    offset: Vec2,
    advance: Vec2,
    sub_texture: Option<SubTexture>,
}

pub struct Face {
    inner: freetype::Face,
    cache: HashMap<GlyphKey, Glyph>,
}

impl Face {
    pub fn load(ft_library: &freetype::Library, url: &str) -> Result<Self, Error> {
        let face = ft_library.new_face(asset::url_to_path(url), 0)?;
        Ok(Face {
            inner: face,
            cache: HashMap::new(),
        })
    }

    pub fn get_or_load_glyph(&mut self, renderer: &mut Renderer, font_pixel_size: u32, ch: usize) -> Option<&Glyph> {
        let key = GlyphKey { font_pixel_size, ch };

        let glyph = match self.cache.entry(key) {
            hash_map::Entry::Occupied(o) => {
                o.into_mut()
            }
            hash_map::Entry::Vacant(v) => {
                let glyph = load_glyph(&self.inner, renderer, font_pixel_size, ch)?;
                v.insert(glyph)
            }
        };

        Some(glyph)
    }
}

#[profile]
pub fn load_glyph(face: &freetype::Face, renderer: &mut Renderer, font_pixel_size: u32, ch: usize) -> Option<Glyph> {
    face.set_pixel_sizes(0, font_pixel_size).ok()?;
    face.load_char(ch, freetype::face::LoadFlag::RENDER).ok()?;

    let glyph = face.glyph();
    let bitmap = Bitmap::from_glyph(&glyph.bitmap());

    let offset = Vec2::new(glyph.bitmap_left() as Scalar, -glyph.bitmap_top() as Scalar);
    let advance = glyph.advance();
    let advance = Vec2::new((advance.x >> 6) as Scalar, -(advance.y >> 6) as Scalar);

    let glyph = if bitmap.stride > 0 {
        let texture = renderer.load_texture(&bitmap);
        let texture_size = texture.size().as_vec2();
        let region = Rect2::with_min_size(Vec2::new(0.0, 0.0), texture_size);
        Glyph {
            offset,
            advance,
            sub_texture: Some(SubTexture {
                texture,
                region,
            })
        }
    } else {
        Glyph {
            offset,
            advance,
            sub_texture: None,
        }
    };

    Some(glyph)
}

impl GameState {
    pub fn load(renderer: &mut Renderer) -> Result<Self, Error> {
        let bitmap = Bitmap::from_url("assets://test.png")?;
        let texture = renderer.load_texture(&bitmap);
        let ft_library = freetype::Library::init()?;
        let face = Face::load(&ft_library, "assets://test_font.otf")?;
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

//        {
//            let texture = self.test_texture.clone();
//            let texture_size = texture.size().as_vec2();
//            buffer.push(RenderCommand::RenderTexturedRect2(TexturedRect2 {
//                src: Rect2::with_min_size(Vec2::new(0.0, 0.0), texture_size),
//                texture,
//                dst: Rect2::with_min_size(Vec2::new(200.0, 10.0 + 10.0 * self.count), texture_size),
//            }));
//        }

        buffer.push(RenderCommand::RenderQuad(Quad {
            texture: None,
            src: Rect2::with_min_size(Vec2::zero(), Vec2::zero()),
            dst: Rect2::with_min_size(Vec2::zero(), Vec2::new(500.0, 500.0)),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }));

        let font_pixel_size = 16;
        let pos = Vec2::new(10.0, 20.0);

        let last_frame = PROFILER.lock().unwrap().last_frame().cloned();

        if let Some(last_frame) = last_frame {
            render_frame_profile(renderer, buffer, &mut self.face, &last_frame, pos, font_pixel_size);
        }
    }
}

#[profile]
fn render_frame_profile(
    renderer: &mut Renderer,
    buffer: &mut Vec<RenderCommand>,
    face: &mut Face,
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
    face: &mut Face,
    font_pixel_size: u32,
    pos: Vec2,
    text: S
) -> Result<(), Error>
where
    S: AsRef<str>
{

    let mut pen_pos = pos;
    for ch in text.as_ref().chars() {
        if let Some(glyph) = face.get_or_load_glyph(renderer, font_pixel_size, ch as usize) {
            if let Some(ref sub_texture) = glyph.sub_texture {
                buffer.push(RenderCommand::RenderQuad(Quad {
                    src: sub_texture.region,
                    texture: Some(sub_texture.texture.clone()),
                    dst: Rect2::with_min_size(pen_pos + glyph.offset, sub_texture.texture.size().as_vec2()),
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                }));
            }

            pen_pos = pen_pos + glyph.advance;
        }
    }

    Ok(())
}