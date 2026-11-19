use super::{
    Circuit, EXIT_GATE, Mat, PANEL, PUMP_02, PUMP_02_PANEL, SceneRole, Solid, Station,
    detail::Face,
    plan::{Sector, WATER_MAX, WATER_MIN},
};
use crate::interaction::Interactable;
use ainda_vivo_core::{collision::Aabb, story::Action};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const SODIUM: Color = Color::srgb(1.0, 0.735, 0.445);
const COLD: Color = Color::srgb(0.67, 0.86, 0.89);
const AQUA: Color = Color::srgb(0.45, 0.90, 0.77);
const EMERGENCY: Color = Color::srgb(0.69, 1.0, 0.76);

impl Station<'_, '_, '_, '_> {
    pub(super) fn install(&mut self, sector: Sector) {
        match sector {
            Sector::Pumps => self.pumps(),
            Sector::Filters => self.filters(),
            Sector::Generators => self.generators(),
            Sector::Tanks => self.tanks(),
            Sector::Electrical => self.electrical(),
            Sector::Basin => self.basin(),
            Sector::Exit => self.exit(),
            _ => unreachable!("sector {sector:?} is a workspace"),
        }
    }

    pub(super) fn collision(&mut self, center: Vec3, size: Vec3) {
        self.b.solids.push(Solid {
            bounds: Aabb::from_center_size(center, size),
            gate: false,
        });
    }

    fn pumps(&mut self) {
        let badge = self.block(
            "Identificação no gancho de retorno",
            super::BADGE,
            Vec3::new(0.17, 0.10, 0.03),
            self.palette.paper.clone(),
            false,
        );
        self.b.commands.entity(badge).insert(Interactable {
            bounds: Aabb::from_center_size(super::BADGE, Vec3::new(0.28, 0.18, 0.10)),
            action: Action::ReadBadge,
        });
        for (lane, tint) in [(-1.0, SODIUM), (1.0, COLD)] {
            for (index, z) in [-33.0, -43.55, -55.2].into_iter().enumerate() {
                let inspected = lane > 0.0 && index == 1;
                let base = if inspected {
                    PUMP_02
                } else {
                    Vec3::new(lane * 8.45, 0.0, z)
                };
                self.pump_unit(base, lane, inspected);
                self.light(
                    Vec3::new(lane * 7.20, 5.25, z + 0.6),
                    tint,
                    42_000.0,
                    8.2,
                    inspected.then_some(Circuit::Inspection),
                );
                let intensity = if index == 0 { 6200.0 } else { 4300.0 };
                let spill_z = if index == 2 && lane > 0.0 {
                    z - 1.0
                } else {
                    z + 0.7
                };
                if lane < 0.0 && index == 2 {
                    // Reuse the spill to outline the silhouette against the concrete.
                    self.spill(Vec3::new(-4.82, 2.12, -56.31), tint, 7800.0, 5.9);
                } else {
                    self.spill(Vec3::new(lane * 5.85, 2.65, spill_z), tint, intensity, 5.9);
                }
                for rail_z in [z - 1.55, z + 1.55] {
                    self.block(
                        "Linha de manutenção",
                        Vec3::new(lane * 7.68, 0.007, rail_z),
                        Vec3::new(5.04, 0.008, 0.063),
                        self.palette.yellow.clone(),
                        false,
                    );
                }
            }
            for (x, y, diameter, material) in [
                (lane * 11.35, 4.85, 0.55, self.palette.oxide.clone()),
                (lane * 12.26, 5.19, 0.34, self.palette.blue.clone()),
            ] {
                self.round(
                    "Adutora suspensa",
                    Vec3::new(x, y, -57.7),
                    Vec3::new(x, y, -26.3),
                    diameter,
                    material,
                );
                for z in [-29.0, -34.5, -40.0, -46.7, -53.3] {
                    self.round(
                        "União da adutora",
                        Vec3::new(x, y, z - 0.19),
                        Vec3::new(x, y, z + 0.19),
                        diameter * 1.24,
                        self.palette.steel.clone(),
                    );
                    self.round(
                        "Tirante da adutora",
                        Vec3::new(x, y, z),
                        Vec3::new(x, 6.15, z),
                        0.085,
                        self.palette.coal.clone(),
                    );
                }
            }
            self.block(
                "Guia da nave",
                Vec3::new(lane * 2.62, 0.004, -42.0),
                Vec3::new(0.089, 0.007, 28.06),
                self.palette.yellow.clone(),
                false,
            );
        }
        for z in [-30.4, -38.1, -48.0, -56.88] {
            self.block(
                "Viga principal",
                Vec3::new(0.0, 5.74, z),
                Vec3::new(27.58, 0.80, 0.38),
                self.palette.coal.clone(),
                false,
            );
            for x in [-13.52, 13.52] {
                self.block(
                    "Pilar da nave",
                    Vec3::new(x, 2.79, z),
                    Vec3::new(0.44, 5.58, 0.52),
                    self.palette.oxide.clone(),
                    true,
                );
            }
        }
        self.hanging_sign("ELÉTRICA →", Vec3::new(0.0, 3.52, -31.25), 3.0, 5.98);
        self.hanging_sign("DECANTAÇÃO ↑", Vec3::new(0.0, 3.62, -48.1), 3.3, 5.6);
        self.floor_directions("BOMBA 02 ↑", Vec2::new(-0.43, -33.7), 0.0);
        self.floor_directions("BOMBA 02 →", Vec2::new(1.49, -40.55), 0.0);
        self.floor_directions("BOMBA 02 ↑", Vec2::new(3.98, -46.0), PI);
        self.floor_directions("ELÉTRICA →", Vec2::new(1.11, -50.4), 0.0);
        self.floor_directions("ELÉTRICA ↑", Vec2::new(10.3, -52.5), -FRAC_PI_2);
        self.hanging_tag(Vec3::new(-1.96, 5.29, -38.1), 1.43, 2.12);
        self.hanging_tag(Vec3::new(2.54, 5.29, -48.0), 1.62, 1.07);
        self.fan(Vec3::new(13.69, 3.12, -41.21), -FRAC_PI_2, 0.56);
    }

