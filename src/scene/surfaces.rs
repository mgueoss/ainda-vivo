use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    mesh::VertexAttributeValues,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Clone, Copy)]
pub enum Surface {
    Paint,
    Tile,
    Ceiling,
    Concrete,
    Oxide,
    Water,
}

pub fn material(
    surface: Surface,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let size = 256;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let uv = Vec2::new(x as f32, y as f32) / size as f32;
            let cloud = noise(uv, IVec2::new(7, 7), 3);
            let grain = noise(uv, IVec2::new(49, 53), 28);
            let grit = hash(x as i32, y as i32, 14) - 0.5;
            let mottled = cloud * 0.15 + grain * 0.12 + grit * 0.065;
            let rgb = match surface {
                Surface::Paint => {
                    let streak = noise(uv, IVec2::new(34, 3), 91);
                    let damp = ((streak * 0.67 + cloud * 0.33 - 0.49) * 3.8).clamp(0.0, 1.0);
                    let chips = (grain > 0.80 && cloud < 0.42) as u8 as f32 * 0.12;
                    let paint = Vec3::new(0.45, 0.49, 0.45) + Vec3::splat(mottled - chips);
                    let runoff = noise(uv, IVec2::new(57, 2), 61).powi(6);
                    paint.lerp(Vec3::new(0.20, 0.24, 0.21), damp * 0.44 + runoff * 0.23)
                }
                Surface::Tile => {
                    let grid = (uv * 4.0).fract();
                    let joint = grid.x.min(1.0 - grid.x).min(grid.y).min(1.0 - grid.y);
                    let tile_tone = hash(x as i32 / 64, y as i32 / 64, 112) * 0.075;
                    let stone = Vec3::new(0.25, 0.26, 0.246) + Vec3::splat(mottled + tile_tone);
                    let grout = Vec3::new(0.105, 0.113, 0.107) + Vec3::splat(grit * 0.05);
                    grout.lerp(stone, (joint / 0.035).clamp(0.0, 1.0))
                }
                Surface::Ceiling => Vec3::new(0.255, 0.27, 0.26) + Vec3::splat(mottled * 0.72),
                Surface::Concrete => {
                    let stain = (cloud * 0.7 + grain * 0.3).powi(3);
                    let wear = noise(uv, IVec2::new(3, 11), 40).powi(4);
                    Vec3::new(0.29, 0.32, 0.306)
                        + Vec3::splat(mottled * 0.66 - stain * 0.20 + wear * 0.068)
                }
                Surface::Oxide => {
                    let wear = noise(uv, IVec2::new(17, 6), 110);
                    Vec3::new(0.26, 0.335, 0.315).lerp(
                        Vec3::new(0.36, 0.21, 0.133),
                        ((wear * 0.5 + cloud * 0.5 - 0.54) * 3.1).clamp(0.0, 0.83),
                    ) + Vec3::splat(mottled * 0.45)
                }
                Surface::Water => {
                    let fold = (std::f32::consts::TAU * (uv.y * 8.0 + cloud * 0.75))
                        .sin()
                        .powi(8);
                    Vec3::new(0.052, 0.177, 0.156)
                        + Vec3::new(0.038, 0.074, 0.066) * (fold * 0.55 + grain * 0.48)
                }
            };
            let rgb = rgb.clamp(Vec3::ZERO, Vec3::ONE) * 255.0;
            data.extend_from_slice(&[rgb.x as u8, rgb.y as u8, rgb.z as u8, 255]);
        }
    }
    let image = texture(size, size, data, true);
    materials.add(StandardMaterial {
        base_color_texture: Some(images.add(image)),
        perceptual_roughness: match surface {
            Surface::Tile => 0.79,
            Surface::Oxide => 0.65,
            Surface::Water => 0.36,
            _ => 0.97,
        },
        metallic: if matches!(surface, Surface::Oxide) {
            0.29
        } else {
            0.0
        },
        reflectance: 0.31,
        ..default()
    })
}

