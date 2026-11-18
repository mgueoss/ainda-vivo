//! Original, procedural set dressing for the prototype's maintenance area.
use super::{Builder, surfaces};
use crate::atmosphere::{Circuit, CircuitLight, DUCT};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub fn setup(
    b: &mut Builder,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let frame = materials.add(Color::srgb(0.25, 0.30, 0.29));
    let black = materials.add(Color::srgb(0.108, 0.135, 0.133));
    let conduit = materials.add(StandardMaterial {
        base_color: Color::srgb(0.31, 0.34, 0.32),
        metallic: 0.25,
        perceptual_roughness: 0.67,
        ..default()
    });
    let worn_yellow = materials.add(Color::srgb(0.60, 0.46, 0.19));
    let rust = materials.add(Color::srgb(0.29, 0.165, 0.098));
    let paper = materials.add(Color::srgb(0.72, 0.66, 0.49));

    for (x, z, word, angle) in [
        (-1.75, 4.0, "OFICINA", FRAC_PI_2),
        (1.75, -4.0, "ARQUIVO", -FRAC_PI_2),
    ] {
        for edge in [-1.20, 1.20] {
            b.block(
                "Batente gasto",
                Vec3::new(x, 1.40, z + edge),
                Vec3::new(0.34, 2.80, 0.075),
                frame.clone(),
                false,
            );
        }
        b.block(
            "Travessa da porta",
            Vec3::new(x, 2.78, z),
            Vec3::new(0.36, 0.38, 2.42),
            frame.clone(),
            false,
        );
        let side = if x < 0.0 { 0.185 } else { -0.185 };
        plate(
            b,
            meshes,
            materials,
            images,
            Panel {
                word,
                position: Vec3::new(x + side, 2.74, z),
                size: Vec2::new(0.91, 0.19),
                angle,
                fill: [26, 32, 28],
                ink: [116, 118, 88],
                illuminated: true,
            },
        );
    }

    for rail in [-1.16, -0.97, 1.24] {
        b.block(
            "Eletroduto no teto",
            Vec3::new(rail, 2.858, -1.0),
            Vec3::new(0.055, 0.055, 21.66),
            conduit.clone(),
            false,
        );
        for z in [-9.0, -6.3, -3.1, 0.2, 3.4, 6.35, 9.0] {
            b.block(
                "Abraçadeira",
                Vec3::new(rail, 2.909, z),
                Vec3::new(0.14, 0.165, 0.075),
                black.clone(),
                false,
            );
        }
    }
    for z in [5.28, 2.73, -2.73, -5.27, -9.58] {
        b.block(
            "Viga no corredor",
            Vec3::new(0.0, 2.84, z),
            Vec3::new(3.2, 0.32, 0.24),
            frame.clone(),
            false,
        );
    }
    for x in [-1.585, 1.585] {
        for z in [-9.58, -5.27, -2.73, 2.73, 5.28] {
            b.block(
                "Junção do revestimento",
                Vec3::new(x, 1.40, z),
                Vec3::new(0.035, 2.80, 0.09),
                frame.clone(),
                false,
            );
        }
    }

    // Quiet visual evidence of maintenance left unfinished.
    b.block(
        "Bandeja para fusível",
        Vec3::new(-5.0, 0.919, 4.0),
        Vec3::new(0.50, 0.035, 0.44),
        black.clone(),
        false,
    );
    b.block(
        "Marca de inventário",
        Vec3::new(-4.75, 0.940, 4.16),
        Vec3::new(0.12, 0.004, 0.08),
        worn_yellow.clone(),
        false,
    );
    let tool = b.block(
        "Chave sobre a bancada",
        Vec3::new(-5.43, 0.921, 4.28),
        Vec3::new(0.31, 0.027, 0.045),
        conduit.clone(),
        false,
    );
    b.commands.entity(tool).insert(
        Transform::from_xyz(-5.43, 0.921, 4.28)
            .with_rotation(Quat::from_rotation_y(0.51))
            .with_scale(Vec3::new(0.31, 0.027, 0.045)),
    );
    for z in [0.69, 1.28] {
        b.block(
            "Dobradiça do armário",
            Vec3::new(-6.879, 1.44, z),
            Vec3::new(0.045, 0.105, 0.064),
            rust.clone(),
            false,
        );
        b.block(
            "Puxador do armário",
            Vec3::new(-6.865, 1.02, z + 0.16),
            Vec3::new(0.074, 0.30, 0.032),
            black.clone(),
            false,
        );
    }
    for x in [-6.9, -6.3, -5.7] {
        b.block(
            "Faixa na caixa",
            Vec3::new(x + 1.8, 0.43, 6.947),
            Vec3::new(0.14, 0.098, 0.008),
            worn_yellow.clone(),
            false,
        );
    }
    for i in 0..5 {
        b.block(
            "Folhas abandonadas",
            Vec3::new(5.78 + i as f32 * 0.014, 0.908 + i as f32 * 0.004, -3.72),
            Vec3::new(0.25, 0.006, 0.19),
            paper.clone(),
            false,
        );
    }
    for i in 0..5 {
        b.block(
            "Linhas no registro",
            Vec3::new(5.16, 0.934, -4.13 + i as f32 * 0.052),
            Vec3::new(0.22 + (i % 3) as f32 * 0.065, 0.002, 0.008),
            rust.clone(),
            false,
        );
    }

    // The visible grille anchors the distant air sound to the right-hand wall.
    b.block(
        "Moldura de ventilação",
        DUCT + Vec3::X * 0.132,
        Vec3::new(0.035, 0.49, 0.96),
        conduit.clone(),
        false,
    );
    b.block(
        "Fundo da ventilação",
        DUCT + Vec3::X * 0.111,
        Vec3::new(0.013, 0.38, 0.82),
        black.clone(),
        false,
    );
    for i in -3..=3 {
        let position = DUCT + Vec3::new(0.095, i as f32 * 0.052, 0.0);
        b.block(
            "Lâmina de ventilação",
            position,
            Vec3::new(0.025, 0.019, 0.78),
            frame.clone(),
            false,
        );
    }
    for (z, height) in [(-6.90, 0.30), (-6.71, 0.46), (-6.54, 0.26)] {
        b.block(
            "Mancha de ferrugem",
            Vec3::new(1.598, 2.01 - height * 0.39, z),
            Vec3::new(0.004, height, 0.028),
            rust.clone(),
            false,
        );
    }
    let amber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.24, 0.10),
        emissive: LinearRgba::rgb(0.56, 0.25, 0.075),
        ..default()
    });
    b.block(
        "Luz de manutenção",
        Vec3::new(-1.581, 2.50, 5.33),
        Vec3::new(0.062, 0.095, 0.23),
        amber,
        false,
    );

    for fixture in [
        Fixture::new(
            Vec3::new(0.0, 2.945, 7.12),
            Color::srgb(1.0, 0.76, 0.51),
            14_000.0,
            5.2,
        ),
        Fixture::new(
            Vec3::new(-0.47, 2.945, 0.12),
            Color::srgb(0.60, 0.80, 0.76),
            4300.0,
            4.4,
        ),
        Fixture::new(
            Vec3::new(0.60, 2.945, -7.28),
            Color::srgb(0.69, 0.82, 0.78),
            6550.0,
            4.4,
        ),
        Fixture::new(
            Vec3::new(-4.54, 2.945, 4.07),
            Color::srgb(1.0, 0.74, 0.43),
            15_400.0,
            5.0,
        )
        .with_circuit(Circuit::Workshop),
        Fixture::new(
            Vec3::new(5.3, 2.945, -4.15),
            Color::srgb(0.63, 0.77, 0.85),
            6300.0,
            5.0,
        ),
    ] {
        fixture.spawn(b, materials, &black);
    }
    // Broad, weak indirect fill in the rooms, with the stronger pools belonging to the practical lights.
    for (center, color, intensity) in [
        (
            Vec3::new(-1.49, 2.38, 5.31),
            Color::srgb(1.0, 0.65, 0.31),
            2400.0,
        ),
        (
            Vec3::new(-5.8, 2.6, 4.0),
            Color::srgb(0.91, 0.73, 0.53),
            2700.0,
        ),
        (
            Vec3::new(5.7, 2.5, -4.0),
            Color::srgb(0.53, 0.68, 0.74),
            1850.0,
        ),
    ] {
        b.commands.spawn((
            PointLight {
                color,
                intensity: intensity * 0.58,
                range: 3.7,
                ..default()
            },
            Transform::from_translation(center),
        ));
    }
}

