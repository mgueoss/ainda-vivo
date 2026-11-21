use super::{rgba_image, set_visible, smooth};
use crate::{
    app::{CollisionWorld, Comfort, LocalPlayer, MainCamera, Screen, Session},
    psychology::Effects,
    scene::presence,
};
use bevy::{ecs::system::SystemParam, light::NotShadowCaster, prelude::*};

const FIGURE_HEIGHT: f32 = 2.48;
const SHAPE: Color = Color::srgb(0.020, 0.029, 0.025);
const SHOULDER: Vec3 = Vec3::new(0.21, 1.51, 0.0);
const ELBOW_REST: Vec3 = Vec3::new(0.29, 1.16, -0.026);
const WRIST_REST: Vec3 = Vec3::new(0.31, 0.88, -0.035);
const ELBOW_UP: Vec3 = Vec3::new(0.51, 1.31, -0.065);
const WRIST_UP: Vec3 = Vec3::new(0.61, 1.60, -0.11);

#[derive(Component)]
struct Figure(Handle<StandardMaterial>);
#[derive(Component)]
struct Ghost;

#[derive(Component)]
enum Part {
    Head,
    UpperArm,
    Forearm,
    Hand,
    Light,
}

pub fn setup(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let solid = meshes.add(Sphere::new(1.0).mesh().uv(14, 10));
    let shape = materials.add(StandardMaterial {
        base_color: SHAPE.with_alpha(0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let backlight = images.add(glow());
    let light_material = materials.add(StandardMaterial {
        base_color: Color::WHITE.with_alpha(0.0),
        base_color_texture: Some(backlight),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let lamp_mesh = meshes.add(Rectangle::new(2.95, 2.65));
    commands
        .spawn((
            Name::new("A Presença — despedida"),
            Figure(shape.clone()),
            Transform::default(),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                Part::Light,
                Mesh3d(lamp_mesh),
                MeshMaterial3d(light_material),
                Transform::from_xyz(0.15, 1.05, 0.31)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                NotShadowCaster,
            ));
            let piece = |transform| {
                (
                    Mesh3d(solid.clone()),
                    MeshMaterial3d(shape.clone()),
                    transform,
                    NotShadowCaster,
                )
            };
            for (position, scale) in [
                (Vec3::new(0.0, 1.30, 0.0), Vec3::new(0.252, 0.40, 0.151)),
                (Vec3::new(0.0, 0.99, 0.008), Vec3::new(0.210, 0.27, 0.133)),
                (Vec3::new(0.0, 1.63, -0.015), Vec3::new(0.083, 0.14, 0.078)),
            ] {
                root.spawn(piece(
                    Transform::from_translation(position).with_scale(scale),
                ));
            }
            for side in [-1.0, 1.0] {
                root.spawn(piece(segment(
                    Vec3::new(side * 0.10, 0.98, 0.0),
                    Vec3::new(side * 0.126, 0.47, 0.035),
                    0.102,
                )));
                root.spawn(piece(segment(
                    Vec3::new(side * 0.126, 0.56, 0.035),
                    Vec3::new(side * 0.142, 0.065, 0.046),
                    0.080,
                )));
                root.spawn(piece(
                    Transform::from_xyz(side * 0.144, 0.06, -0.006)
                        .with_scale(Vec3::new(0.082, 0.055, 0.128)),
                ));
            }
            root.spawn(piece(segment(
                -SHOULDER.with_y(0.0) + Vec3::Y * 1.51,
                Vec3::new(-0.28, 1.13, 0.04),
                0.082,
            )));
            root.spawn(piece(segment(
                Vec3::new(-0.28, 1.16, 0.04),
                Vec3::new(-0.32, 0.84, 0.038),
                0.063,
            )));
            root.spawn(piece(
                Transform::from_xyz(-0.32, 0.81, 0.03).with_scale(Vec3::new(0.047, 0.085, 0.032)),
            ));
            root.spawn((Part::Head, piece(head(0.0))));
            root.spawn((Part::UpperArm, piece(segment(SHOULDER, ELBOW_REST, 0.082))));
            root.spawn((Part::Forearm, piece(segment(ELBOW_REST, WRIST_REST, 0.064))));
            root.spawn((Part::Hand, Transform::default(), Visibility::Inherited))
                .with_children(|hand| {
                    hand.spawn(piece(
                        Transform::from_xyz(0.0, 0.038, 0.0)
                            .with_scale(Vec3::new(0.052, 0.069, 0.023)),
                    ));
                    for (x, reach) in [
                        (-0.032, 0.122),
                        (-0.010, 0.143),
                        (0.013, 0.134),
                        (0.034, 0.109),
                    ] {
                        hand.spawn(piece(segment(
                            Vec3::new(x * 0.90, 0.059, 0.0),
                            Vec3::new(x, reach, 0.0),
                            0.012,
                        )));
                    }
                    hand.spawn(piece(segment(
                        Vec3::new(-0.031, 0.026, 0.0),
                        Vec3::new(-0.065, 0.072, -0.006),
                        0.015,
                    )));
                });
        });
    let mask = images.add(presence::mask());
    commands.spawn((
        Name::new("Reflexo incerto"),
        Ghost,
        Mesh3d(meshes.add(Rectangle::new(1.56, FIGURE_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.0),
            base_color_texture: Some(mask),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
    ));
}

fn glow() -> Image {
    const SIZE: u32 = 64;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for col in 0..SIZE {
            let uv = Vec2::new(col as f32, row as f32) / (SIZE - 1) as f32;
            let pos = (uv - Vec2::new(0.49, 0.45)) * Vec2::new(2.2, 2.04);
            let value = (1.0 - pos.length()).max(0.0).powf(1.72);
            let grain = ((row * 67 + col * 37) % 29) as f32 / 29.0 - 0.5;
            let alpha = (value * (165.0 + 17.0 * grain)).round() as u8;
            pixels.extend_from_slice(&[111, 95, 66, alpha]);
        }
    }
    rgba_image(SIZE, SIZE, pixels)
}

fn segment(from: Vec3, to: Vec3, radius: f32) -> Transform {
    let direction = to - from;
    Transform::from_translation((from + to) * 0.5)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, direction.normalize()))
        .with_scale(Vec3::new(radius, direction.length() * 0.59 + 0.025, radius))
}

fn head(age: f32) -> Transform {
    let lean = smooth((age - 3.45) / 0.92);
    Transform::from_xyz(0.055 * lean, 1.80 - 0.013 * lean, -0.025)
        .with_rotation(Quat::from_rotation_z(-0.29 * lean))
        .with_scale(Vec3::new(0.136, 0.181, 0.119))
}

fn wave(age: f32) -> (Vec3, Vec3, Quat) {
    let lift = smooth((age - 4.32) / 1.02);
    let release = smooth((age - 6.13) / 0.84);
    let raised = lift * (1.0 - release);
    let elbow = ELBOW_REST.lerp(ELBOW_UP, raised);
    let wrist = WRIST_REST.lerp(WRIST_UP, raised);
    let swing = (age * 5.0).sin() * 0.43 * smooth((age - 5.13) / 0.34) * (1.0 - release);
    let orientation = Quat::from_rotation_arc(Vec3::Y, (wrist - elbow).normalize())
        * Quat::from_rotation_z(swing);
    (elbow, wrist, orientation)
}

fn anchor(camera: &Transform, floor: f32, world: &CollisionWorld, gate: bool) -> Transform {
    let forward = camera.forward().with_y(0.0).normalize_or_zero();
    let camera_feet = camera.translation.with_y(floor);
    for distance in [3.64_f32, 3.0, 2.45, 1.82, 1.19, 0.66, 0.38] {
        let scale = (distance / 2.80).min(1.0);
        for yaw in [0.0_f32, -0.23, 0.23] {
            let direction = Quat::from_rotation_y(yaw) * forward;
            let origin = camera_feet + direction * distance;
            let right = direction.cross(Vec3::Y);
            let clear = [(0.0, 0.36), (0.0, 1.84), (0.59, 1.65), (-0.28, 1.39)]
                .into_iter()
                .all(|(x, y)| {
                    let point = origin + (Vec3::Y * y + right * x + direction * 0.25) * scale;
                    !world.occludes(camera.translation, point, gate)
                });
            if clear {
                return Transform::from_translation(origin)
                    .looking_to(-direction, Vec3::Y)
                    .with_scale(Vec3::splat(scale));
            }
        }
    }
    // In a cramped corner the apparition is presented as a very close frame.
    // This miniature has no collider and fits in the local body's free radius.
    Transform::from_translation(camera.translation + forward * 0.21 - Vec3::Y * 0.09)
        .looking_to(-forward, Vec3::Y)
        .with_scale(Vec3::splat(0.10))
}

fn set_opacity(
    materials: &mut Assets<StandardMaterial>,
    material: &MeshMaterial3d<StandardMaterial>,
    alpha: f32,
) {
    if let Some(surface) = materials.get_mut(material) {
        let color = Color::WHITE.with_alpha(alpha);
        if surface.base_color != color {
            surface.base_color = color;
        }
    }
}

#[derive(Default)]
pub struct FigureState {
    anchor: Option<Transform>,
    phantom: Option<(Vec3, f32)>,
}

#[derive(SystemParam)]
#[allow(
    clippy::type_complexity,
    reason = "Disjoint Bevy actor, limb and phantom queries."
)]
pub struct FigureViews<'w, 's> {
    camera: Query<'w, 's, &'static Transform, With<MainCamera>>,
    figure: Query<
        'w,
        's,
        (
            &'static Figure,
            &'static mut Transform,
            &'static mut Visibility,
        ),
        (
            With<Figure>,
            Without<MainCamera>,
            Without<Part>,
            Without<Ghost>,
        ),
    >,
    parts: Query<
        'w,
        's,
        (
            &'static Part,
            &'static mut Transform,
            Option<&'static MeshMaterial3d<StandardMaterial>>,
        ),
        (With<Part>, Without<MainCamera>, Without<Ghost>),
    >,
    ghost: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static mut Visibility,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
        (With<Ghost>, Without<MainCamera>),
    >,
}

