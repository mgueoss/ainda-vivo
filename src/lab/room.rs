use super::{Lab, settings::Config};
use crate::{
    app::Flashlight,
    preferences::Preferences,
    vhs::{VhsImage, VhsMode},
};
use ainda_vivo_core::{
    collision::Aabb,
    movement::{PlayerBody, PlayerCommand},
};
use bevy::{
    asset::RenderAssetUsages,
    camera::{
        Exposure,
        visibility::{NoFrustumCulling, RenderLayers},
    },
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfigBuilder, NotShadowCaster},
    mesh::PrimitiveTopology,
    prelude::*,
};

pub(super) const LAYER: usize = 6;
pub(super) const FLOOR: f32 = 0.025;

#[derive(Component)]
pub(super) struct LabCamera;
#[derive(Component)]
pub(super) struct KeyLight;
#[derive(Component)]
pub(super) struct LabTorch;
#[derive(Component)]
pub(super) struct Grid;
#[derive(Component)]
pub(super) struct Seat;

#[derive(Clone, Copy, Component, PartialEq, Eq)]
pub(super) enum Guide {
    Bones,
    Bounds,
    Wire,
    Planted,
    Lifted,
    Track,
}

pub(super) struct Room {
    pub camera: Entity,
    pub seat: Entity,
    pub solids: Vec<Aabb>,
}