struct Fixture {
    position: Vec3,
    color: Color,
    lumens: f32,
    range: f32,
    circuit: Option<Circuit>,
}

impl Fixture {
    fn new(position: Vec3, color: Color, lumens: f32, range: f32) -> Self {
        Self {
            position,
            color,
            lumens,
            range,
            circuit: None,
        }
    }

    fn with_circuit(mut self, circuit: Circuit) -> Self {
        self.circuit = Some(circuit);
        self
    }

    fn spawn(
        self,
        b: &mut Builder,
        materials: &mut Assets<StandardMaterial>,
        housing: &Handle<StandardMaterial>,
    ) {
        let lumens = self.lumens * 0.76;
        b.block(
            "Carcaça da luminária",
            self.position,
            Vec3::new(0.88, 0.095, 0.31),
            housing.clone(),
            false,
        );
        let glow = self.color.to_linear() * 1.8;
        let factor = if matches!(self.circuit, Some(Circuit::Exit)) {
            0.06
        } else {
            1.0
        };
        let lens = materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.22, 0.21),
            emissive: glow * factor,
            ..default()
        });
        b.block(
            "Difusor da luminária",
            self.position - Vec3::Y * 0.056,
            Vec3::new(0.69, 0.027, 0.155),
            lens.clone(),
            false,
        );
        let light = b
            .commands
            .spawn((
                SpotLight {
                    color: self.color,
                    intensity: lumens * factor,
                    range: self.range,
                    inner_angle: 0.43,
                    outer_angle: 0.79,
                    ..default()
                },
                Transform::from_translation(self.position - Vec3::Y * 0.15)
                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            ))
            .id();
        if let Some(circuit) = self.circuit {
            b.commands.entity(light).insert(CircuitLight {
                circuit,
                lumens,
                glow,
                lens,
            });
        }
    }
}

struct Panel<'a> {
    word: &'a str,
    position: Vec3,
    size: Vec2,
    angle: f32,
    fill: [u8; 3],
    ink: [u8; 3],
    illuminated: bool,
}

fn plate(
    b: &mut Builder,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    panel: Panel,
) {
    let image = images.add(surfaces::sign(panel.word, panel.fill, panel.ink));
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image),
        unlit: panel.illuminated,
        perceptual_roughness: 0.92,
        ..default()
    });
    b.commands.spawn((
        Name::new(panel.word.to_owned()),
        Mesh3d(meshes.add(Rectangle::new(panel.size.x, panel.size.y))),
        MeshMaterial3d(material),
        Transform::from_translation(panel.position)
            .with_rotation(Quat::from_rotation_y(panel.angle)),
    ));
}