    fn pump_unit(&mut self, base: Vec3, outer: f32, inspected: bool) {
        let toward = Vec3::X * outer;
        self.block(
            "Base chumbada",
            base + Vec3::Y * 0.17,
            Vec3::new(5.20, 0.34, 2.68),
            self.palette.concrete.clone(),
            true,
        );
        self.block(
            "Apoio do motor",
            base + Vec3::new(outer * 0.67, 0.52, 0.0),
            Vec3::new(2.4, 0.42, 1.06),
            self.palette.coal.clone(),
            true,
        );
        let center = base + Vec3::Y * 1.16;
        self.round(
            "Motor da bomba",
            center - toward * 0.36,
            center + toward * 1.77,
            1.31,
            self.palette.oxide.clone(),
        );
        self.collision(center + toward * 0.65, Vec3::new(2.11, 1.15, 1.12));
        for count in 0..8 {
            let step = -0.18 + count as f32 * 0.24;
            self.round(
                "Anel de arrefecimento do motor",
                center + toward * (step - 0.032),
                center + toward * (step + 0.040),
                1.44,
                self.palette.coal.clone(),
            );
        }
        let impeller = center - toward * 1.12;
        self.round(
            "Carcaça da bomba",
            impeller - toward * 0.54,
            impeller + toward * 0.55,
            1.76,
            self.palette.blue.clone(),
        );
        self.collision(impeller, Vec3::new(1.09, 1.52, 1.52));
        self.round(
            "Flange de inspeção",
            impeller - toward * 0.60,
            impeller - toward * 0.525,
            1.37,
            self.palette.steel.clone(),
        );
        let yaw = -outer * FRAC_PI_2;
        let flange = Face::new(impeller - toward * 0.604, yaw);
        for i in 0..8 {
            let (y, x) = (i as f32 * TAU / 8.0).sin_cos();
            self.fastener(flange, Vec3::new(x * 0.566, y * 0.566, 0.010), 0.088);
        }
        self.round(
            "Tubo de descarga",
            impeller,
            impeller + Vec3::Y * 2.9,
            0.56,
            self.palette.oxide.clone(),
        );
        self.round(
            "Coletor superior da bomba",
            impeller + Vec3::Y * 2.74,
            Vec3::new(outer * 11.55, impeller.y + 2.74, impeller.z),
            0.55,
            self.palette.oxide.clone(),
        );
        self.handwheel(impeller + Vec3::new(-outer * 0.42, 0.72, 0.0), true, 1.05);
        let pedestal = base + Vec3::new(-outer * 1.87, 0.0, 0.83);
        self.round(
            "Suporte de instrumentos",
            pedestal + Vec3::Y * 0.315,
            pedestal + Vec3::Y * 1.02,
            0.091,
            self.palette.steel.clone(),
        );
        let panel = if inspected {
            PUMP_02_PANEL
        } else {
            pedestal + Vec3::Y * 1.09
        };
        self.local_controls(Face::new(panel, yaw), inspected);
        if inspected {
            self.spill(panel + Vec3::new(-0.53, 1.04, 0.32), SODIUM, 5_100.0, 4.30);
        }
        self.block(
            "Etiqueta da bomba",
            base + Vec3::new(outer * 0.48, 0.62, 0.55),
            Vec3::new(0.29, 0.09, 0.018),
            self.palette.yellow.clone(),
            false,
        );
    }