/// Keep tile and paint detail at a consistent size across different wall lengths.
pub fn world_uv_cube(center: Vec3, size: Vec3, metres_per_repeat: f32) -> Mesh {
    let mut mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0));
    if let (
        Some(VertexAttributeValues::Float32x3(positions)),
        Some(VertexAttributeValues::Float32x3(normals)),
    ) = (
        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
    ) {
        let uv: Vec<[f32; 2]> = positions
            .iter()
            .zip(normals)
            .map(|(&p, n)| {
                let p = center + Vec3::from_array(p) * size;
                let axes = if n[0].abs() > 0.5 {
                    [p.z, -p.y]
                } else if n[1].abs() > 0.5 {
                    [p.x, p.z]
                } else {
                    [p.x, -p.y]
                };
                axes.map(|component| component / metres_per_repeat)
            })
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    }
    mesh
}

pub fn sign(word: &str, fill: [u8; 3], ink: [u8; 3]) -> Image {
    const HEIGHT: u32 = 48;
    let glyphs: Vec<_> = word.chars().map(glyph).collect();
    let scale = 4;
    let word_width = (glyphs.len() as u32 * 6).saturating_sub(1) * scale;
    let width = (word_width + 26).max(192);
    let left = (width - word_width) / 2;
    let top = (HEIGHT - 7 * scale) / 2;
    let mut data = Vec::with_capacity((width * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..width {
            let trim = (3..6).contains(&x)
                || (3..6).contains(&(width - 1 - x))
                || (3..5).contains(&y)
                || (3..5).contains(&(HEIGHT - 1 - y));
            let mut letter = false;
            if x >= left && (top..top + 7 * scale).contains(&y) {
                let col = (x - left) / scale;
                let row = ((y - top) / scale) as usize;
                letter = col % 6 < 5
                    && glyphs
                        .get((col / 6) as usize)
                        .is_some_and(|bits| bits[row] & (1 << (4 - col % 6)) != 0);
            }
            let mut pixel = if letter || trim { ink } else { fill };
            let wear = hash(x as i32, y as i32, 44);
            if wear < 0.08 {
                pixel = pixel.map(|value| value.saturating_sub(24));
            }
            data.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
        }
    }
    texture(width, HEIGHT, data, false)
}

fn glyph(letter: char) -> [u8; 7] {
    match letter {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'Á' | 'Â' | 'Ã' => [10, 4, 14, 17, 31, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'Ç' => [14, 17, 16, 16, 17, 14, 4],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' | 'É' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'Í' => [2, 4, 14, 4, 4, 4, 14],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' | 'Ó' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '↑' => [4, 14, 21, 4, 4, 4, 4],
        '←' => [0, 4, 8, 31, 8, 4, 0],
        '→' => [0, 4, 2, 31, 2, 4, 0],
        _ => [0; 7],
    }
}

pub(super) fn texture(width: u32, height: u32, data: Vec<u8>, tiled: bool) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    if tiled {
        let data = image
            .data
            .as_mut()
            .expect("texture pixels were initialized");
        let mut offset = 0;
        let mut size = width as usize;
        while size > 1 {
            let half = size / 2;
            for y in 0..half {
                for x in 0..half {
                    let corner = offset + (y * size * 2 + x * 2) * 4;
                    for channel in 0..4 {
                        let value = [corner, corner + 4, corner + size * 4, corner + size * 4 + 4]
                            .iter()
                            .map(|&pixel| data[pixel + channel] as u16)
                            .sum::<u16>();
                        data.push((value / 4) as u8);
                    }
                }
            }
            offset += size * size * 4;
            size = half;
            image.texture_descriptor.mip_level_count += 1;
        }
    }
    let wrap = if tiled {
        ImageAddressMode::Repeat
    } else {
        ImageAddressMode::ClampToEdge
    };
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: wrap,
        address_mode_v: wrap,
        anisotropy_clamp: if tiled { 4 } else { 1 },
        ..ImageSamplerDescriptor::linear()
    });
    image
}

pub(super) fn noise(uv: Vec2, cells: IVec2, seed: u32) -> f32 {
    let uv = uv * cells.as_vec2();
    let origin = uv.floor().as_ivec2();
    let f = uv.fract();
    let f = f * f * (Vec2::splat(3.0) - 2.0 * f);
    let corner = |x: i32, y: i32| {
        hash(
            (origin.x + x).rem_euclid(cells.x),
            (origin.y + y).rem_euclid(cells.y),
            seed,
        )
    };
    let low = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * f.x;
    let high = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * f.x;
    low + (high - low) * f.y
}

fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut number =
        (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ seed;
    number ^= number >> 16;
    number = number.wrapping_mul(0x7FEB_352D);
    number ^= number >> 15;
    (number >> 8) as f32 / 16_777_215.0
}