pub fn update(
    (session, player, comfort): (Res<Session>, Res<LocalPlayer>, Res<Comfort>),
    effects: Res<Effects>,
    world: Res<CollisionWorld>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut views: FigureViews,
    mut state: Local<FigureState>,
) {
    let active = matches!(session.screen, Screen::Playing | Screen::Paused);
    let farewell = active.then_some(effects.farewell).flatten();
    let (
        Ok(camera),
        Ok((shape, mut pose, mut figure_visibility)),
        Ok((mut ghost, mut ghost_visibility, ghost_material)),
    ) = (
        views.camera.single(),
        views.figure.single_mut(),
        views.ghost.single_mut(),
    )
    else {
        return;
    };
    if let Some(age) = farewell {
        let fixed = *state.anchor.get_or_insert_with(|| {
            anchor(
                camera,
                player.body.feet.y,
                &world,
                session.progress().exit_unlocked(),
            )
        });
        let reveal = smooth((age - 1.50) / 0.85);
        set_visible(&mut figure_visibility, reveal > 0.002);
        set_visible(&mut ghost_visibility, false);
        state.phantom = None;
        if reveal <= 0.002 {
            return;
        }
        if *pose != fixed {
            *pose = fixed;
        }
        if let Some(material) = materials.get_mut(&shape.0) {
            let tint = SHAPE.with_alpha(reveal);
            if material.base_color != tint {
                material.base_color = tint;
            }
        }
        let (elbow, wrist, orientation) = wave(age);
        for (part, mut transform, material) in &mut views.parts {
            match part {
                Part::Head => *transform = head(age),
                Part::UpperArm => *transform = segment(SHOULDER, elbow, 0.082),
                Part::Forearm => *transform = segment(elbow, wrist, 0.062),
                Part::Hand => {
                    *transform = Transform::from_translation(wrist).with_rotation(orientation)
                }
                Part::Light => {
                    if let Some(material) = material {
                        set_opacity(&mut materials, material, 0.88 * smooth((age - 1.40) / 1.63));
                    }
                }
            }
        }
        return;
    }
    state.anchor = None;
    set_visible(&mut figure_visibility, false);
    let phantom = active.then_some(effects.hallucination).flatten();
    let now = session.elapsed;
    if let Some(at) = phantom {
        if state.phantom.is_none_or(|(old, _)| old != at) {
            state.phantom = Some((at, now));
        }
    } else {
        state.phantom = None;
    }
    if let Some((at, since)) = state.phantom {
        let position = at + Vec3::Y * FIGURE_HEIGHT * 0.5;
        let toward_observer = camera.translation.with_y(position.y) - position;
        let age = now - since;
        let alpha =
            0.62 * comfort.psycho_av * smooth(age / 0.40) * (1.0 - smooth((age - 0.84) / 0.65));
        set_visible(&mut ghost_visibility, alpha > 0.003);
        if alpha > 0.003 && toward_observer.length_squared() > 0.01 {
            *ghost = Transform::from_translation(position).looking_to(-toward_observer, Vec3::Y);
            set_opacity(&mut materials, ghost_material, alpha);
        }
    } else {
        set_visible(&mut ghost_visibility, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn farewell_motion_is_deliberate_and_returns_to_rest() {
        let (before, down, _) = wave(3.6);
        assert_eq!(before, ELBOW_REST);
        assert_eq!(down, WRIST_REST);
        let (_, up, first) = wave(5.5);
        let (_, _, second) = wave(5.9);
        assert!(up.y - down.y > 0.6);
        assert!(first.angle_between(second) > 0.12);
        assert_eq!(wave(7.1).1, WRIST_REST);
        assert!(head(4.5).rotation.angle_between(head(2.7).rotation) > 0.23);
    }
}