    fn filters(&mut self) {
        for (x, z) in [
            (-28.1, -43.55),
            (-19.05, -41.8),
            (-28.2, -31.6),
            (-19.6, -30.5),
        ] {
            let base = Vec3::new(x, 0.0, z);
            self.vessel(base, 2.52, 2.8, self.palette.pale.clone());
            self.light(base + Vec3::new(0.0, 4.02, 1.2), COLD, 19_400.0, 6.8, None);
        }
        self.round(
            "Coletor dos filtros",
            Vec3::new(-29.9, 3.73, -46.4),
            Vec3::new(-29.9, 3.73, -26.4),
            0.39,
            self.palette.rust.clone(),
        );
        self.sign(
            "FILTROS 01",
            Vec3::new(-27.9, 2.65, -46.713),
            Vec2::new(1.65, 0.30),
            0.0,
        );
        self.fan(Vec3::new(-30.69, 2.61, -36.14), FRAC_PI_2, 1.75);
    }

    fn tanks(&mut self) {
        for (x, z, diameter) in [
            (-28.8, -59.8, 4.4),
            (-19.5, -60.35, 4.52),
            (-27.45, -51.25, 2.59),
        ] {
            let base = Vec3::new(x, 0.0, z);
            self.vessel(base, diameter, 3.45, self.palette.oxide.clone());
            self.light(base + Vec3::new(0.0, 4.9, 0.85), AQUA, 24_000.0, 7.0, None);
        }
        self.block(
            "Faixa de segurança da ala dos tanques",
            Vec3::new(-23.0, 0.005, -53.62),
            Vec3::new(12.0, 0.009, 0.12),
            self.palette.yellow.clone(),
            false,
        );
        self.spill(Vec3::new(-24.7, 2.9, -63.36), AQUA, 4200.0, 4.9);
    }

