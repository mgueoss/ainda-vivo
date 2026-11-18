//! Reusable machine housings, fasteners, controls and instrumentation.
use super::{Mat, PANEL, SceneRole, Station, motion::FixtureMotion, surfaces};
use crate::interaction::Interactable;
use ainda_vivo_core::{collision::Aabb, story::Action};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use std::f32::consts::FRAC_PI_2;

pub(super) struct DetailKit {
    pub rounded: Handle<Mesh>,
    hex: Handle<Mesh>,
    hardware: Mat,
    red: Mat,
    dial: Mat,
}

impl DetailKit {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        Self {
            rounded: meshes.add(chamfer_box()),
            hex: meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(6)),
            hardware: materials.add(StandardMaterial {
                base_color: Color::srgb(0.62, 0.65, 0.61),
                metallic: 0.58,
                perceptual_roughness: 0.43,
                ..default()
            }),
            red: materials.add(StandardMaterial {
                base_color: Color::srgb(0.62, 0.185, 0.125),
                perceptual_roughness: 0.48,
                ..default()
            }),
            dial: materials.add(StandardMaterial {
                base_color_texture: Some(images.add(dial_image())),
                alpha_mode: AlphaMode::Mask(0.5),
                perceptual_roughness: 0.59,
                ..default()
            }),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Face {
    origin: Vec3,
    pub rotation: Quat,
}

impl Face {
    pub fn new(origin: Vec3, yaw: f32) -> Self {
        Self {
            origin,
            rotation: Quat::from_rotation_y(yaw),
        }
    }

    pub fn point(self, offset: Vec3) -> Vec3 {
        self.origin + self.rotation * offset
    }

    pub fn pose(self, offset: Vec3, scale: Vec3) -> Transform {
        Transform::from_translation(self.point(offset))
            .with_rotation(self.rotation)
            .with_scale(scale)
    }
}

impl Station<'_, '_, '_, '_> {
    pub(super) fn part(
        &mut self,
        name: &str,
        mesh: Handle<Mesh>,
        material: Mat,
        pose: Transform,
    ) -> Entity {
        self.b
            .commands
            .spawn((
                Name::new(name.to_owned()),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                pose,
            ))
            .id()
    }

    pub(super) fn rounded(&mut self, name: &str, material: Mat, pose: Transform) -> Entity {
        self.part(name, self.detail.rounded.clone(), material, pose)
    }

    pub(super) fn fastener(&mut self, face: Face, point: Vec3, width: f32) {
        let pose = face
            .pose(point, Vec3::new(width, width * 0.30, width))
            .with_rotation(face.rotation * Quat::from_rotation_x(FRAC_PI_2));
        self.part(
            "Parafuso sextavado",
            self.detail.hex.clone(),
            self.detail.hardware.clone(),
            pose,
        );
    }

    fn button(&mut self, face: Face, point: Vec3, diameter: f32, material: Mat) -> Entity {
        let axis = face.rotation * Vec3::Z;
        let start = face.point(point);
        self.round(
            "Aro torneado do botão",
            start,
            start + axis * 0.020,
            diameter,
            self.detail.hardware.clone(),
        );
        self.round(
            "Anel de vedação",
            start + axis * 0.019,
            start + axis * 0.029,
            diameter * 0.84,
            self.palette.coal.clone(),
        );
        let pose = face
            .pose(
                point + Vec3::Z * 0.031,
                Vec3::new(diameter * 0.70, 0.020, diameter * 0.70),
            )
            .with_rotation(face.rotation * Quat::from_rotation_x(FRAC_PI_2));
        self.part(
            "Lente ou tecla do comando",
            self.cylinder.clone(),
            material,
            pose,
        )
    }

