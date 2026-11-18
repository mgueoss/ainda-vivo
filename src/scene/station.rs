//! The floor plan is shared by the visible shell, collision and route checks.
mod detail;
mod machines;
mod motion;
mod plan;
mod workspaces;

use super::{Builder, SceneRole, surfaces};
use crate::{
    app::{CollisionWorld, Solid},
    atmosphere::{Circuit, CircuitLight},
};
use bevy::prelude::*;
use std::{collections::HashMap, f32::consts::FRAC_PI_2};

pub use motion::{FixtureMotion, animate};
pub use plan::{EXIT_GATE, PANEL};
use plan::{PORTALS, ROOMS, Room, Sector, Side, WATER_MAX, WATER_MIN, get};

const WALL: f32 = 0.24;
const FLOOR_DEPTH: f32 = 0.38;

pub fn reached_exit(feet: Vec3) -> bool {
    (get(Sector::Exit).min.y..plan::EXIT_THRESHOLD).contains(&feet.z) && feet.x.abs() < 3.0
}

pub(super) struct Shell {
    pub wall: Handle<StandardMaterial>,
    pub tile: Handle<StandardMaterial>,
    pub ceiling: Handle<StandardMaterial>,
}

pub(super) fn setup(
    b: &mut Builder,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    shell: Shell,
) {
    let cylinder = meshes.add(Cylinder::new(0.5, 1.0));
    let valve = meshes.add(Torus::new(0.31, 0.46));
    let plate = meshes.add(Rectangle::new(1.0, 1.0));
    let palette = Palette::new(materials, images);
    let detail = detail::DetailKit::new(meshes, materials, images);
    let mut kit = Station {
        b,
        meshes,
        materials,
        images,
        shell,
        palette,
        cylinder,
        valve,
        plate,
        detail,
        signs: HashMap::new(),
    };
    for &room in ROOMS {
        kit.build_room(room);
        match room.sector {
            Sector::Access => (),
            Sector::Dispatch
            | Sector::Dock
            | Sector::Stores
            | Sector::Records
            | Sector::Lockers
            | Sector::Inspection
            | Sector::Control => kit.furnish(room.sector),
            Sector::Pumps
            | Sector::Filters
            | Sector::Generators
            | Sector::Tanks
            | Sector::Electrical
            | Sector::Basin
            | Sector::Exit => kit.install(room.sector),
        }
    }
    let area: f32 = ROOMS
        .iter()
        .map(|room| {
            let size = room.max - room.min;
            size.x * size.y
        })
        .sum();
    info!(
        "Estação 07: {} setores, {} ligações, {:.0} m² de planta.",
        ROOMS.len(),
        PORTALS.len(),
        area
    );
}