    fn vessel(&mut self, base: Vec3, diameter: f32, height: f32, metal: Mat) {
        self.block(
            "Sapata quadrada do reservatório",
            base + Vec3::Y * 0.14,
            Vec3::new(diameter + 0.25, 0.28, diameter + 0.25),
            self.palette.concrete.clone(),
            true,
        );
        self.round(
            "Reservatório cilíndrico",
            base + Vec3::Y * 0.28,
            base + Vec3::Y * height,
            diameter,
            metal,
        );
        self.collision(
            base + Vec3::Y * ((height + 0.28) * 0.5),
            Vec3::new(diameter * 0.97, height - 0.28, diameter * 0.97),
        );
        for y in [0.39, height - 0.52, height - 0.14] {
            self.round(
                "Aro do reservatório",
                base + Vec3::Y * (y - 0.045),
                base + Vec3::Y * (y + 0.055),
                diameter * 1.025,
                self.palette.steel.clone(),
            );
        }
        self.round(
            "Tubo de inspeção",
            base + Vec3::new(0.0, 0.40, diameter * 0.47),
            base + Vec3::new(0.0, height - 0.25, diameter * 0.47),
            0.21,
            self.palette.blue.clone(),
        );
        self.handwheel(base + Vec3::new(0.0, 1.29, diameter * 0.56), false, 0.63);
        self.block(
            "Marca de nível",
            base + Vec3::new(-0.21, 1.73, diameter * 0.50),
            Vec3::new(0.36, 0.18, 0.022),
            self.palette.yellow.clone(),
            false,
        );
    }

    fn generators(&mut self) {
        for (x, z) in [(24.8, -33.0), (24.8, -41.35)] {
            self.light(
                Vec3::new(x - 1.3, 4.62, z),
                Color::srgb(1.0, 0.631, 0.44),
                25_000.0,
                7.1,
                None,
            );
            self.block(
                "Bloco do gerador",
                Vec3::new(x, 1.16, z),
                Vec3::new(4.16, 2.32, 2.58),
                self.palette.oxide.clone(),
                true,
            );
            self.block(
                "Guarnição do gerador",
                Vec3::new(x - 2.09, 1.17, z + 0.1),
                Vec3::new(0.055, 1.93, 2.24),
                self.palette.coal.clone(),
                false,
            );
            for slat in 0..11 {
                self.block(
                    "Grelha do gerador",
                    Vec3::new(x - 2.133, 0.53 + slat as f32 * 0.127, z + 0.11),
                    Vec3::new(0.065, 0.035, 2.03),
                    self.palette.steel.clone(),
                    false,
                );
            }
            for cap in [-0.9, 0.9] {
                self.round(
                    "Escape do gerador",
                    Vec3::new(x + cap, 2.28, z),
                    Vec3::new(x + cap, 4.01, z),
                    0.32,
                    self.palette.rust.clone(),
                );
                self.round(
                    "Cotovelo de escape",
                    Vec3::new(x + cap, 4.01, z),
                    Vec3::new(30.65, 4.01, z),
                    0.30,
                    self.palette.rust.clone(),
                );
            }
            self.block(
                "Indicador da reserva",
                Vec3::new(x - 2.153, 2.02, z + 0.63),
                Vec3::new(0.048, 0.073, 0.14),
                self.palette.amber_led.clone(),
                false,
            );
        }
        self.block(
            "Degrau de equipamento",
            Vec3::new(16.25, 0.27, -29.9),
            Vec3::new(1.86, 0.54, 0.82),
            self.palette.coal.clone(),
            true,
        );
        self.light(Vec3::new(17.0, 4.56, -45.15), COLD, 15_900.0, 6.2, None);
    }

