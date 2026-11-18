use super::{
    Mat, Station,
    plan::{PORTALS, ROOMS, Room, Sector, get},
};
use bevy::prelude::*;

const AMBER: Color = Color::srgb(1.0, 0.72, 0.43);
const FLUORESCENT: Color = Color::srgb(0.73, 0.88, 0.83);

impl Station<'_, '_, '_, '_> {
    pub(super) fn furnish(&mut self, sector: Sector) {
        let palette = &self.palette;
        let blue = palette.blue.clone();
        let amber = palette.amber_led.clone();
        let coal = palette.coal.clone();
        let steel = palette.steel.clone();
        let rust = palette.rust.clone();
        let yellow = palette.yellow.clone();
        let oxide = palette.oxide.clone();
        let room = get(sector);
        match sector {
            Sector::Dispatch => {
                self.light(Vec3::new(0.4, 3.61, -16.1), AMBER, 24_000.0, 6.4, None);
                self.light(
                    Vec3::new(1.6, 3.61, -23.4),
                    FLUORESCENT,
                    16_000.0,
                    6.3,
                    None,
                );
                self.sign(
                    "ESTAÇÃO 07",
                    Vec3::new(4.75, 2.16, -25.70),
                    Vec2::new(2.7, 0.46),
                    0.0,
                );
                self.spill(Vec3::new(6.08, 2.22, -24.66), FLUORESCENT, 2_200.0, 3.2);
                self.desk(Vec3::new(-5.7, 0.0, -16.75), false);
                self.cabinets(-8.55, -23.25, 2, oxide.clone());
                self.block(
                    "Moldura da planta",
                    Vec3::new(-8.6, 1.76, -14.8),
                    Vec3::new(2.38, 1.78, 0.17),
                    coal.clone(),
                    false,
                );
                for x in [-9.54, -7.66] {
                    self.block(
                        "Suporte da planta",
                        Vec3::new(x, 0.68, -14.81),
                        Vec3::new(0.095, 1.36, 0.16),
                        steel.clone(),
                        true,
                    );
                    self.block(
                        "Sapata do suporte",
                        Vec3::new(x, 0.037, -14.81),
                        Vec3::new(0.32, 0.074, 0.42),
                        coal.clone(),
                        true,
                    );
                }
                self.plan_board(Vec3::new(-8.6, 1.76, -14.696));
                self.sign(
                    "PLANTA",
                    Vec3::new(-8.6, 2.80, -14.697),
                    Vec2::new(1.06, 0.21),
                    0.0,
                );
                self.markings(-2.58, -24.4, 9, yellow.clone());
                self.dispatch_mechanisms();
            }
            Sector::Dock => {
                for (z, x) in [(-6.0, -24.0), (2.7, -14.8), (7.1, -23.5)] {
                    self.light(Vec3::new(x, 5.17, z), AMBER, 27_000.0, 8.0, None);
                }
                let loading_wall = -29.70;
                self.spill(Vec3::new(-25.5, 2.55, -1.85), AMBER, 11_400.0, 7.1);
                self.round(
                    "Trilho superior de carga",
                    Vec3::new(-27.92, 4.84, -11.64),
                    Vec3::new(-27.92, 4.84, 9.62),
                    0.22,
                    steel.clone(),
                );
                self.hanging_tag(Vec3::new(-27.93, 4.73, -3.72), 1.77, 2.56);
                self.block(
                    "Portão de carga abaixado",
                    Vec3::new(loading_wall, 1.95, -1.2),
                    Vec3::new(0.12, 3.9, 5.7),
                    coal.clone(),
                    false,
                );
                for row in 0..21 {
                    self.block(
                        "Ripa da porta de carga",
                        Vec3::new(loading_wall + 0.07, 0.09 + row as f32 * 0.17, -1.2),
                        Vec3::new(0.045, 0.023, 5.53),
                        steel.clone(),
                        false,
                    );
                }
                self.sign(
                    "CARGA 02",
                    Vec3::new(loading_wall + 0.08, 4.14, -1.2),
                    Vec2::new(1.95, 0.35),
                    std::f32::consts::FRAC_PI_2,
                );
                for (x, z, stack) in [
                    (-26.5, 3.4, 2),
                    (-26.1, 6.0, 1),
                    (-19.5, 6.35, 2),
                    (-17.1, -4.6, 1),
                    (-23.6, -7.85, 2),
                ] {
                    self.crates(Vec3::new(x, 0.0, z), stack);
                }
                self.block(
                    "Batente de doca",
                    Vec3::new(-27.89, 0.48, -3.9),
                    Vec3::new(0.19, 0.96, 0.17),
                    yellow.clone(),
                    true,
                );
                self.block(
                    "Batente de doca",
                    Vec3::new(-27.89, 0.48, 1.48),
                    Vec3::new(0.19, 0.96, 0.17),
                    yellow.clone(),
                    true,
                );
            }
            Sector::Stores => {
                for (x, z) in [(-22.0, -22.0), (-16.5, -16.4)] {
                    self.light(Vec3::new(x, 3.91, z), FLUORESCENT, 19_000.0, 7.0, None);
                }
                for (x, z) in [
                    (-26.5, -22.4),
                    (-26.5, -17.6),
                    (-15.0, -23.6),
                    (-15.0, -14.25),
                ] {
                    self.rack(Vec3::new(x, 0.0, z), 3.42, false);
                }
                self.crates(Vec3::new(-24.4, 0.0, -14.8), 1);
                self.sign(
                    "ESTOQUE 03",
                    Vec3::new(-20.5, 2.35, -25.7),
                    Vec2::new(1.6, 0.25),
                    0.0,
                );
            }
            Sector::Records => {
                for (x, z, tint) in [
                    (13.8, -5.5, AMBER),
                    (25.3, -8.1, FLUORESCENT),
                    (23.7, 5.0, FLUORESCENT),
                    (13.6, 5.7, AMBER),
                ] {
                    self.light(Vec3::new(x, 3.02, z), tint, 14_500.0, 5.4, None);
                }
                for (x, z) in [
                    (12.0, 3.80),
                    (16.7, -7.9),
                    (18.2, 5.3),
                    (20.9, -3.5),
                    (20.9, 0.78),
                    (20.9, 4.7),
                    (25.0, -8.25),
                    (25.0, -3.40),
                    (25.0, 1.10),
                    (29.15, 1.3),
                    (29.15, 5.6),
                ] {
                    self.rack(Vec3::new(x, 0.0, z), 3.25, true);
                }
                self.spill(
                    Vec3::new(21.13, 2.47, -1.7),
                    Color::srgb(0.78, 0.89, 0.94),
                    5100.0,
                    6.1,
                );
                self.desk(Vec3::new(27.6, 0.0, -8.3), false);
                self.sign(
                    "ARQUIVO 01",
                    Vec3::new(13.0, 2.31, -11.71),
                    Vec2::new(1.6, 0.25),
                    0.0,
                );
            }
            Sector::Lockers => {
                for (x, z) in [(15.5, -17.6), (24.8, -20.7)] {
                    self.light(
                        Vec3::new(x, 3.21, z),
                        Color::srgb(0.67, 0.81, 0.99),
                        14_800.0,
                        6.3,
                        None,
                    );
                }
                self.cabinets(13.4, -24.8, 6, blue.clone());
                self.cabinets(25.0, -13.04, 4, oxide);
                for x in [14.3, 24.8] {
                    self.block(
                        "Banco de troca",
                        Vec3::new(x, 0.24, -18.15),
                        Vec3::new(3.1, 0.48, 0.47),
                        self.palette.wood.clone(),
                        true,
                    );
                    self.block(
                        "Sapata do banco",
                        Vec3::new(x, 0.033, -18.15),
                        Vec3::new(3.15, 0.066, 0.50),
                        coal.clone(),
                        false,
                    );
                }
                self.block(
                    "Balcão da pia",
                    Vec3::new(27.85, 0.44, -23.65),
                    Vec3::new(1.55, 0.88, 1.0),
                    self.palette.pale.clone(),
                    true,
                );
                self.block(
                    "Chapa espelhada manchada",
                    Vec3::new(28.67, 1.63, -23.65),
                    Vec3::new(0.065, 0.83, 1.15),
                    self.palette.oxide.clone(),
                    false,
                );
            }
            Sector::Inspection => {
                for (x, z, tint) in [
                    (-23.5, -69.1, AMBER),
                    (-17.0, -74.1, FLUORESCENT),
                    (-26.6, -79.1, AMBER),
                ] {
                    self.light(Vec3::new(x, 3.30, z), tint, 14_800.0, 6.0, None);
                }
                for z in [-68.2, -71.0, -76.5, -79.3] {
                    self.block(
                        "Nervura amarela",
                        Vec3::new(-31.8, 1.55, z),
                        Vec3::new(0.28, 3.1, 0.32),
                        yellow.clone(),
                        false,
                    );
                    self.block(
                        "Travessa da inspeção",
                        Vec3::new(-23.0, 3.17, z),
                        Vec3::new(19.2, 0.22, 0.18),
                        coal.clone(),
                        false,
                    );
                }
                self.round(
                    "Coletor na inspeção",
                    Vec3::new(-31.45, 2.48, -81.68),
                    Vec3::new(-31.45, 2.48, -66.32),
                    0.31,
                    rust,
                );
                self.rack(Vec3::new(-29.55, 0.0, -78.6), 3.22, false);
                self.crates(Vec3::new(-16.25, 0.0, -79.35), 1);
                self.block(
                    "Luz indicadora da tubulação",
                    Vec3::new(-32.72, 1.91, -74.0),
                    Vec3::new(0.055, 0.094, 0.22),
                    amber,
                    false,
                );
            }
            Sector::Control => {
                for (x, z) in [(18.5, -70.8), (28.0, -81.3)] {
                    self.light(
                        Vec3::new(x, 3.51, z),
                        Color::srgb(0.63, 0.83, 0.90),
                        14_400.0,
                        6.6,
                        None,
                    );
                }
                self.spill(
                    Vec3::new(30.7, 1.75, -82.0),
                    Color::srgb(0.54, 0.91, 0.85),
                    2300.0,
                    3.6,
                );
                self.desk(Vec3::new(29.8, 0.0, -83.8), true);
                self.desk(Vec3::new(17.5, 0.0, -82.2), true);
                self.fan(Vec3::new(32.44, 2.58, -85.68), 0.0, 2.21);
                self.cabinets(31.3, -67.15, 3, blue);
                self.block(
                    "Painel antigo de linha",
                    Vec3::new(24.0, 1.98, -85.72),
                    Vec3::new(6.2, 1.48, 0.10),
                    coal.clone(),
                    false,
                );
                for i in 0..8 {
                    let x = 21.45 + i as f32 * 0.72;
                    let led = if i == 3 || i == 6 {
                        self.palette.amber_led.clone()
                    } else {
                        self.palette.dim_screen.clone()
                    };
                    self.block(
                        "Estado dos ramais",
                        Vec3::new(x, 1.91 + (i % 3) as f32 * 0.19, -85.658),
                        Vec3::new(0.14, 0.071, 0.021),
                        led,
                        false,
                    );
                    self.block(
                        "Linha no painel",
                        Vec3::new(x, 1.61, -85.658),
                        Vec3::new(0.50, 0.022, 0.021),
                        steel.clone(),
                        false,
                    );
                }
                self.sign(
                    "LINHA 07",
                    Vec3::new(24.0, 2.47, -85.656),
                    Vec2::new(1.75, 0.24),
                    0.0,
                );
            }
            _ => unreachable!("room {room:?} has industrial fittings"),
        }
    }