/// Check the authored connections against the physical scene, including furnishings.
pub fn audit_routes(world: &CollisionWorld) -> Result<(usize, usize), String> {
    use ainda_vivo_core::{
        collision::{body_bounds, move_and_slide},
        movement::{SPAWN, STANDING_HEIGHT},
    };
    use std::collections::VecDeque;
    let open = world.active(true);
    for portal in PORTALS {
        let [a, b] = portal.rooms.map(get);
        let side = Side::ALL.into_iter().find(|side| side.contains(a, *portal));
        if Side::ALL
            .into_iter()
            .filter(|side| side.contains(b, *portal))
            .count()
            != 1
            || side.is_none()
        {
            return Err(format!("Porta fora do encontro de {:?}", portal.rooms));
        }
        let direction = side.expect("opening verified").inward();
        let doorway = Vec3::new(portal.at.x, 0.004, portal.at.y);
        for normal in [direction, -direction] {
            let crossed = move_and_slide(
                doorway + normal * 1.35,
                STANDING_HEIGHT,
                -normal * 2.7,
                1.0,
                &open,
            );
            if crossed.feet.distance(doorway - normal * 1.35) > 0.03 {
                return Err(format!("Passagem obstruída em {:?}", portal.rooms));
            }
        }
    }

    // Flood grid cells where a full standing body fits. A half-metre stride is
    // smaller than the body-expanded wall thickness, including at doorway edges.
    const STEP: f32 = 0.5;
    const WIDTH: usize = 136;
    const DEPTH: usize = 218;
    let minimum = Vec2::new(-33.0, -99.0);
    let cell = |index: usize| {
        minimum + Vec2::new((index % WIDTH) as f32 + 0.5, (index / WIDTH) as f32 + 0.5) * STEP
    };
    let mut room_indices = vec![None; WIDTH * DEPTH];
    for (index, room_index) in room_indices.iter_mut().enumerate() {
        let position = cell(index);
        if !(position.cmpge(WATER_MIN).all() && position.cmple(WATER_MAX).all()) {
            *room_index = ROOMS.iter().position(|room| room.contains(position));
        }
    }
    let spawn = ((SPAWN.xz() - minimum) / STEP).as_uvec2();
    let start = spawn.y as usize * WIDTH + spawn.x as usize;
    let filled = |power| {
        let bounds = world.active(power);
        let mut queue = VecDeque::from([start]);
        let mut seen = vec![false; WIDTH * DEPTH];
        let mut rooms = vec![false; ROOMS.len()];
        seen[start] = true;
        while let Some(index) = queue.pop_front() {
            let Some(room_index) = room_indices[index] else {
                continue;
            };
            let position = cell(index);
            let body = body_bounds(Vec3::new(position.x, 0.005, position.y), STANDING_HEIGHT);
            if bounds.iter().any(|bound| body.overlaps(*bound)) {
                continue;
            }
            rooms[room_index] = true;
            let (x, z) = (index % WIDTH, index / WIDTH);
            for neighbour in [
                (x > 0).then(|| index - 1),
                (x + 1 < WIDTH).then(|| index + 1),
                (z > 0).then(|| index - WIDTH),
                (z + 1 < DEPTH).then(|| index + WIDTH),
            ]
            .into_iter()
            .flatten()
            {
                if !seen[neighbour] {
                    seen[neighbour] = true;
                    queue.push_back(neighbour);
                }
            }
        }
        rooms
    };
    for (opened, seen) in [(false, filled(false)), (true, filled(true))] {
        for (room, reachable) in ROOMS.iter().zip(seen) {
            let expected = opened || room.sector != Sector::Exit;
            if reachable != expected {
                return Err(format!(
                    "Conectividade incorreta em {} com saída {}",
                    room.title,
                    if opened { "aberta" } else { "fechada" }
                ));
            }
        }
    }
    Ok((ROOMS.len(), PORTALS.len()))
}

type Mat = Handle<StandardMaterial>;

struct Palette {
    concrete: Mat,
    oxide: Mat,
    water: Mat,
    coal: Mat,
    steel: Mat,
    pale: Mat,
    yellow: Mat,
    rust: Mat,
    blue: Mat,
    paper: Mat,
    wood: Mat,
    green_led: Mat,
    amber_led: Mat,
    dim_screen: Mat,
}

impl Palette {
    fn new(materials: &mut Assets<StandardMaterial>, images: &mut Assets<Image>) -> Self {
        let concrete = surfaces::material(surfaces::Surface::Concrete, images, materials);
        let oxide = surfaces::material(surfaces::Surface::Oxide, images, materials);
        let water = surfaces::material(surfaces::Surface::Water, images, materials);
        let mut matte = |r, g, b| {
            materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
                perceptual_roughness: 0.83,
                ..default()
            })
        };
        Self {
            concrete,
            oxide,
            water,
            coal: matte(0.115, 0.145, 0.142),
            steel: matte(0.365, 0.412, 0.392),
            pale: matte(0.55, 0.57, 0.499),
            yellow: matte(0.70, 0.50, 0.189),
            rust: matte(0.44, 0.236, 0.157),
            blue: matte(0.21, 0.34, 0.38),
            paper: matte(0.67, 0.626, 0.46),
            wood: matte(0.386, 0.287, 0.196),
            green_led: materials.add(emissive(Color::srgb(0.44, 0.86, 0.57), 1.9)),
            amber_led: materials.add(emissive(Color::srgb(0.95, 0.57, 0.20), 1.05)),
            dim_screen: materials.add(emissive(Color::srgb(0.21, 0.40, 0.40), 0.30)),
        }
    }
}

fn emissive(color: Color, gain: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        emissive: color.to_linear() * gain,
        perceptual_roughness: 0.71,
        ..default()
    }
}