    fn electrical(&mut self) {
        for (x, z, tint) in [
            (19.2, -53.2, COLD),
            (28.8, -50.95, SODIUM),
            (32.18, -58.3, SODIUM),
            (23.5, -63.7, COLD),
        ] {
            self.light(Vec3::new(x, 3.81, z), tint, 18_500.0, 6.7, None);
        }
        for z in [-50.1, -51.32, -52.54, -61.57, -62.79, -64.02] {
            self.block(
                "Módulo de distribuição",
                Vec3::new(34.18, 1.22, z),
                Vec3::new(1.12, 2.44, 1.10),
                self.palette.oxide.clone(),
                true,
            );
            self.block(
                "Tampa do módulo",
                Vec3::new(33.598, 1.14, z),
                Vec3::new(0.05, 1.91, 0.94),
                self.palette.steel.clone(),
                false,
            );
            for (band, mat) in [
                (0.07, self.palette.amber_led.clone()),
                (-0.15, self.palette.dim_screen.clone()),
            ] {
                self.block(
                    "Indicador do módulo",
                    Vec3::new(33.556, 1.89, z + band),
                    Vec3::new(0.056, 0.085, 0.093),
                    mat,
                    false,
                );
            }
        }
        self.round(
            "Barramento de alimentação",
            Vec3::new(34.37, 3.28, -64.65),
            Vec3::new(34.37, 3.28, -49.18),
            0.24,
            self.palette.rust.clone(),
        );
        self.round(
            "Cabo do quadro",
            PANEL + Vec3::Y * 0.36,
            PANEL + Vec3::Y * 1.96,
            0.095,
            self.palette.coal.clone(),
        );
        self.main_electrical_panel();
        self.fan(Vec3::new(34.68, 2.65, -56.68), -FRAC_PI_2, 1.54);
        let memo = self.block(
            "Instrução de retorno grampeada",
            super::RETURN_MEMO,
            Vec3::new(0.026, 0.31, 0.40),
            self.palette.paper.clone(),
            false,
        );
        self.b.commands.entity(memo).insert(Interactable {
            bounds: Aabb::from_center_size(super::RETURN_MEMO, Vec3::new(0.14, 0.38, 0.47)),
            action: Action::ReadReturnMemo,
        });
        self.sign(
            "POSTO DE RETORNO",
            super::RETURN_MEMO + Vec3::Y * 0.33,
            Vec2::new(1.35, 0.24),
            -FRAC_PI_2,
        );
        self.floor_directions("NAVE ↑", Vec2::new(29.63, -56.37), PI);
        self.floor_directions("BOMBAS ↑", Vec2::new(18.80, -52.48), FRAC_PI_2);
        self.sign(
            "QUADRO GERAL",
            Vec3::new(34.48, 2.13, PANEL.z),
            Vec2::new(1.46, 0.30),
            -std::f32::consts::FRAC_PI_2,
        );
        self.block(
            "Faixa de segurança do quadro",
            Vec3::new(32.19, 0.006, PANEL.z),
            Vec3::new(0.086, 0.009, 3.1),
            self.palette.yellow.clone(),
            false,
        );
    }