    fn cabinets(&mut self, x: f32, z: f32, count: u32, front: Mat) {
        for index in 0..count {
            let x = x + index as f32 * 0.61;
            self.block(
                "Corpo do armário",
                Vec3::new(x, 1.02, z),
                Vec3::new(0.59, 2.04, 0.54),
                self.palette.steel.clone(),
                true,
            );
            self.block(
                "Porta do armário",
                Vec3::new(x, 1.03, z + 0.274),
                Vec3::new(0.538, 1.84, 0.028),
                front.clone(),
                false,
            );
            self.block(
                "Puxador de metal",
                Vec3::new(x + 0.15, 1.06, z + 0.302),
                Vec3::new(0.038, 0.137, 0.049),
                self.palette.coal.clone(),
                false,
            );
            let label = if index % 3 == 1 {
                self.palette.paper.clone()
            } else {
                self.palette.pale.clone()
            };
            self.block(
                "Ficha do armário",
                Vec3::new(x - 0.095, 1.73, z + 0.295),
                Vec3::new(0.14, 0.062, 0.017),
                label,
                false,
            );
            for row in 0..3 {
                self.block(
                    "Frestas de ventilação",
                    Vec3::new(x, 0.29 + row as f32 * 0.049, z + 0.293),
                    Vec3::new(0.35, 0.018, 0.019),
                    self.palette.coal.clone(),
                    false,
                );
            }
        }
    }