pub(super) fn spawn(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Room {
    let layer = RenderLayers::layer(LAYER);
    let camera = commands
        .spawn((
            Name::new("Dev Lab · câmera"),
            LabCamera,
            ChildOf(root),
            Camera3d::default(),
            Camera {
                order: 1,
                clear_color: Color::srgb(0.015, 0.021, 0.022).into(),
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov: 51.0_f32.to_radians(),
                near: 0.025,
                far: 65.0,
                ..default()
            }),
            AmbientLight {
                color: Color::srgb(0.87, 0.91, 1.0),
                brightness: 360.0,
                ..default()
            },
            Exposure { ev100: 10.0 },
            Tonemapping::TonyMcMapface,
            Msaa::Sample4,
            Transform::from_xyz(0.0, 1.53, 4.2),
            layer.clone(),
        ))
        .id();
    commands.spawn((
        KeyLight,
        ChildOf(root),
        layer.clone(),
        Transform::default(),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 32.0,
            ..default()
        }
        .build(),
    ));
    commands.spawn((
        LabTorch,
        ChildOf(root),
        layer.clone(),
        Visibility::Hidden,
        Transform::default(),
        SpotLight {
            color: Color::srgb(1.0, 0.956, 0.858),
            intensity: 345_000.0,
            range: 15.1,
            inner_angle: 0.22,
            outer_angle: 0.51,
            shadow_depth_bias: 0.025,
            shadows_enabled: true,
            ..default()
        },
    ));
    let palette = [
        material(materials, Color::srgb(0.31, 0.335, 0.337)),
        material(materials, Color::srgb(0.45, 0.46, 0.45)),
        material(materials, Color::srgb(0.225, 0.251, 0.256)),
        material(materials, Color::srgb(0.13, 0.154, 0.154)),
        material(materials, Color::srgb(0.35, 0.27, 0.15)),
    ];
    let cube = meshes.add(Cuboid::default());
    let mut solids = Vec::with_capacity(16);
    for (label, pos, dimensions, color) in [
        ("Piso da sala", [0., -0.10, -5.5], [17., 0.20, 24.], 0),
        ("Parede oeste", [-8.5, 2.05, -5.5], [0.25, 4.10, 24.], 1),
        ("Parede leste", [8.5, 2.05, -5.5], [0.25, 4.10, 24.], 1),
        ("Parede do retorno", [0., 2.05, 6.5], [17., 4.10, 0.24], 1),
        ("Parede do fundo", [0., 2.05, -17.5], [17., 4.10, 0.24], 1),
        ("Fundo de inspeção", [0., 1.7, -4.1], [5.4, 3.4, 0.20], 2),
        (
            "Coluna de espiada",
            [-2.45, 1.44, -1.72],
            [0.47, 2.88, 0.43],
            1,
        ),
        (
            "Corredor · esquerda",
            [3.2, 1.42, -10.4],
            [0.34, 2.84, 9.4],
            2,
        ),
        (
            "Corredor · direita",
            [6.0, 1.42, -10.4],
            [0.34, 2.84, 9.4],
            2,
        ),
        (
            "Corredor · travessa",
            [4.6, 2.79, -11.35],
            [3.10, 0.28, 7.6],
            3,
        ),
        (
            "Alcova · anteparo",
            [4.25, 1.44, -14.6],
            [1.65, 2.88, 0.26],
            3,
        ),
        ("Obstáculo baixo", [-4.2, 0.27, -6.3], [0.70, 0.54, 1.20], 4),
        ("Obstáculo alto", [-4.48, 0.91, -8.6], [0.82, 1.82, 0.80], 2),
    ] {
        let pos = Vec3::from(pos);
        let size = Vec3::from(dimensions);
        solids.push(Aabb::from_center_size(pos, size));
        commands.spawn((
            Name::new(label),
            ChildOf(root),
            layer.clone(),
            Mesh3d(cube.clone()),
            MeshMaterial3d(palette[color].clone()),
            Transform::from_translation(pos).with_scale(size),
        ));
    }
    commands.spawn((
        Name::new("Platô de inspeção"),
        ChildOf(root),
        layer.clone(),
        Mesh3d(meshes.add(Cylinder::new(2.08, FLOOR))),
        MeshMaterial3d(palette[2].clone()),
        Transform::from_xyz(0.0, FLOOR * 0.5, 0.0),
    ));
    let mut lines = Vec::new();
    for unit in -8..=8 {
        let x = unit as f32;
        lines.extend([[x, 0.002, -17.4], [x, 0.002, 6.4]]);
    }
    for unit in -17..=6 {
        let z = unit as f32;
        lines.extend([[-8.4, 0.002, z], [8.4, 0.002, z]]);
    }
    detail_mesh(
        commands,
        root,
        Grid,
        meshes.add(line_mesh(lines)),
        material(materials, Color::srgb(0.40, 0.43, 0.43)),
    );

    let mut ruler = Vec::new();
    let base = Vec3::new(-2.14, 0.006, 1.0);
    for i in 0..=4 {
        let mark = base + Vec3::Z * (i as f32 - 2.0);
        ruler.extend([mark.to_array(), (mark - Vec3::X * 0.19).to_array()]);
        let xy = [
            (0.0, 0.0),
            (0.075, 0.0),
            (0.075, 0.085),
            (0.075, 0.17),
            (0.0, 0.17),
            (0.0, 0.085),
        ];
        let segments = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0), (5, 2)];
        let active = [0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110][i];
        for (segment, (a, b)) in segments.into_iter().enumerate() {
            if active & (1 << segment) != 0 {
                for corner in [a, b] {
                    let (x, z) = xy[corner];
                    ruler.push((mark + Vec3::new(-0.31 + x, 0.0, 0.08 + z)).to_array());
                }
            }
        }
    }
    ruler.extend([
        (base - Vec3::Z * 2.0).to_array(),
        (base + Vec3::Z * 2.0).to_array(),
    ]);
    detail_mesh(
        commands,
        root,
        Grid,
        meshes.add(line_mesh(ruler)),
        unlit(materials, Color::srgb(0.66, 0.57, 0.41)),
    );
    let mut lane = Vec::new();
    for index in 0..54 {
        let distance = index as f32 * 0.25;
        let (point, yaw) = super::motion::track(distance);
        let across = Quat::from_rotation_y(yaw) * Vec3::X;
        let point = point.with_y(0.008);
        if index % 4 == 0 {
            lane.extend([
                (point - across * 0.58).to_array(),
                (point + across * 0.58).to_array(),
            ]);
        }
        if index > 0 {
            let (previous, heading) = super::motion::track(distance - 0.25);
            let normal = Quat::from_rotation_y(heading) * Vec3::X;
            for side in [-0.58, 0.58] {
                lane.extend([
                    (previous.with_y(0.008) + normal * side).to_array(),
                    (point + across * side).to_array(),
                ]);
            }
        }
    }
    detail_mesh(
        commands,
        root,
        Guide::Track,
        meshes.add(line_mesh(lane)),
        unlit(materials, Color::srgb(0.68, 0.55, 0.24)),
    );
    for (guide, tint) in [
        (Guide::Bones, Color::srgb(1.0, 0.54, 0.18)),
        (Guide::Bounds, Color::srgb(0.31, 0.92, 0.79)),
        (Guide::Wire, Color::srgb(0.10, 0.52, 0.48)),
        (Guide::Planted, Color::srgb(0.37, 1.0, 0.72)),
        (Guide::Lifted, Color::srgb(1.0, 0.57, 0.23)),
    ] {
        detail_mesh(
            commands,
            root,
            guide,
            meshes.add(line_mesh(vec![[0.; 3]; 2])),
            unlit(materials, tint),
        );
    }
    let seat = commands
        .spawn((
            Seat,
            ChildOf(root),
            Name::new("Banco para testar apoio sentado"),
            Transform::default(),
            Visibility::Hidden,
        ))
        .with_children(|chair| {
            for (position, size) in [
                ([0.0, 0.464, 0.0], [0.53, 0.060, 0.47]),
                ([-0.209, 0.218, -0.160], [0.037, 0.437, 0.037]),
                ([-0.209, 0.218, 0.160], [0.037, 0.437, 0.037]),
                ([0.209, 0.218, -0.160], [0.037, 0.437, 0.037]),
                ([0.209, 0.218, 0.160], [0.037, 0.437, 0.037]),
            ] {
                chair.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(palette[3].clone()),
                    layer.clone(),
                    Transform::from_translation(Vec3::from(position)).with_scale(Vec3::from(size)),
                ));
            }
        })
        .id();
    Room {
        camera,
        seat,
        solids,
    }
}