    fn basin(&mut self) {
        let centre = (WATER_MAX + WATER_MIN) * 0.5;
        let size = WATER_MAX - WATER_MIN;
        self.block(
            "Água parada da bacia",
            Vec3::new(centre.x, -0.53, centre.y),
            Vec3::new(size.x, 0.026, size.y),
            self.palette.water.clone(),
            false,
        );
        for x in [WATER_MIN.x, WATER_MAX.x] {
            self.basin_rail(Vec3::new(x, 0.0, centre.y), Vec3::Z, size.y, 10);
            self.round(
                "Dreno suspenso da bacia",
                Vec3::new(x * 0.77, 4.33, -85.29),
                Vec3::new(x * 0.77, 4.33, -60.1),
                0.49,
                self.palette.oxide.clone(),
            );
        }
        for z in [WATER_MIN.y, WATER_MAX.y] {
            self.basin_rail(Vec3::new(centre.x, 0.0, z), Vec3::X, size.x, 7);
        }
        for (x, z, tint, lumens) in [
            (-7.2, -61.8, SODIUM, 26_000.0),
            (3.7, -67.5, AQUA, 31_000.0),
            (-4.7, -75.0, AQUA, 34_000.0),
            (8.0, -77.9, COLD, 25_000.0),
            (0.0, -83.65, EMERGENCY, 30_000.0),
        ] {
            let circuit = (z < -83.0).then_some(Circuit::Exit);
            self.light(Vec3::new(x, 4.92, z), tint, lumens, 8.0, circuit);
        }
        self.spill(Vec3::new(-3.5, 1.25, -70.6), AQUA, 11_900.0, 7.0);
        self.spill(Vec3::new(3.23, 1.12, -77.8), AQUA, 8200.0, 6.6);
        for side in [-1.0, 1.0] {
            self.spill(
                Vec3::new(side * 6.65, 2.28, -84.58),
                Color::srgb(0.69, 0.83, 0.87),
                27_800.0,
                5.6,
            );
            self.light(
                Vec3::new(side * 7.64, 4.88, -82.44),
                COLD,
                15_800.0,
                6.62,
                Some(Circuit::Exit),
            );
            let support = Vec3::new(side * 5.47, 0.0, -79.93);
            self.round(
                "Adutora da travessia",
                support + Vec3::Y * 0.09,
                support + Vec3::Y * 4.36,
                0.41,
                self.palette.oxide.clone(),
            );
            self.collision(support + Vec3::Y * 2.16, Vec3::new(0.37, 4.32, 0.37));
            for y in [0.32, 2.15, 3.73] {
                self.round(
                    "União da adutora da travessia",
                    support + Vec3::Y * (y - 0.075),
                    support + Vec3::Y * (y + 0.075),
                    0.51,
                    self.palette.steel.clone(),
                );
            }
        }
        for (x, z) in [(-3.05, -78.48), (2.98, -78.65)] {
            let body = Vec3::new(x, 0.0, z);
            self.round(
                "Queda da adutora",
                body + Vec3::Y * 4.33,
                body + Vec3::Y * 0.26,
                0.61,
                self.palette.oxide.clone(),
            );
            self.round(
                "Boca de descarga",
                body + Vec3::Y * 0.36,
                body + Vec3::Y * 0.14,
                0.80,
                self.palette.steel.clone(),
            );
        }
        for x in [-6.15, 6.15] {
            self.block(
                "Guia da passarela seca",
                Vec3::new(x, 0.005, -72.0),
                Vec3::new(0.12, 0.009, 16.1),
                self.palette.yellow.clone(),
                false,
            );
            self.floor_directions("SAÍDA ↑", Vec2::new(x * 1.24, -66.26), 0.0);
            self.floor_directions(
                if x > 0.0 { "SAÍDA ←" } else { "SAÍDA →" },
                Vec2::new(x * 1.24, -81.37),
                0.0,
            );
        }
        self.hanging_sign("SAÍDA ↑", Vec3::new(0.0, 3.51, -80.81), 2.10, 5.17);
        self.hanging_tag(Vec3::new(3.37, 4.31, -70.52), 1.19, 0.26);
    }

    fn basin_rail(&mut self, center: Vec3, direction: Vec3, length: f32, posts: u32) {
        let lengthwise = direction.abs() * (length - 0.19);
        let section =
            |thickness: f32, height: f32| Vec3::new(thickness, height, thickness) + lengthwise;
        self.block(
            "Mureta da bacia",
            center - Vec3::Y * 0.28,
            section(0.32, 0.84),
            self.palette.concrete.clone(),
            true,
        );
        self.collision(center + Vec3::Y * 0.56, section(0.14, 1.12));
        self.block(
            "Corrimão amarelo",
            center + Vec3::Y * 1.095,
            section(0.12, 0.092),
            self.palette.yellow.clone(),
            false,
        );
        self.block(
            "Travessa de guarda",
            center + Vec3::Y * 0.62,
            section(0.095, 0.065),
            self.palette.coal.clone(),
            false,
        );
        for index in 0..=posts {
            let local = -length * 0.5 + 0.09 + index as f32 / posts as f32 * (length - 0.18);
            self.block(
                "Coluna do guarda-corpo",
                center + direction * local + Vec3::Y * 0.63,
                Vec3::new(0.11, 0.95, 0.11),
                self.palette.oxide.clone(),
                false,
            );
        }
    }

