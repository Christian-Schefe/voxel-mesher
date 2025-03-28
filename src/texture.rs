use crate::geometry::Quad;
use bevy_math::Vec2;
use image::{ImageBuffer, RgbaImage};

pub fn sort_quads(quads: &mut Vec<Quad>) {
    quads.sort_by(|a, b| {
        if a.tex_size.1 != b.tex_size.1 {
            b.tex_size.1.cmp(&a.tex_size.1)
        } else {
            b.tex_size.0.cmp(&a.tex_size.0)
        }
    });
}

pub fn pack_quad_texture(quads: &Vec<Quad>) -> (Vec<TexQuad>, usize) {
    let mut size = 1;
    let mut tex_quads = try_pack_quad_texture(quads, size);
    while tex_quads.is_none() {
        size *= 2;
        tex_quads = try_pack_quad_texture(quads, size);
    }
    (tex_quads.unwrap(), size)
}

pub fn apply_uv_to_quads(quads: &mut Vec<Quad>, tex_quads: &Vec<TexQuad>, size: usize) {
    for (quad, tex_quad) in quads.iter_mut().zip(tex_quads.iter()) {
        let min_uv = (
            tex_quad.x as f32 / size as f32,
            (size - tex_quad.y - tex_quad.height) as f32 / size as f32,
        );
        let uv_size = (
            (tex_quad.width) as f32 / size as f32,
            (tex_quad.height) as f32 / size as f32,
        );
        let uvs = quad
            .uvs
            .map(|uv| Vec2::new(min_uv.0 + uv.x * uv_size.0, min_uv.1 + uv.y * uv_size.1));
        quad.uvs = uvs;
    }
}

pub fn create_texture_file(tex_quads: &Vec<TexQuad>, size: usize) -> RgbaImage {
    let mut img_buf = ImageBuffer::new(size as u32, size as u32);
    for (i, tex_quad) in tex_quads.iter().enumerate() {
        let color = [
            (tex_quad.x * 255 / size) as u8,
            (tex_quad.y * 255 / size) as u8,
            255 - (i * 255 / tex_quads.len()) as u8,
            255,
        ];
        for y in 0..tex_quad.height {
            for x in 0..tex_quad.width {
                img_buf.put_pixel(
                    (tex_quad.x + x) as u32,
                    (tex_quad.y + y) as u32,
                    image::Rgba(color),
                );
            }
        }
    }
    img_buf
}

fn try_pack_quad_texture(quads: &Vec<Quad>, size: usize) -> Option<Vec<TexQuad>> {
    let mut tex_quads = Vec::new();
    let mut texture = vec![0; size * size];
    for quad in quads {
        let mut x = 0;
        let mut y = 0;
        let mut found = false;
        while y < size {
            while x < size {
                if can_place_quad(&texture, size, x, y, quad) {
                    place_quad(&mut texture, size, x, y, quad);
                    tex_quads.push(TexQuad {
                        x,
                        y,
                        width: quad.tex_size.0,
                        height: quad.tex_size.1,
                    });
                    found = true;
                    break;
                }
                x += 1;
            }
            if found {
                break;
            }
            x = 0;
            y += 1;
        }
        if !found {
            return None;
        }
    }
    Some(tex_quads)
}

fn can_place_quad(texture: &Vec<u8>, size: usize, x: usize, y: usize, quad: &Quad) -> bool {
    if x + quad.tex_size.0 > size || y + quad.tex_size.1 > size {
        return false;
    }
    for j in 0..quad.tex_size.1 {
        for i in 0..quad.tex_size.0 {
            if texture[(y + j) * size + x + i] != 0 {
                return false;
            }
        }
    }
    true
}

fn place_quad(texture: &mut Vec<u8>, size: usize, x: usize, y: usize, quad: &Quad) {
    for j in 0..quad.tex_size.1 {
        for i in 0..quad.tex_size.0 {
            texture[(y + j) * size + x + i] = 1;
        }
    }
}

#[derive(Debug)]
pub struct TexQuad {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}