struct Station<'a, 'builder, 'w, 's> {
    b: &'a mut Builder<'builder, 'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    images: &'a mut Assets<Image>,
    shell: Shell,
    palette: Palette,
    cylinder: Handle<Mesh>,
    valve: Handle<Mesh>,
    plate: Handle<Mesh>,
    detail: detail::DetailKit,
    signs: HashMap<&'static str, Mat>,
}

impl Station<'_, '_, '_, '_> {
    fn build_room(&mut self, room: Room) {
        let tiled = matches!(
            room.sector,
            Sector::Access | Sector::Dispatch | Sector::Records | Sector::Lockers | Sector::Control
        );
        let floor = if tiled {
            self.shell.tile.clone()
        } else {
            self.palette.concrete.clone()
        };
        if room.sector == Sector::Basin {
            let left_max = Vec2::new(WATER_MIN.x, room.max.y);
            let right_min = Vec2::new(WATER_MAX.x, room.min.y);
            for (min, max) in [
                (room.min, left_max),
                (right_min, room.max),
                (
                    Vec2::new(WATER_MIN.x, room.min.y),
                    Vec2::new(WATER_MAX.x, WATER_MIN.y),
                ),
                (
                    Vec2::new(WATER_MIN.x, WATER_MAX.y),
                    Vec2::new(WATER_MAX.x, room.max.y),
                ),
            ] {
                self.slab(
                    "Piso em torno da água",
                    min,
                    max,
                    -FLOOR_DEPTH * 0.5,
                    floor.clone(),
                );
            }
            self.slab(
                "Fundo de concreto da bacia",
                WATER_MIN,
                WATER_MAX,
                -1.02,
                floor,
            );
        } else {
            self.slab(
                "Piso da estação",
                room.min,
                room.max,
                -FLOOR_DEPTH * 0.5,
                floor,
            );
        }
        self.slab(
            "Teto da estação",
            room.min,
            room.max,
            room.height + FLOOR_DEPTH * 0.5,
            self.shell.ceiling.clone(),
        );

        for side in Side::ALL {
            let end = side.along(room.max);
            let mut start = side.along(room.min);
            let mut gaps = PORTALS
                .iter()
                .filter_map(|&portal| {
                    portal
                        .neighbour(room.sector)
                        .filter(|_| side.contains(room, portal))
                        .map(|other| (portal, other))
                })
                .collect::<Vec<_>>();
            gaps.sort_by(|(left, _), (right, _)| {
                side.along(left.at).total_cmp(&side.along(right.at))
            });
            for (portal, other) in gaps {
                let middle = side.along(portal.at);
                let left = middle - portal.width * 0.5;
                let right = middle + portal.width * 0.5;
                self.wall(room, side, start, left, 0.0, room.height);
                self.wall(room, side, left, right, portal.height, room.height);
                self.frame(
                    room,
                    side,
                    middle,
                    portal.width,
                    portal.height,
                    get(other).title,
                );
                start = right;
            }
            self.wall(room, side, start, end, 0.0, room.height);
        }
    }

    fn slab(&mut self, name: &str, min: Vec2, max: Vec2, y: f32, material: Mat) {
        let center = (min + max) * 0.5;
        let size = max - min;
        self.b.surface(
            self.meshes,
            name,
            Vec3::new(center.x, y, center.y),
            Vec3::new(size.x, FLOOR_DEPTH, size.y),
            material,
        );
    }

    fn wall(&mut self, room: Room, side: Side, start: f32, end: f32, low: f32, high: f32) {
        let width = end - start;
        let height = high - low;
        if width < 0.001 || height < 0.001 {
            return;
        }
        let along = (start + end) * 0.5;
        let inward = side.inward();
        let center = side.at(room, along, (low + high) * 0.5) + inward * (WALL * 0.5);
        self.b.surface(
            self.meshes,
            "Alvenaria da estação",
            center,
            side.size(width, height, WALL),
            self.shell.wall.clone(),
        );
        if low == 0.0 {
            let material = match room.sector {
                Sector::Electrical | Sector::Basin | Sector::Control => self.palette.blue.clone(),
                Sector::Dock | Sector::Generators => self.palette.rust.clone(),
                _ => self.palette.oxide.clone(),
            };
            let band = side.at(room, along, 0.475) + inward * (WALL + 0.012);
            self.b.block(
                "Revestimento inferior",
                band,
                side.size(width, 0.92, 0.024),
                material,
                false,
            );
            self.b.block(
                "Remate de metal",
                band + Vec3::Y * 0.477,
                side.size(width, 0.036, 0.045),
                self.palette.coal.clone(),
                false,
            );
        }
    }