    pub(super) fn meter(&mut self, face: Face, point: Vec3, diameter: f32, angle: f32) {
        let axis = face.rotation * Vec3::Z;
        let start = face.point(point);
        self.round(
            "Corpo do manômetro",
            start,
            start + axis * 0.052,
            diameter,
            self.detail.hardware.clone(),
        );
        self.round(
            "Vedação do mostrador",
            start + axis * 0.051,
            start + axis * 0.056,
            diameter * 0.89,
            self.palette.coal.clone(),
        );
        self.part(
            "Escala do manômetro",
            self.plate.clone(),
            self.detail.dial.clone(),
            face.pose(
                point + Vec3::Z * 0.057,
                Vec3::new(diameter * 0.83, diameter * 0.83, 1.0),
            ),
        );
        let needle = Quat::from_rotation_z(angle);
        let center = point + Vec3::Z * 0.061 + needle * Vec3::Y * (diameter * 0.14);
        let needle_pose = face
            .pose(center, Vec3::new(diameter * 0.035, diameter * 0.47, 0.004))
            .with_rotation(face.rotation * needle);
        self.rounded(
            "Ponteiro do manômetro",
            self.detail.red.clone(),
            needle_pose,
        );
        self.round(
            "Eixo do ponteiro",
            start + axis * 0.059,
            start + axis * 0.069,
            diameter * 0.099,
            self.palette.coal.clone(),
        );
    }

    pub(super) fn main_electrical_panel(&mut self) {
        let yaw = -FRAC_PI_2;
        let face = Face::new(PANEL, yaw);
        self.collision(PANEL, Vec3::new(0.32, 0.72, 0.78));
        let cabinet = self.rounded(
            "Caixa do quadro geral",
            self.palette.oxide.clone(),
            face.pose(Vec3::ZERO, Vec3::new(0.78, 0.72, 0.32)),
        );
        self.b.commands.entity(cabinet).insert(Interactable {
            bounds: Aabb::from_center_size(PANEL, Vec3::new(0.38, 0.81, 0.84)),
            action: Action::RestorePower,
            label: "restaurar energia",
        });
        self.rounded(
            "Tampa chanfrada do quadro",
            self.palette.steel.clone(),
            face.pose(Vec3::Z * 0.164, Vec3::new(0.735, 0.672, 0.074)),
        );
        self.rounded(
            "Placa interna de comandos",
            self.palette.coal.clone(),
            face.pose(Vec3::Z * 0.204, Vec3::new(0.649, 0.592, 0.019)),
        );
        for x in [-0.306, 0.306] {
            for y in [-0.254, 0.254] {
                self.fastener(face, Vec3::new(x, y, 0.22), 0.039);
            }
        }
        self.sign(
            "QG 07",
            face.point(Vec3::new(-0.111, 0.239, 0.218)),
            Vec2::new(0.254, 0.062),
            yaw,
        );
        self.meter(face, Vec3::new(-0.140, 0.055, 0.215), 0.258, -0.74);
        self.button(
            face,
            Vec3::new(0.128, 0.196, 0.216),
            0.080,
            self.palette.amber_led.clone(),
        );
        self.button(
            face,
            Vec3::new(0.252, 0.196, 0.216),
            0.080,
            self.palette.dim_screen.clone(),
        );
        let ready = self.part(
            "Indicador de saída energizada",
            self.cylinder.clone(),
            self.palette.green_led.clone(),
            face.pose(
                Vec3::new(0.252, 0.196, 0.260),
                Vec3::new(0.056, 0.010, 0.056),
            )
            .with_rotation(face.rotation * Quat::from_rotation_x(FRAC_PI_2)),
        );
        self.b
            .commands
            .entity(ready)
            .insert((SceneRole::PowerLamp, Visibility::Hidden));
        self.sign(
            "REDE",
            face.point(Vec3::new(0.128, 0.128, 0.218)),
            Vec2::new(0.091, 0.035),
            yaw,
        );
        self.sign(
            "SAÍDA",
            face.point(Vec3::new(0.252, 0.128, 0.218)),
            Vec2::new(0.091, 0.035),
            yaw,
        );
        self.button(
            face,
            Vec3::new(-0.223, -0.161, 0.213),
            0.069,
            self.detail.red.clone(),
        );
        self.rounded(
            "Porta-fusível do quadro",
            self.palette.steel.clone(),
            face.pose(
                Vec3::new(-0.088, -0.163, 0.239),
                Vec3::new(0.133, 0.073, 0.048),
            ),
        );
        self.sign(
            "FUSÍVEL",
            face.point(Vec3::new(-0.083, -0.229, 0.219)),
            Vec2::new(0.27, 0.042),
            yaw,
        );
        self.rounded(
            "Berço do disjuntor",
            self.palette.steel.clone(),
            face.pose(
                Vec3::new(0.191, -0.081, 0.234),
                Vec3::new(0.180, 0.234, 0.077),
            ),
        );
        self.rounded(
            "Fenda do disjuntor",
            self.palette.coal.clone(),
            face.pose(
                Vec3::new(0.191, -0.078, 0.278),
                Vec3::new(0.092, 0.170, 0.025),
            ),
        );
        let lever = self.detail.rounded.clone();
        let dark = self.palette.coal.clone();
        let cap = self.palette.yellow.clone();
        self.b
            .commands
            .spawn((
                Name::new("Alavanca do disjuntor"),
                face.pose(Vec3::new(0.191, -0.081, 0.284), Vec3::ONE),
                Visibility::Inherited,
            ))
            .with_children(|pivot| {
                pivot
                    .spawn((
                        Transform::default(),
                        FixtureMotion::Breaker,
                        Visibility::Inherited,
                    ))
                    .with_children(|lever_arm| {
                        lever_arm.spawn((
                            Mesh3d(lever.clone()),
                            MeshMaterial3d(dark),
                            Transform::from_xyz(0.0, 0.0, 0.038)
                                .with_scale(Vec3::new(0.064, 0.037, 0.099)),
                        ));
                        lever_arm.spawn((
                            Mesh3d(lever),
                            MeshMaterial3d(cap),
                            Transform::from_xyz(0.0, 0.0, 0.088)
                                .with_scale(Vec3::new(0.120, 0.061, 0.039)),
                        ));
                    });
            });
        for (offset, end) in [(-0.229, -0.83), (0.229, -0.62)] {
            self.round(
                "Cabo flexível do quadro",
                face.point(Vec3::new(offset, -0.32, 0.041)),
                face.point(Vec3::new(offset, end, 0.041)),
                0.036,
                self.palette.coal.clone(),
            );
            self.round(
                "Prensa-cabo do quadro",
                face.point(Vec3::new(offset, -0.394, 0.041)),
                face.point(Vec3::new(offset, -0.338, 0.041)),
                0.083,
                self.detail.hardware.clone(),
            );
        }
    }

