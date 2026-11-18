use crate::{app::*, interaction::Interactable};
use ainda_vivo_core::{collision::Aabb, movement::SPAWN, story::Action};
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

mod dressing;
pub mod presence;
pub mod recording;
pub mod station;
mod surfaces;
pub mod water;

pub const DISTANCE_DARK: Color = Color::srgb(0.010, 0.015, 0.018);
pub const DISTANCE_START: f32 = 5.2;
pub const DISTANCE_END: f32 = 18.6;

#[derive(Component)]
pub enum SceneRole {
    Fuse,
    Gate,
    PowerLamp,
}

struct Builder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    cube: Handle<Mesh>,
    solids: &'a mut Vec<Solid>,
}

impl Builder<'_, '_, '_> {
    fn surface(
        &mut self,
        meshes: &mut Assets<Mesh>,
        name: &str,
        center: Vec3,
        size: Vec3,
        material: Handle<StandardMaterial>,
    ) {
        let entity = self.block(name, center, size, material, true);
        self.commands.entity(entity).insert(Mesh3d(
            meshes.add(surfaces::world_uv_cube(center, size, 3.2)),
        ));
    }

    fn block(
        &mut self,
        name: &str,
        center: Vec3,
        size: Vec3,
        material: Handle<StandardMaterial>,
        solid: bool,
    ) -> Entity {
        let entity = self
            .commands
            .spawn((
                Name::new(name.to_owned()),
                Mesh3d(self.cube.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(center).with_scale(size),
            ))
            .id();
        if solid {
            self.solids.push(Solid {
                bounds: Aabb::from_center_size(center, size),
                gate: false,
            });
        }
        entity
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut world: ResMut<CollisionWorld>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let wall = surfaces::material(surfaces::Surface::Paint, &mut images, &mut materials);
    let floor = surfaces::material(surfaces::Surface::Tile, &mut images, &mut materials);
    let ceiling = surfaces::material(surfaces::Surface::Ceiling, &mut images, &mut materials);
    let dark = materials.add(Color::srgb(0.18, 0.25, 0.232));
    let wood = materials.add(Color::srgb(0.28, 0.20, 0.12));
    let paper = materials.add(Color::srgb(0.72, 0.67, 0.49));
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.27, 0.25),
        metallic: 0.5,
        perceptual_roughness: 0.6,
        ..default()
    });
    let fuse_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.89, 0.68, 0.27),
        emissive: LinearRgba::rgb(0.26, 0.124, 0.012),
        perceptual_roughness: 0.62,
        ..default()
    });
    let mut b = Builder {
        commands: &mut commands,
        cube,
        solids: &mut world.0,
    };
    station::setup(
        &mut b,
        &mut meshes,
        &mut materials,
        &mut images,
        station::Shell {
            wall: wall.clone(),
            tile: floor,
            ceiling,
        },
    );
    // Wide door openings in the corridor: left at z=4, right at z=-4.
    for (x, gaps) in [
        (-1.75, [(-12.0, 2.8), (5.2, 10.0)]),
        (1.75, [(-12.0, -5.2), (-2.8, 10.0)]),
    ] {
        for (start, end) in gaps {
            b.surface(
                &mut meshes,
                "Parede do corredor",
                Vec3::new(x, 1.5, (start + end) * 0.5),
                Vec3::new(0.3, 3.0, end - start),
                wall.clone(),
            );
            b.block(
                "Faixa inferior",
                Vec3::new(x * 0.91, 0.34, (start + end) * 0.5),
                Vec3::new(0.026, 0.68, end - start),
                dark.clone(),
                false,
            );
        }
    }
    b.block(
        "Mesa do fusível",
        Vec3::new(-5.0, 0.45, 4.0),
        Vec3::new(1.5, 0.9, 1.0),
        wood.clone(),
        true,
    );
    let fuse_center = Vec3::new(-5.0, 1.02, 4.0);
    let along_x = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    b.commands.spawn((
        Name::new("Fusível cerâmico"),
        Mesh3d(meshes.add(Cylinder::new(0.076, 0.28))),
        MeshMaterial3d(fuse_mat),
        Transform::from_translation(fuse_center).with_rotation(along_x),
        SceneRole::Fuse,
        Interactable {
            bounds: Aabb::from_center_size(fuse_center, Vec3::new(0.42, 0.28, 0.32)),
            action: Action::TakeFuse,
        },
    ));
    let end_cap = meshes.add(Cylinder::new(0.083, 0.056));
    for x in [-0.137, 0.137] {
        b.commands.spawn((
            Name::new("Contato do fusível"),
            SceneRole::Fuse,
            Mesh3d(end_cap.clone()),
            MeshMaterial3d(metal.clone()),
            Transform::from_translation(fuse_center + Vec3::X * x).with_rotation(along_x),
        ));
    }
    b.block(
        "Armário",
        Vec3::new(-7.3, 1.0, 1.0),
        Vec3::new(0.8, 2.0, 2.2),
        metal.clone(),
        true,
    );
    b.block(
        "Caixas",
        Vec3::new(-4.5, 0.35, 7.5),
        Vec3::new(1.3, 0.7, 1.1),
        wood.clone(),
        true,
    );
    b.block(
        "Travessa para teste de agachar",
        Vec3::new(-5.0, 1.5, 1.2),
        Vec3::new(2.5, 0.35, 0.55),
        metal.clone(),
        true,
    );
    b.block(
        "Mesa de registro",
        Vec3::new(5.2, 0.45, -4.0),
        Vec3::new(1.8, 0.9, 1.2),
        wood.clone(),
        true,
    );
    let note_center = Vec3::new(5.2, 0.92, -4.0);
    let note = b.block(
        "Registro 01",
        note_center,
        Vec3::new(0.55, 0.025, 0.4),
        paper,
        false,
    );
    b.commands.entity(note).insert(Interactable {
        bounds: Aabb::from_center_size(note_center, Vec3::new(0.6, 0.12, 0.48)),
        action: Action::ReadNote,
    });
    for z in [-7.0, 0.0] {
        b.block(
            "Estante",
            Vec3::new(7.1, 1.0, z),
            Vec3::new(0.9, 2.0, 2.2),
            metal.clone(),
            true,
        );
    }
    dressing::setup(&mut b, &mut meshes, &mut materials, &mut images);
    b.commands.spawn((
        Name::new("Câmera local"),
        MainCamera,
        bevy::ui::IsDefaultUiCamera,
        Camera3d::default(),
        bevy::camera::Exposure { ev100: 8.75 },
        DistanceFog {
            color: DISTANCE_DARK,
            falloff: FogFalloff::Linear {
                start: DISTANCE_START,
                end: DISTANCE_END,
            },
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 65.0_f32.to_radians(),
            near: 0.05,
            far: 122.0,
            ..default()
        }),
        Tonemapping::TonyMcMapface,
        Msaa::Sample4,
        Transform::from_translation(SPAWN + Vec3::Y * 1.62),
    ));
    b.commands.spawn((
        Name::new("Lanterna local"),
        Flashlight,
        SpotLight {
            color: Color::srgb(1.0, 0.956, 0.858),
            intensity: 330_000.0,
            range: 15.5,
            inner_angle: 0.24,
            outer_angle: 0.51,
            shadows_enabled: true,
            shadow_depth_bias: 0.025,
            ..default()
        },
        Transform::from_translation(SPAWN + Vec3::Y * 1.5),
    ));
    info!(
        "Mapa procedural pronto: {} colisores; física 60 Hz; sem assets externos.",
        b.solids.len()
    );
}

pub fn sync_scene(session: Res<Session>, mut objects: Query<(&SceneRole, &mut Visibility)>) {
    for (role, mut visibility) in &mut objects {
        let visible = match role {
            SceneRole::Fuse => !session.progress.has_fuse(),
            SceneRole::Gate => !session.progress.exit_unlocked(),
            SceneRole::PowerLamp => session.progress.power_restored(),
        };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