    fn frame(
        &mut self,
        room: Room,
        side: Side,
        along: f32,
        width: f32,
        height: f32,
        title: &'static str,
    ) {
        let face = side.inward() * (WALL + 0.023);
        for edge in [-1.0, 1.0] {
            let upright = side.at(room, along + edge * (width * 0.5 + 0.025), height * 0.5) + face;
            self.b.block(
                "Perfil da passagem",
                upright,
                side.size(0.15, height, 0.06),
                self.palette.steel.clone(),
                false,
            );
        }
        let header = side.at(room, along, height + 0.045) + face;
        self.b.block(
            "Cabeceira da passagem",
            header,
            side.size(width + 0.20, 0.15, 0.066),
            self.palette.coal.clone(),
            false,
        );
        let sign = side.at(room, along, height + 0.175) + side.inward() * (WALL + 0.095);
        self.sign(
            title,
            sign,
            Vec2::new((width * 0.76).min(2.70), 0.27),
            side.rotation(),
        );
    }

    fn sign(&mut self, title: &'static str, position: Vec3, size: Vec2, yaw: f32) {
        let material = self.signs.entry(title).or_insert_with(|| {
            let (fill, ink) = if title == "SAÍDA" {
                ([18, 53, 44], [145, 199, 159])
            } else if title.contains("ELÉTRICA") {
                ([58, 44, 23], [211, 169, 96])
            } else {
                ([28, 38, 38], [151, 162, 144])
            };
            let image = self.images.add(surfaces::sign(title, fill, ink));
            self.materials.add(StandardMaterial {
                base_color_texture: Some(image),
                unlit: true,
                ..default()
            })
        });
        self.b.commands.spawn((
            Name::new(title),
            Mesh3d(self.plate.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(size.extend(1.0)),
        ));
    }

    fn block(
        &mut self,
        name: &str,
        position: Vec3,
        size: Vec3,
        material: Mat,
        solid: bool,
    ) -> Entity {
        self.b.block(name, position, size, material, solid)
    }

    fn round(&mut self, name: &str, start: Vec3, end: Vec3, diameter: f32, material: Mat) {
        let offset = end - start;
        self.b.commands.spawn((
            Name::new(name.to_owned()),
            Mesh3d(self.cylinder.clone()),
            MeshMaterial3d(material),
            Transform::from_translation((start + end) * 0.5)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, offset.normalize()))
                .with_scale(Vec3::new(diameter, offset.length(), diameter)),
        ));
    }

    fn light(
        &mut self,
        position: Vec3,
        tint: Color,
        lumens: f32,
        range: f32,
        circuit: Option<Circuit>,
    ) {
        let lumens = lumens * 0.74;
        self.block(
            "Carcaça industrial",
            position + Vec3::Y * 0.035,
            Vec3::new(1.42, 0.13, 0.37),
            self.palette.coal.clone(),
            false,
        );
        let start = if matches!(circuit, Some(Circuit::Exit)) {
            0.06
        } else {
            1.0
        };
        let source = self.materials.add(emissive(tint, 1.65 * start));
        self.block(
            "Difusor industrial",
            position - Vec3::Y * 0.047,
            Vec3::new(1.21, 0.046, 0.225),
            source.clone(),
            false,
        );
        let fixture = self
            .b
            .commands
            .spawn((
                SpotLight {
                    color: tint,
                    intensity: lumens * start,
                    inner_angle: 0.41,
                    outer_angle: 0.76,
                    range,
                    ..default()
                },
                Transform::from_translation(position - Vec3::Y * 0.15)
                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            ))
            .id();
        if let Some(circuit) = circuit {
            self.b.commands.entity(fixture).insert(CircuitLight {
                circuit,
                lumens,
                glow: tint.to_linear() * 1.65,
                lens: source,
            });
        }
    }

    fn spill(&mut self, position: Vec3, tint: Color, lumens: f32, range: f32) {
        self.b.commands.spawn((
            PointLight {
                color: tint,
                intensity: lumens * 0.64,
                range: range * 0.91,
                ..default()
            },
            Transform::from_translation(position),
        ));
    }
}