    pub(super) fn local_controls(&mut self, face: Face) {
        self.rounded(
            "Caixa de comando da bomba",
            self.palette.blue.clone(),
            face.pose(Vec3::ZERO, Vec3::new(0.45, 0.38, 0.16)),
        );
        self.rounded(
            "Tampa da caixa de comando",
            self.palette.steel.clone(),
            face.pose(Vec3::Z * 0.081, Vec3::new(0.418, 0.349, 0.044)),
        );
        self.meter(face, Vec3::new(-0.058, 0.035, 0.105), 0.193, 0.73);
        self.button(
            face,
            Vec3::new(0.112, 0.065, 0.106),
            0.074,
            self.palette.amber_led.clone(),
        );
        self.button(
            face,
            Vec3::new(0.112, -0.056, 0.106),
            0.074,
            self.detail.red.clone(),
        );
        for x in [-0.168, 0.168] {
            for y in [-0.126, 0.126] {
                self.fastener(face, Vec3::new(x, y, 0.103), 0.028);
            }
        }
    }

    pub(super) fn terminal(&mut self, base: Vec3, live: bool) {
        let face = Face::new(base, 0.0);
        let signal = if live {
            self.palette.dim_screen.clone()
        } else {
            self.palette.coal.clone()
        };
        self.rounded(
            "Suporte do monitor",
            self.palette.steel.clone(),
            face.pose(Vec3::new(0.0, 0.945, -0.14), Vec3::new(0.38, 0.047, 0.29)),
        );
        self.rounded(
            "Gabinete do monitor",
            self.palette.oxide.clone(),
            face.pose(Vec3::new(0.0, 1.163, -0.15), Vec3::new(0.51, 0.402, 0.407)),
        );
        self.rounded(
            "Aro do monitor",
            self.palette.coal.clone(),
            face.pose(Vec3::new(0.0, 1.170, 0.052), Vec3::new(0.446, 0.322, 0.057)),
        );
        self.rounded(
            "Vidro do monitor",
            signal,
            face.pose(Vec3::new(0.0, 1.184, 0.078), Vec3::new(0.361, 0.241, 0.022)),
        );
        self.fastener(face, Vec3::new(-0.194, 1.037, 0.076), 0.018);
        self.fastener(face, Vec3::new(0.194, 1.037, 0.076), 0.018);
        self.rounded(
            "Base do teclado",
            self.palette.coal.clone(),
            face.pose(Vec3::new(0.0, 0.943, 0.300), Vec3::new(0.41, 0.037, 0.213)),
        );
        for row in 0..3 {
            for column in 0..8 {
                let key = if column == 7 {
                    self.palette.blue.clone()
                } else {
                    self.palette.pale.clone()
                };
                self.rounded(
                    "Tecla do terminal",
                    key,
                    face.pose(
                        Vec3::new(
                            -0.171 + column as f32 * 0.049,
                            0.964,
                            0.229 + row as f32 * 0.043,
                        ),
                        Vec3::new(0.043, 0.019, 0.037),
                    ),
                );
            }
        }
        self.rounded(
            "Barra de espaço",
            self.palette.steel.clone(),
            face.pose(
                Vec3::new(-0.027, 0.964, 0.367),
                Vec3::new(0.231, 0.019, 0.036),
            ),
        );
    }
}

fn dial_image() -> Image {
    let size = 108;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let uv = (Vec2::new(x as f32, y as f32) + 0.5) / size as f32;
            let pos = (uv - 0.5) * Vec2::new(2.0, -2.0);
            let radius = pos.length();
            let angle = pos.x.atan2(pos.y);
            let mark = (angle + 2.25) / 0.30;
            let tick = (mark - mark.round()).abs() < 0.083
                && (0.0..=15.0).contains(&mark.round())
                && (0.70..=0.90).contains(&radius);
            let rim = (0.93..1.0).contains(&radius);
            let wear = (surfaces::noise(uv, IVec2::new(28, 31), 337) * 21.0) as u8;
            let color: [u8; 3] = if tick || rim {
                if tick && angle > 1.36 {
                    [115, 36, 23]
                } else {
                    [46, 52, 49]
                }
            } else {
                [185, 178, 148]
            };
            data.extend(color.map(|channel| channel.saturating_sub(wear)));
            data.push(if radius < 1.0 { 255 } else { 0 });
        }
    }
    surfaces::texture(size, size, data, false)
}

