use std::collections::{
    HashMap,
    hash_map,
};

use failure::Error;
use profiler::{last_frame, profile, Frame, BlockRef};
use rdraw::*;

use crate::renderer::*;
use crate::bitmap::*;
use crate::math::*;
use crate::asset;

pub struct Input {
    pub dt: f32
}

pub struct GameState {
    canvas: Canvas,
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
    texture_region: Option<TextureRegion>,
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

    #[profile]
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

        Some(glyph as &Glyph)
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
            texture_region: Some(TextureRegion {
                texture,
                region,
            })
        }
    } else {
        Glyph {
            offset,
            advance,
            texture_region: None,
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
            canvas: Canvas::new(),
            face,
            count: 0.0,
            test_texture: texture,
        })
    }

    #[profile]
    pub fn update(&mut self, input: &Input, renderer: &mut Renderer, dl: &mut DisplayList) {
        self.count += input.dt;

        let canvas = &mut self.canvas;

        draw_graph(canvas, dl, 400.0, 50.0, 500.0, 200.0, self.count);

        draw_lines(canvas, dl, 600.0, 300.0, 600.0, 50.0, self.count);

        draw_widths(canvas, dl, 500.0, 300.0, 30.0);
//        {
//            let texture = self.test_texture.clone();
//            let texture_size = texture.size().as_vec2();
//            dl.render_textured_quad(
//                Rect2::with_min_size(Vec2::new(200.0, 10.0 + 10.0 * self.count), texture_size),
//                texture.whole_texture(),
//                Vec4::new(1.0, 1.0, 1.0, 1.0),
//            );
//        }

//        {
//            dl.render_quad(Rect2::with_min_size(Vec2::zero(), Vec2::new(500.0, 500.0)), Vec4::new(1.0, 1.0, 1.0, 1.0));
//        }

        let font_pixel_size = 16;
        let pos = Vec2::new(10.0, 20.0);

        let last_frame = last_frame();

        if let Some(last_frame) = last_frame {
            render_frame_profile(renderer, dl, &mut self.face, &last_frame, pos, font_pixel_size);
        }
    }
}

#[profile]
fn draw_lines(canvas: &mut Canvas, dl: &mut DisplayList, x: Scalar, y: Scalar, w: Scalar, _h: Scalar, t: Scalar) {
    let pad = 5.0;
    let s = w / 9.0 - pad * 2.0;
    let joins = [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel];
    let caps = [LineCap::Butt, LineCap::Round, LineCap::Square];

    let pts = [
        -s * 0.25 + (t * 0.3).cos() * s * 0.5,
        (t * 0.3).sin() * s * 0.5,
        -s * 0.25,
        0.0,
        s * 0.25,
        0.0,
        s * 0.25 + (-t * 0.3).cos() * s * 0.5,
        (-t * 0.3).sin() * s * 0.5,
    ];

    for (i,join) in joins.iter().enumerate() {
        for (j, cap) in caps.iter().enumerate() {
            let fx = x + s * 0.5 + (i * 3 + j) as Scalar / 9.0 * w + pad;
            let fy = y - s * 0.5 + pad;

            canvas.set_line_cap(*cap);
            canvas.set_line_join(*join);

            canvas.set_line_width(s * 0.3);
            canvas.set_stroke_color(Color::rgba(0, 0, 0, 160));

            canvas.begin_path()
                  .move_to(fx + pts[0], fy + pts[1])
                  .line_to(fx + pts[2], fy + pts[3])
                  .line_to(fx + pts[4], fy + pts[5])
                  .line_to(fx + pts[6], fy + pts[7])
                  .stroke(dl);

            canvas.set_line_cap(LineCap::Butt);
            canvas.set_line_join(LineJoin::Miter);

            canvas.set_line_width(1.0);
            canvas.set_stroke_color(Color::rgba(0, 192, 255, 255));
            canvas.begin_path()
                  .move_to(fx + pts[0], fy + pts[1])
                  .line_to(fx + pts[2], fy + pts[3])
                  .line_to(fx + pts[4], fy + pts[5])
                  .line_to(fx + pts[6], fy + pts[7])
                  .stroke(dl);
        }
    }
}

#[profile]
fn draw_widths(canvas: &mut Canvas, dl: &mut DisplayList, x: Scalar, mut y: Scalar, length: Scalar) {
    canvas.set_stroke_color(Color::rgba(0, 0, 0, 255));

    for i in 0..20 {
        let width = (i as f32 + 0.5) * 0.1;
        canvas.set_line_width(width);
        canvas.begin_path()
              .move_to(x, y)
              .line_to(x + length, y + length * 0.3)
              .stroke(dl);
        y += 10.0;
    }
}

