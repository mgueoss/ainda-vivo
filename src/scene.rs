use crate::{app::*, interaction::Interactable};
use ainda_vivo_core::{collision::Aabb, movement::SPAWN, story::Action};
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

#[derive(Component)]
pub enum SceneRole {
    Fuse,
    Gate,
    Presence,
    PowerLamp,
}

struct Builder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    cube: Handle<Mesh>,
    solids: &'a mut Vec<Solid>,
}

impl Builder<'_, '_, '_> {
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
    mut world: ResMut<CollisionWorld>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let wall = materials.add(StandardMaterial {
        base_color: Color::srgb(0.34, 0.39, 0.36),
        perceptual_roughness: 0.96,
        ..default()
    });
    let floor = materials.add(StandardMaterial {
        base_color: Color::srgb(0.19, 0.22, 0.22),
        perceptual_roughness: 0.93,
        ..default()
    });
    let ceiling = materials.add(Color::srgb(0.25, 0.27, 0.26));
    let dark = materials.add(Color::srgb(0.065, 0.09, 0.09));
    let wood = materials.add(Color::srgb(0.28, 0.20, 0.12));
    let paper = materials.add(Color::srgb(0.72, 0.67, 0.49));
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.27, 0.25),
        metallic: 0.5,
        perceptual_roughness: 0.6,
        ..default()
    });
    let fuse_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.93, 0.61, 0.18),
        emissive: LinearRgba::rgb(0.5, 0.22, 0.025),
        ..default()
    });
    let green = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.42, 0.30),
        emissive: LinearRgba::rgb(0.5, 2.5, 1.4),
        ..default()
    });
    let bulb = materials.add(StandardMaterial {
        base_color: Color::srgb(0.76, 0.83, 0.75),
        emissive: LinearRgba::rgb(2.0, 2.4, 2.0),
        ..default()
    });
    let shadow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.018, 0.023, 0.025),
        perceptual_roughness: 1.0,
        ..default()
    });
    let mut b = Builder {
        commands: &mut commands,
        cube,
        solids: &mut world.0,
    };
    b.block(
        "Piso",
        Vec3::new(0.0, -0.15, -1.0),
        Vec3::new(16.0, 0.3, 22.0),
        floor.clone(),
        true,
    );
    b.block(
        "Teto",
        Vec3::new(0.0, 3.15, -1.0),
        Vec3::new(16.0, 0.3, 22.0),
        ceiling,
        true,
    );
    for x in [-8.0, 8.0] {
        b.block(
            "Parede externa",
            Vec3::new(x, 1.5, -1.0),
            Vec3::new(0.3, 3.0, 22.3),
            wall.clone(),
            true,
        );
    }
    for z in [-12.0, 10.0] {
        b.block(
            "Parede externa",
            Vec3::new(0.0, 1.5, z),
            Vec3::new(16.0, 3.0, 0.3),
            wall.clone(),
            true,
        );
    }
    // Wide door openings in the corridor: left at z=4, right at z=-4.
    for (x, gaps) in [
        (-1.75, [(-12.0, 2.8), (5.2, 10.0)]),
        (1.75, [(-12.0, -5.2), (-2.8, 10.0)]),
    ] {
        for (start, end) in gaps {
            b.block(
                "Parede do corredor",
                Vec3::new(x, 1.5, (start + end) * 0.5),
                Vec3::new(0.3, 3.0, end - start),
                wall.clone(),
                true,
            );
            b.block(
                "Faixa inferior",
                Vec3::new(x * 0.901, 0.48, (start + end) * 0.5),
                Vec3::new(0.025, 0.35, end - start),
                dark.clone(),
                false,
            );
        }
    }
    // Floor seams are purely visual; no tiny collision edges to catch the player.
    for z in -11..10 {
        b.block(
            "Junta do piso",
            Vec3::new(0.0, 0.002, z as f32),
            Vec3::new(3.2, 0.002, 0.018),
            dark.clone(),
            false,
        );
    }
    b.block(
        "Mesa do fusível",
        Vec3::new(-5.0, 0.45, 4.0),
        Vec3::new(1.5, 0.9, 1.0),
        wood.clone(),
        true,
    );
    let fuse_center = Vec3::new(-5.0, 1.02, 4.0);
    let fuse = b.block(
        "Fusível",
        fuse_center,
        Vec3::new(0.32, 0.20, 0.20),
        fuse_mat,
        false,
    );
    b.commands.entity(fuse).insert((
        SceneRole::Fuse,
        Interactable {
            bounds: Aabb::from_center_size(fuse_center, Vec3::new(0.42, 0.28, 0.32)),
            action: Action::TakeFuse,
            label: "recolher fusível",
        },
    ));
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
        label: "ler registro 01",
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
    let panel_center = Vec3::new(1.42, 1.3, -8.6);
    let panel = b.block(
        "Quadro elétrico",
        panel_center,
        Vec3::new(0.24, 0.7, 0.6),
        metal.clone(),
        true,
    );
    b.commands.entity(panel).insert(Interactable {
        bounds: Aabb::from_center_size(panel_center, Vec3::new(0.3, 0.76, 0.68)),
        action: Action::RestorePower,
        label: "restaurar energia",
    });
    let gate_center = Vec3::new(0.0, 1.35, -9.8);
    let gate_size = Vec3::new(3.2, 2.7, 0.24);
    let gate = b.block("Porta de saída", gate_center, gate_size, dark, false);
    b.solids.push(Solid {
        bounds: Aabb::from_center_size(gate_center, gate_size),
        gate: true,
    });
    b.commands.entity(gate).insert(SceneRole::Gate);
    b.block(
        "Sinal da saída",
        Vec3::new(0.0, 2.83, -9.65),
        Vec3::new(0.8, 0.18, 0.04),
        green.clone(),
        false,
    );
    let indicator = b.block(
        "Indicador do quadro",
        Vec3::new(1.285, 1.48, -8.6),
        Vec3::new(0.025, 0.1, 0.14),
        green.clone(),
        false,
    );
    b.commands
        .entity(indicator)
        .insert((SceneRole::PowerLamp, Visibility::Hidden));
    // The prototype presence is an inert, distant silhouette; never attacks or
    // teleports onto the camera. AI, death and private illusions are future work.
    let presence = b.block(
        "A Presença — silhueta provisória",
        Vec3::new(5.6, 0.9, -8.0),
        Vec3::new(0.48, 1.8, 0.3),
        shadow.clone(),
        false,
    );
    b.commands
        .entity(presence)
        .insert((SceneRole::Presence, Visibility::Hidden));
    let head = b
        .commands
        .spawn((
            Name::new("Silhueta: cabeça"),
            Mesh3d(meshes.add(Sphere::new(0.18))),
            MeshMaterial3d(shadow),
            Transform::from_xyz(5.6, 1.92, -8.0),
            SceneRole::Presence,
            Visibility::Hidden,
        ))
        .id();
    let _ = head;
    for (x, z) in [
        (0.0, 7.0),
        (0.0, 0.0),
        (0.0, -7.0),
        (-5.0, 4.0),
        (5.0, -4.0),
    ] {
        b.block(
            "Luminária",
            Vec3::new(x, 2.96, z),
            Vec3::new(0.7, 0.05, 0.22),
            bulb.clone(),
            false,
        );
        b.commands.spawn((
            PointLight {
                color: Color::srgb(0.68, 0.79, 0.73),
                intensity: 8_000.0,
                range: 7.5,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(x, 2.7, z),
        ));
    }
    b.commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.8, 0.5),
            intensity: 3_000.0,
            range: 4.0,
            ..default()
        },
        Transform::from_xyz(0.0, 2.3, -11.0),
    ));
    b.commands.spawn((
        Name::new("Câmera local"),
        MainCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 65.0_f32.to_radians(),
            near: 0.05,
            far: 65.0,
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
            color: Color::srgb(1.0, 0.94, 0.79),
            intensity: 140_000.0,
            range: 18.0,
            inner_angle: 0.27,
            outer_angle: 0.52,
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
            SceneRole::Fuse => !session.progress.has_fuse,
            SceneRole::Gate => !session.progress.power_restored,
            SceneRole::PowerLamp => session.progress.power_restored,
            SceneRole::Presence => session.progress.has_fuse && !session.progress.power_restored,
        };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