/// A unit box with flat main faces, twelve bevels and eight corner facets.
fn chamfer_box() -> Mesh {
    const EDGE: f32 = 0.5;
    const INSET: f32 = 0.401;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uv = Vec::new();
    let mut indices = Vec::new();
    let mut face = |points: &[Vec3], outward: Vec3| {
        let start = positions.len() as u32;
        positions.extend(points.iter().map(|point| point.to_array()));
        normals.extend(std::iter::repeat_n(
            outward.normalize().to_array(),
            points.len(),
        ));
        uv.extend(
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
                .into_iter()
                .take(points.len()),
        );
        let facing = (points[1] - points[0])
            .cross(points[2] - points[0])
            .dot(outward)
            > 0.0;
        for corner in 1..(points.len() as u32 - 1) {
            let [a, b] = if facing {
                [corner, corner + 1]
            } else {
                [corner + 1, corner]
            };
            indices.extend([start, start + a, start + b]);
        }
    };
    for axis in 0..3 {
        for sign in [-1.0, 1.0] {
            let mut outward = Vec3::ZERO;
            outward[axis] = sign;
            let points = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(a, b)| {
                let mut point = outward * EDGE;
                point[(axis + 1) % 3] = a * INSET;
                point[(axis + 2) % 3] = b * INSET;
                point
            });
            face(&points, outward);
        }
    }
    for (a, b, c) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
        for sa in [-1.0, 1.0] {
            for sb in [-1.0, 1.0] {
                let mut outward = Vec3::ZERO;
                outward[a] = sa;
                outward[b] = sb;
                let points = [
                    (EDGE, INSET, -1.0),
                    (INSET, EDGE, -1.0),
                    (INSET, EDGE, 1.0),
                    (EDGE, INSET, 1.0),
                ]
                .map(|(width, height, along)| {
                    let mut point = Vec3::ZERO;
                    point[a] = sa * width;
                    point[b] = sb * height;
                    point[c] = along * INSET;
                    point
                });
                face(&points, outward);
            }
        }
    }
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let outward = Vec3::new(x, y, z);
                let points = [Vec3::X, Vec3::Y, Vec3::Z]
                    .map(|axis| outward * (Vec3::splat(INSET) + axis * (EDGE - INSET)));
                face(&points, outward);
            }
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_indices(Indices::U32(indices))
}