#[profile]
fn draw_graph(canvas: &mut Canvas, dl: &mut DisplayList, x: Scalar, y: Scalar, w: Scalar, h: Scalar, t: Scalar) {
    let samples = [
        (1.0 + (t * 1.2345 + (t * 0.33457).cos() * 0.44).sin()) * 0.5,
        (1.0 + (t * 0.68363 + (t * 1.3).cos() * 1.55).sin()) * 0.5,
        (1.0 + (t * 1.1642 + (t * 0.33457).cos() * 1.24).sin()) * 0.5,
        (1.0 + (t * 0.56345 + (t * 1.63).cos() * 0.14).sin()) * 0.5,
        (1.0 + (t * 1.6245 + (t * 0.254).cos() * 0.3).sin()) * 0.5,
        (1.0 + (t * 0.345 + (t * 0.03).cos() * 0.6).sin()) * 0.5,
    ];
    let dx = w / 5.0;

    let mut sx = [0.0; 6];
    let mut sy = [0.0; 6];
    for i in 0..6 {
        sx[i] = x + i as Scalar * dx;
        sy[i] = y + h * samples[i] * 0.8;
    }

    // Graph background
//    canvas.begin_path()
//        .move_to(sx[0], sy[0]);
//    for i in 1..6 {
//        canvas.bezier_to(sx[i - 1] + dx * 0.5, sy[i - 1], sx[i] - dx * 0.5, sy[i], sx[i], sy[i]);
//    }
//    canvas.line_to(x + w, y + h)
//        .line_to(x, y + h);
//        .fill(dl);

    // Graph line
    canvas.begin_path()
        .move_to(sx[0], sy[0] + 2.0);

    for i in 1..6 {
        canvas.bezier_to(sx[i - 1] + dx * 0.5, sy[i - 1] + 2.0, sx[i] - dx * 0.5, sy[i] + 2.0, sx[i], sy[i] + 2.0);
    }
    canvas.set_stroke_color(Color::rgba(0, 0, 0, 32));
    canvas.set_line_width(3.0);
    canvas.stroke(dl);

    canvas.begin_path()
        .move_to(sx[0], sy[0]);
    for i in 1..6 {
        canvas.bezier_to(sx[i - 1] + dx * 0.5, sy[i - 1], sx[i] - dx * 0.5, sy[i], sx[i], sy[i]);
    }
    canvas.set_stroke_color(Color::rgba(0, 160, 192, 255));
    canvas.set_line_width(3.0);
    canvas.stroke(dl);
//// Graph sample pos
//for (i = 0; i < 6; i++) {
//nvgBeginPath(vg);
//nvgRect(vg, sx[i]-10, sy[i]-10+2, 20,20);
//nvgFill(vg);
//}
//
//nvgBeginPath(vg);
//for (i = 0; i < 6; i++)
//nvgCircle(vg, sx[i], sy[i], 4.0f);
//nvgFillColor(vg, nvgRGBA(0,160,192,255));
//nvgFill(vg);
//nvgBeginPath(vg);
//for (i = 0; i < 6; i++)
//nvgCircle(vg, sx[i], sy[i], 2.0f);
//nvgFillColor(vg, nvgRGBA(220,220,220,255));
//nvgFill(vg);
//
//nvgStrokeWidth(vg, 1.0f);
}

#[profile]
fn render_frame_profile(
    renderer: &mut Renderer,
    dl: &mut DisplayList,
    face: &mut Face,
    last_frame: &Frame,
    pos: Vec2,
    font_pixel_size: u32,
) {
    let root_block = last_frame.root_block();
    let mut pos = pos;
    render_block(renderer, dl, face, font_pixel_size, &root_block, &mut pos);
}

fn render_block(renderer: &mut Renderer, dl: &mut DisplayList,
                face: &mut Face, font_pixel_size: u32,
                block: &BlockRef, pos: &mut Vec2) {
    let data = block.data();
    let delta_ms = data.delta() * 1000.0;

    let mut text_pos = *pos + Vec2::new(20.0 * block.level() as f32, 0.0);
    let text = if block.level() > 0 {
        format!("{:.2} ms {} ({}:{})", delta_ms, data.name(), data.file(), data.line())
    } else {
        format!("{:.2} ms", delta_ms)
    };
    render_text_line(renderer, dl, face, font_pixel_size, text, &mut text_pos).unwrap();
    pos.y = pos.y + font_pixel_size as f32;

    for child in block.children() {
        if child.index() < 10 && child.level() < 3 {
            render_block(renderer, dl, face, font_pixel_size, &child, pos);
        } else {
            let mut text_pos = *pos + Vec2::new(20.0 * child.level() as f32, 0.0);
            render_text_line(renderer, dl, face, font_pixel_size, "...", &mut text_pos).unwrap();
            pos.y = pos.y + font_pixel_size as f32;
            break;
        }
    }
}

#[profile]
fn render_text_line<S>(
    renderer: &mut Renderer,
    dl: &mut DisplayList,
    face: &mut Face,
    font_pixel_size: u32,
    text: S,
    pos: &mut Vec2,
) -> Result<(), Error>
where
    S: AsRef<str>
{
    for ch in text.as_ref().chars() {
        if let Some(glyph) = face.get_or_load_glyph(renderer, font_pixel_size, ch as usize) {
            if let Some(ref texture_region) = glyph.texture_region {
                dl.render_textured_quad(
                    Rect2::with_min_size(*pos + glyph.offset, texture_region.texture.size().as_vec2()),
                    texture_region.clone(),
                    Vec4::new(1.0, 0.0, 0.0, 1.0),
                );
            }

            *pos = *pos + glyph.advance;
        }
    }

    Ok(())
}


