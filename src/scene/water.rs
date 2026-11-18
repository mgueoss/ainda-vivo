use super::surfaces;
use crate::atmosphere::Atmosphere;
use bevy::{light::NotShadowCaster, prelude::*};
use std::f32::consts::FRAC_PI_2;

#[derive(Component)]
pub struct Ripple(f32);

const REFLECTION: Color = Color::srgb(0.38, 0.53, 0.49);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    const SIZE: u32 = 80;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let local = (Vec2::new(x as f32, y as f32) + 0.5) * (2.0 / SIZE as f32) - 1.0;
            let ring = (1.0 - (local.length() - 0.81).abs() / 0.075).max(0.0);
            pixels.extend_from_slice(&[255, 255, 255, (ring * ring * 255.0) as u8]);
        }
    }
    let image = images.add(surfaces::texture(SIZE, SIZE, pixels, false));
    let plane = meshes.add(Rectangle::new(2.0, 2.0));
    for delay in [0.96, 1.37] {
        commands.spawn((
            Name::new("Onda breve na bacia"),
            Ripple(delay),
            Mesh3d(plane.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: REFLECTION.with_alpha(0.0),
                base_color_texture: Some(image.clone()),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            Visibility::Hidden,
            NotShadowCaster,
        ));
    }
}

pub fn update(
    scene: Res<Atmosphere>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ripples: Query<(
        &Ripple,
        &MeshMaterial3d<StandardMaterial>,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    for (ripple, material, mut pose, mut visible) in &mut ripples {
        if let Some(trace) = scene.episode.trace
            && (0.0..2.3).contains(&(trace.age - ripple.0))
        {
            let age = (trace.age - ripple.0) / 2.3;
            pose.translation = trace.water_position() + Vec3::Z * (age * -0.48);
            pose.scale = Vec3::splat(0.16 + age * 1.48);
            if let Some(material) = materials.get_mut(&material.0) {
                material.base_color =
                    REFLECTION.with_alpha(0.32 * (age * 8.0).min(1.0) * (1.0 - age));
            }
            *visible = Visibility::Visible;
        } else if *visible != Visibility::Hidden {
            *visible = Visibility::Hidden;
        }
    }
}