    fn rack(&mut self, base: Vec3, width: f32, files: bool) {
        let depth = 0.69;
        let height = 2.35;
        self.block(
            "Fundo de estante",
            base + Vec3::new(0.0, height * 0.5, -depth * 0.46),
            Vec3::new(width, height, 0.072),
            self.palette.oxide.clone(),
            false,
        );
        for x in [-width * 0.48, 0.0, width * 0.48] {
            for z in [-depth * 0.47, depth * 0.47] {
                self.block(
                    "Montante da estante",
                    base + Vec3::new(x, height * 0.5, z),
                    Vec3::new(0.085, height, 0.088),
                    self.palette.steel.clone(),
                    false,
                );
            }
        }
        self.collision(
            base + Vec3::Y * (height * 0.5),
            Vec3::new(width, height, depth),
        );
        for i in 0..4 {
            let y = 0.17 + i as f32 * 0.55;
            self.block(
                "Prateleira",
                base + Vec3::Y * y,
                Vec3::new(width, 0.065, depth),
                self.palette.steel.clone(),
                false,
            );
            for column in 0..4 {
                if (i + column) % 5 == 0 {
                    continue;
                }
                let x = ((column as f32 + 0.5) / 4.0 - 0.5) * width * 0.95;
                let h = 0.18 + (column + (i * 2) % 3) as f32 * 0.046;
                let color = match (i + column) % 4 {
                    0 => self.palette.paper.clone(),
                    1 => self.palette.wood.clone(),
                    2 => self.palette.pale.clone(),
                    _ => self.palette.oxide.clone(),
                };
                let item_width = if files { width * 0.143 } else { width * 0.195 };
                self.block(
                    if files {
                        "Caixa de registros"
                    } else {
                        "Caixa de material"
                    },
                    base + Vec3::new(x, y + 0.033 + h * 0.5, -0.035),
                    Vec3::new(item_width, h, 0.45),
                    color,
                    false,
                );
                self.block(
                    "Etiqueta gasta",
                    base + Vec3::new(x - 0.04, y + 0.10, 0.193),
                    Vec3::new(item_width * 0.36, 0.046, 0.004),
                    self.palette.paper.clone(),
                    false,
                );
            }
        }
    }