impl DisplayList {
    #[profile]
    pub fn render_textured_quad(&mut self, dst: Rect2, texture_region: TextureRegion, color: Vec4) {
        self.render_quad_raw(dst, color, Some(texture_region))
    }

    pub fn render_quad(&mut self, dst: Rect2, color: Vec4) {
        self.render_quad_raw(dst, color, None)
    }

    fn render_quad_raw(&mut self, dst: Rect2, color: Vec4, texture_region: Option<TextureRegion>) {
        let mut vertices = Vec::with_capacity(4);
        let mut indices = Vec::with_capacity(6);

        let viewport_size = self.viewport().size();
        let inv_viewport_size = 1.0 / viewport_size;
        let dst_rect_size = dst.size();
        let dst_rect_min = Vec2::new(dst.min.x, viewport_size.y - dst.min.y - dst_rect_size.y);
        let dst_rect_max = Vec2::new(dst.max.x, viewport_size.y - dst.min.y);
        let min = dst_rect_min.hadamard(inv_viewport_size) * 2.0 - Vec2::new(1.0, 1.0);
        let max = dst_rect_max.hadamard(inv_viewport_size) * 2.0 - Vec2::new(1.0, 1.0);

        let (tex_min, tex_max, texture) = if let Some(ref texture_region) = texture_region {
            let src = texture_region.region;
            let src_rect_size = src.size();
            let texture_size = texture_region.texture.size().as_vec2();
            let inv_texture_size = 1.0 / texture_size;
            let src_rect_min = Vec2::new(src.min.x, texture_size.y - src.min.y - src_rect_size.y);
            let src_rect_max = Vec2::new(src.max.x, texture_size.y - src.min.y);
            let tex_min = src_rect_min.hadamard(inv_texture_size);
            let tex_max = src_rect_max.hadamard(inv_texture_size);
            (tex_min, tex_max, Some(texture_region.texture.clone()))
        } else {
            (Vec2::zero(), Vec2::one(), None)
        };

        let vertex_index = vertices.len() as u32;

        // Bottom Left
        vertices.push(TriangleVertex {
            pos: Vec2::new(min.x, min.y),
            tex_coord: Vec2::new(tex_min.x, tex_min.y),
            color,
        });

        // Bottom Right
        vertices.push(TriangleVertex {
            pos: Vec2::new(max.x, min.y),
            tex_coord: Vec2::new(tex_max.x, tex_min.y),
            color,
        });

        // Top Right
        vertices.push(TriangleVertex {
            pos: Vec2::new(max.x, max.y),
            tex_coord: Vec2::new(tex_max.x, tex_max.y),
            color,
        });

        // Top Left
        vertices.push(TriangleVertex {
            pos: Vec2::new(min.x, max.y),
            tex_coord: Vec2::new(tex_min.x, tex_max.y),
            color,
        });

        indices.push(vertex_index);
        indices.push(vertex_index + 1);
        indices.push(vertex_index + 2);
        indices.push(vertex_index);
        indices.push(vertex_index + 2);
        indices.push(vertex_index + 3);

        self.render_triangles(vertices, indices, texture);
    }
}

impl StrokeRenderer for DisplayList {
    #[profile]
    fn render(&mut self, stroke: Stroke) {
        let nverts = stroke.path_iter().map(|path| path.verts().len()).sum();
        let nindices = (nverts - 2) * 3;
        let mut verts = Vec::with_capacity(nverts);
        let mut indices = Vec::with_capacity(nindices);

        let viewport_size = self.viewport().size();
        let inv_viewport_size = 1.0 / viewport_size;
        let transform_pos = |pos: Vec2| {
            let pos = Vec2::new(pos.x, viewport_size.y - pos.y);
            let pos = pos.hadamard(inv_viewport_size) * 2.0 - Vec2::new(1.0, 1.0);
            pos
        };

        let color = stroke.color();

        for path in stroke.path_iter() {
            let offset = verts.len();
            let ntriangles = path.verts().len();
            verts.extend(path.verts().iter().map(|vert| {
                TriangleVertex {
                    pos: transform_pos(Vec2::new(vert.x, vert.y)),
                    tex_coord: Vec2::new(vert.u, vert.v),
                    color: Vec4::new(color.r as Scalar / 255.0, color.g as Scalar / 255.0, color.b as Scalar / 255.0, color.a as Scalar / 255.0),
                }
            }));

            for index in 0..(ntriangles - 2) {
                indices.push((index + offset) as u32);
                indices.push((index + 1 + offset) as u32);
                indices.push((index + 2 + offset) as u32);
            }
        }

        self.render_triangles(verts, indices, None);
    }
}