fn detail_mesh(
    commands: &mut Commands,
    root: Entity,
    detail: impl Component,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    commands.spawn((
        detail,
        ChildOf(root),
        RenderLayers::layer(LAYER),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::default(),
        NoFrustumCulling,
        NotShadowCaster,
    ));
}

pub(super) fn line_mesh(lines: Vec<[f32; 3]>) -> Mesh {
    Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, lines)
}

pub(super) fn material(
    assets: &mut Assets<StandardMaterial>,
    base_color: Color,
) -> Handle<StandardMaterial> {
    assets.add(StandardMaterial {
        base_color,
        perceptual_roughness: 0.88,
        ..default()
    })
}

fn unlit(assets: &mut Assets<StandardMaterial>, base_color: Color) -> Handle<StandardMaterial> {
    assets.add(StandardMaterial {
        base_color,
        unlit: true,
        ..default()
    })
}

pub(super) fn walk(
    mut lab: ResMut<Lab>,
    config: Res<Config>,
    time: Res<Time<Fixed>>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<Preferences>,
) {
    let Some(active) = lab.active.as_mut() else {
        return;
    };
    if config.lens.free {
        return;
    }
    let mut command = if active.captured && !super::ctrl(&keys) {
        crate::desktop::movement_input(&keys, &bindings)
    } else {
        PlayerCommand {
            crouch: active.captured && super::ctrl(&keys) && !super::chord_down(&keys),
            ..default()
        }
    };
    command.yaw = config.lens.yaw;
    command.jump = std::mem::take(&mut active.jump);
    let gate = active.observer.solid();
    if let Some(gate) = gate {
        active.room.solids.push(gate);
    }
    active
        .walker
        .step(command, time.delta_secs(), &active.room.solids);
    if gate.is_some() {
        active.room.solids.pop();
    }
    if active.walker.feet.y < -3.0 {
        active.walker = walker(Vec3::new(0.0, 0.002, 4.2));
    }
}

