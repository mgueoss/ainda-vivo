//! Small movements in the station, driven by the paused expedition clock.
use super::{Station, detail::Face};
use crate::{
    app::{Screen, Session},
    atmosphere::Atmosphere,
};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, TAU};

pub(super) const RESTING_NEEDLE: f32 = 1.96;

#[derive(Component)]
pub enum FixtureMotion {
    Fan {
        rate: f32,
        phase: f32,
    },
    Hanging {
        phase: f32,
    },
    Breaker,
    PressureNeedle {
        center: Vec3,
        rotation: Quat,
        offset: f32,
    },
}

pub fn animate(
    time: Res<Time>,
    session: Res<Session>,
    atmosphere: Res<Atmosphere>,
    mut mechanisms: Query<(&FixtureMotion, &mut Transform)>,
) {
    if session.screen != Screen::Playing {
        return;
    }
    let elapsed = session.elapsed;
    for (mechanism, mut pose) in &mut mechanisms {
        pose.rotation = match *mechanism {
            FixtureMotion::Fan { rate, phase } => {
                Quat::from_rotation_z((elapsed * rate + phase).rem_euclid(TAU))
            }
            FixtureMotion::Hanging { phase } => {
                let wind = 0.56 + 0.44 * (elapsed * 0.33 + phase).sin().powi(2);
                let side = (elapsed * 1.72 + phase).sin() * 0.23 * wind;
                let depth = (elapsed * 1.27 + phase * 1.3).cos() * 0.10 * wind;
                Quat::from_rotation_z(side) * Quat::from_rotation_x(depth)
            }
            FixtureMotion::Breaker => {
                let angle = if session.progress.power_restored() {
                    -0.52
                } else {
                    0.48
                };
                pose.rotation.slerp(
                    Quat::from_rotation_x(angle),
                    1.0 - (-18.0 * time.delta_secs()).exp(),
                )
            }
            FixtureMotion::PressureNeedle {
                center,
                rotation,
                offset,
            } => {
                let angle = RESTING_NEEDLE - 3.99 * atmosphere.episode.pressure;
                let pointer = rotation * Quat::from_rotation_z(angle);
                pose.translation = center + pointer * Vec3::Y * offset;
                pointer
            }
        };
    }
}

impl Station<'_, '_, '_, '_> {
    pub(super) fn dispatch_mechanisms(&mut self) {
        // Illuminated enamel behind the first silhouette, also a readable route landmark.
        let face = Face::new(Vec3::new(3.36, 1.42, -22.46), 0.0);
        let body = Vec3::new(2.15, 2.45, 0.31);
        self.collision(
            face.point(Vec3::new(0.0, -0.20, 0.0)),
            Vec3::new(body.x, 2.77, 0.48),
        );
        self.rounded(
            "Armário de distribuição da triagem",
            self.palette.oxide.clone(),
            face.pose(Vec3::ZERO, body),
        );
        self.rounded(
            "Chapa de esmalte gasta",
            self.palette.pale.clone(),
            face.pose(Vec3::Z * 0.159, Vec3::new(1.96, 2.20, 0.071)),
        );
        for x in [-0.82, 0.82] {
            for y in [-0.96, 0.96] {
                self.fastener(face, Vec3::new(x, y, 0.197), 0.069);
            }
            self.rounded(
                "Sapata da distribuição",
                self.palette.steel.clone(),
                face.pose(Vec3::new(x, -1.32, 0.0), Vec3::new(0.21, 0.176, 0.43)),
            );
        }
        self.sign(
            "DISTRIBUIÇÃO",
            face.point(Vec3::new(0.015, 1.005, 0.198)),
            Vec2::new(0.98, 0.152),
            0.0,
        );
        self.spill(
            Vec3::new(3.49, 2.42, -21.56),
            Color::srgb(0.91, 0.85, 0.70),
            4600.0,
            4.15,
        );
        self.hanging_tag(Vec3::new(-1.54, 3.56, -20.9), 0.85, 0.72);
        self.fan(Vec3::new(9.71, 2.43, -21.2), -FRAC_PI_2, 1.2);
    }