    fn crates(&mut self, base: Vec3, count: u32) {
        self.block(
            "Palete",
            base + Vec3::Y * 0.10,
            Vec3::new(1.47, 0.20, 1.22),
            self.palette.wood.clone(),
            true,
        );
        for i in 0..count {
            let y = 0.54 + i as f32 * 0.68;
            let size = Vec3::new(1.34 - i as f32 * 0.12, 0.68, 1.09 - i as f32 * 0.08);
            self.block(
                "Engradado de serviço",
                base + Vec3::Y * y,
                size,
                self.palette.wood.clone(),
                true,
            );
            for x in [-0.42, 0.42] {
                self.block(
                    "Cinta do engradado",
                    base + Vec3::new(x, y, size.z * 0.5 + 0.007),
                    Vec3::new(0.093, size.y, 0.03),
                    self.palette.coal.clone(),
                    false,
                );
            }
            self.block(
                "Etiqueta de carga",
                base + Vec3::new(0.14, y + 0.075, size.z * 0.5 + 0.016),
                Vec3::new(0.25, 0.12, 0.018),
                self.palette.paper.clone(),
                false,
            );
        }
    }

    fn desk(&mut self, base: Vec3, screens: bool) {
        self.block(
            "Mesa de trabalho",
            base + Vec3::Y * 0.46,
            Vec3::new(2.55, 0.92, 0.89),
            self.palette.oxide.clone(),
            true,
        );
        self.block(
            "Tampo laminado",
            base + Vec3::Y * 0.895,
            Vec3::new(2.58, 0.05, 0.926),
            self.palette.wood.clone(),
            false,
        );
        for x in [-0.70, 0.61] {
            for j in 0..3 {
                self.block(
                    "Formulário abandonado",
                    base + Vec3::new(x + j as f32 * 0.026, 0.926 + j as f32 * 0.006, 0.14),
                    Vec3::new(0.285, 0.008, 0.19),
                    self.palette.paper.clone(),
                    false,
                );
            }
        }
        for x in [-0.28, 0.48] {
            self.terminal(base + Vec3::X * x, screens);
        }
    }