pub(super) fn walker(feet: Vec3) -> PlayerBody {
    PlayerBody {
        feet,
        previous_feet: feet,
        ..default()
    }
}

type LabCameraView<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static mut Projection,
        &'static mut AmbientLight,
        &'static mut Exposure,
        Option<&'static mut VhsImage>,
        Option<&'static DistanceFog>,
    ),
    (
        With<LabCamera>,
        Without<KeyLight>,
        Without<LabTorch>,
        Without<Flashlight>,
    ),
>;

type KeyView<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut DirectionalLight),
    (With<KeyLight>, Without<LabTorch>),
>;
type TorchView<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut SpotLight,
    ),
    (With<LabTorch>, Without<KeyLight>),
>;

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy scene queries remain independently scoped."
)]
pub(super) fn present(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<Config>,
    mut lab: ResMut<Lab>,
    mut camera: LabCameraView,
    mut key: KeyView,
    mut torch: TorchView,
    mut grid: Query<&mut Visibility, (With<Grid>, Without<LabTorch>)>,
) {
    let Some(active) = lab.active.as_mut() else {
        return;
    };
    let Ok((entity, mut position, mut projection, mut ambient, mut exposure, vhs, fog)) =
        camera.single_mut()
    else {
        return;
    };
    if !config.lens.free {
        let wanted = if active.walker.crouched { 0.93 } else { 1.62 };
        active.eye += (wanted - active.eye) * (1.0 - (-14.0 * time.delta_secs()).exp());
    }
    *position = active.camera_transform(&config);
    if let Projection::Perspective(p) = &mut *projection {
        p.fov = config.lens.fov.to_radians();
    }
    let light = &config.lighting;
    ambient.brightness = light.fill;
    exposure.ev100 = light.exposure;
    let mode = [VhsMode::Off, VhsMode::Soft, VhsMode::Strong][light.vhs as usize];
    match (mode, vhs) {
        (VhsMode::Off, Some(_)) => {
            commands.entity(entity).remove::<VhsImage>();
        }
        (VhsMode::Off, None) => {}
        (_, Some(mut current)) => *current = VhsImage::lab(mode, time.elapsed_secs()),
        (_, None) => {
            commands
                .entity(entity)
                .insert(VhsImage::lab(mode, time.elapsed_secs()));
        }
    }
    match (light.fog, fog.is_some()) {
        (true, false) => {
            commands.entity(entity).insert(DistanceFog {
                color: crate::scene::DISTANCE_DARK,
                falloff: FogFalloff::Linear {
                    start: crate::scene::DISTANCE_START,
                    end: crate::scene::DISTANCE_END,
                },
                ..default()
            });
        }
        (false, true) => {
            commands.entity(entity).remove::<DistanceFog>();
        }
        _ => {}
    }
    if let Ok((mut transform, mut sun)) = key.single_mut() {
        let (azimuth, elevation) = (light.azimuth.to_radians(), light.elevation.to_radians());
        let direction = Vec3::new(
            azimuth.sin() * elevation.cos(),
            elevation.sin(),
            azimuth.cos() * elevation.cos(),
        );
        *transform = Transform::from_translation(direction).looking_at(Vec3::ZERO, Vec3::Y);
        sun.illuminance = light.key;
        sun.color = super::settings::Lighting::COLORS[light.color].1;
        sun.shadows_enabled = light.shadow;
    }
    if let Ok((mut transform, mut shown, mut source)) = torch.single_mut() {
        *transform = *position;
        *shown = if light.torch {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        source.shadows_enabled = light.shadow;
    }
    for mut node in &mut grid {
        *node = if config.grid {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