    fn exit(&mut self) {
        let gate_size = Vec3::new(4.48, 2.96, 0.25);
        let gate = self.block(
            "Comporta da saída",
            EXIT_GATE,
            gate_size,
            self.palette.coal.clone(),
            false,
        );
        self.b.commands.entity(gate).insert((
            SceneRole::Gate,
            Interactable {
                bounds: Aabb::from_center_size(EXIT_GATE, Vec3::new(3.90, 2.60, 0.38)),
                action: Action::CheckGate,
            },
        ));
        self.b.solids.push(Solid {
            bounds: Aabb::from_center_size(EXIT_GATE, gate_size),
            gate: true,
        });
        for i in 0..16 {
            let bar = self.block(
                "Nervura da comporta",
                Vec3::new(0.0, 0.12 + i as f32 * 0.174, EXIT_GATE.z + 0.143),
                Vec3::new(4.35, 0.035, 0.05),
                self.palette.steel.clone(),
                false,
            );
            self.b.commands.entity(bar).insert(SceneRole::Gate);
        }
        for z in [-89.55, -95.05] {
            self.light(
                Vec3::new(0.0, 3.42, z),
                EMERGENCY,
                26_000.0,
                7.1,
                Some(Circuit::Exit),
            );
        }
        self.block(
            "Porta de serviço",
            Vec3::new(0.0, 1.25, -98.61),
            Vec3::new(2.67, 2.50, 0.21),
            self.palette.blue.clone(),
            false,
        );
        self.block(
            "Barra antipânico",
            Vec3::new(0.0, 1.01, -98.42),
            Vec3::new(2.12, 0.078, 0.15),
            self.palette.steel.clone(),
            false,
        );
        self.sign(
            "SAÍDA",
            Vec3::new(0.0, 2.76, -98.68),
            Vec2::new(1.14, 0.29),
            0.0,
        );
        for lane in [-1.0, 1.0] {
            self.block(
                "Guia de evacuação",
                Vec3::new(lane * 1.13, 0.006, -92.45),
                Vec3::new(0.086, 0.012, 11.67),
                self.palette.yellow.clone(),
                false,
            );
        }
    }

    fn handwheel(&mut self, center: Vec3, sideways: bool, scale: f32) {
        let rotation = if sideways {
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
        } else {
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
        };
        self.b.commands.spawn((
            Name::new("Volante da válvula"),
            Mesh3d(self.valve.clone()),
            MeshMaterial3d(self.palette.rust.clone()),
            Transform::from_translation(center)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(scale)),
        ));
        let horizontal = if sideways { Vec3::Z } else { Vec3::X };
        for along in [horizontal, Vec3::Y] {
            self.round(
                "Raio da válvula",
                center - along * (scale * 0.4),
                center + along * (scale * 0.4),
                0.055 * scale,
                self.palette.rust.clone(),
            );
        }
        let stem = if sideways { Vec3::X } else { Vec3::Z };
        self.round(
            "Eixo da válvula",
            center - stem * (scale * 0.26),
            center + stem * (scale * 0.11),
            0.16 * scale,
            self.palette.steel.clone(),
        );
    }

    fn hanging_sign(&mut self, title: &'static str, center: Vec3, width: f32, ceiling: f32) {
        self.block(
            "Painel de sinalização suspenso",
            center,
            Vec3::new(width, 0.46, 0.108),
            self.palette.coal.clone(),
            false,
        );
        for side in [-1.0, 1.0] {
            let tip = center + Vec3::new(width * 0.43 * side, 0.18, 0.0);
            self.round(
                "Fixação da sinalização",
                tip,
                tip.with_y(ceiling),
                0.042,
                self.palette.coal.clone(),
            );
        }
        self.sign(
            title,
            center + Vec3::Z * 0.060,
            Vec2::new(width * 0.95, 0.38),
            0.0,
        );
    }
}