    pub(super) fn fan(&mut self, origin: Vec3, yaw: f32, phase: f32) {
        let face = Face::new(origin, yaw);
        self.rounded(
            "Moldura do exaustor",
            self.palette.oxide.clone(),
            face.pose(Vec3::ZERO, Vec3::new(1.18, 1.16, 0.26)),
        );
        let axis = face.rotation * Vec3::Z;
        let start = face.point(Vec3::Z * 0.131);
        self.round(
            "Duto circular do exaustor",
            start,
            start + axis * 0.012,
            1.0,
            self.palette.coal.clone(),
        );
        self.part(
            "Aro do duto",
            self.valve.clone(),
            self.palette.steel.clone(),
            face.pose(Vec3::Z * 0.169, Vec3::splat(1.14))
                .with_rotation(face.rotation * Quat::from_rotation_x(FRAC_PI_2)),
        );
        for x in [-0.47, 0.47] {
            for y in [-0.47, 0.47] {
                self.fastener(face, Vec3::new(x, y, 0.148), 0.060);
            }
        }
        let blade = self.detail.rounded.clone();
        let blade_mat = self.palette.blue.clone();
        let cylinder = self.cylinder.clone();
        let hub = self.palette.steel.clone();
        self.b
            .commands
            .spawn((
                Name::new("Eixo do exaustor"),
                face.pose(Vec3::Z * 0.183, Vec3::ONE),
                Visibility::Inherited,
            ))
            .with_children(|assembly| {
                assembly
                    .spawn((
                        Name::new("Rotor do exaustor"),
                        Transform::default(),
                        Visibility::Inherited,
                        FixtureMotion::Fan {
                            rate: 1.22 + phase * 0.23,
                            phase,
                        },
                    ))
                    .with_children(|rotor| {
                        for index in 0..5 {
                            let turn = Quat::from_rotation_z(index as f32 * TAU / 5.0);
                            let shape =
                                Transform::from_translation(turn * Vec3::new(0.045, 0.259, 0.0))
                                    .with_rotation(
                                        turn * Quat::from_rotation_z(-0.24)
                                            * Quat::from_rotation_y(0.25),
                                    )
                                    .with_scale(Vec3::new(0.145, 0.31, 0.027));
                            rotor.spawn((
                                Mesh3d(blade.clone()),
                                MeshMaterial3d(blade_mat.clone()),
                                shape,
                            ));
                        }
                        rotor.spawn((
                            Mesh3d(cylinder),
                            MeshMaterial3d(hub),
                            Transform::from_xyz(0.0, 0.0, 0.025)
                                .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                                .with_scale(Vec3::new(0.21, 0.070, 0.21)),
                        ));
                    });
            });
        for x in [-0.34, -0.17, 0.0, 0.17, 0.34] {
            let length = 2.0 * (0.47_f32.powi(2) - x * x).sqrt();
            self.rounded(
                "Grade do exaustor",
                self.palette.steel.clone(),
                face.pose(Vec3::new(x, 0.0, 0.265), Vec3::new(0.021, length, 0.027)),
            );
        }
    }

    pub(super) fn hanging_tag(&mut self, anchor: Vec3, span: f32, phase: f32) {
        self.round(
            "Pino do pendente",
            anchor,
            anchor + Vec3::Y * 0.225,
            0.056,
            self.palette.steel.clone(),
        );
        let link = self.valve.clone();
        let chain = self.palette.steel.clone();
        let mesh = self.detail.rounded.clone();
        let metal = self.palette.oxide.clone();
        let faded = self.palette.yellow.clone();
        self.b
            .commands
            .spawn((
                Name::new("Pendente de manutenção"),
                Transform::from_translation(anchor),
                FixtureMotion::Hanging { phase },
                Visibility::Inherited,
            ))
            .with_children(|pendant| {
                let links = (span / 0.11).ceil() as u32;
                for index in 0..links {
                    let turn = if index.is_multiple_of(2) {
                        0.0
                    } else {
                        FRAC_PI_2
                    };
                    let pose =
                        Transform::from_xyz(0.0, -(index as f32 + 0.43) * span / links as f32, 0.0)
                            .with_rotation(
                                Quat::from_rotation_y(turn) * Quat::from_rotation_x(FRAC_PI_2),
                            )
                            .with_scale(Vec3::new(0.117, 0.082, 0.169));
                    pendant.spawn((Mesh3d(link.clone()), MeshMaterial3d(chain.clone()), pose));
                }
                let plate = Transform::from_xyz(0.0, -span - 0.142, 0.0)
                    .with_scale(Vec3::new(0.386, 0.289, 0.031));
                pendant.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(metal), plate));
                for side in [-1.0, 1.0] {
                    let stripe = Transform::from_xyz(0.0, -span - 0.190, side * 0.017)
                        .with_rotation(Quat::from_rotation_z(0.20))
                        .with_scale(Vec3::new(0.282, 0.062, 0.008));
                    pendant.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(faded.clone()), stripe));
                }
            });
    }
}