    fn markings(&mut self, x: f32, z: f32, count: u32, material: Mat) {
        for i in 0..count {
            self.block(
                "Faixa desbotada no piso",
                Vec3::new(x + i as f32 * 0.56, 0.006, z),
                Vec3::new(0.32, 0.008, 0.19),
                material.clone(),
                false,
            );
        }
    }

    fn plan_board(&mut self, position: Vec3) {
        use bevy::{
            asset::RenderAssetUsages,
            render::render_resource::{Extent3d, TextureDimension, TextureFormat},
        };
        const WIDTH: u32 = 288;
        const HEIGHT: u32 = 330;
        let rooms: &[Room] = ROOMS;
        let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                let xy = Vec2::new(
                    column as f32 * 82.0 / WIDTH as f32 - 40.0,
                    row as f32 * 119.0 / HEIGHT as f32 - 104.0,
                );
                let colour = if PORTALS.iter().any(|p| xy.distance_squared(p.at) < 0.64) {
                    [196, 157, 90]
                } else if let Some(room) = rooms.iter().find(|room| room.contains(xy)) {
                    let edge = (xy - room.min).min(room.max - xy).min_element();
                    if edge < 0.43 {
                        [114, 154, 142]
                    } else {
                        [43, 68, 63]
                    }
                } else if column % 16 == 0 || row % 16 == 0 {
                    [29, 50, 47]
                } else {
                    [23, 36, 34]
                };
                pixels.extend_from_slice(&[colour[0], colour[1], colour[2], 255]);
            }
        }
        let image = self.images.add(Image::new(
            Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ));
        let mat = self.materials.add(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            ..default()
        });
        self.b.commands.spawn((
            Name::new("Planta da estação"),
            Mesh3d(self.plate.clone()),
            MeshMaterial3d(mat),
            Transform::from_translation(position).with_scale(Vec3::new(1.98, 1.56, 1.0)),
        ));
    }
}
