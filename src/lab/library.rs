use super::{
    Lab, Options,
    motion::{self, Clock, Limb, MotionEvent},
    room::{FLOOR, Guide, LAYER, Seat},
    settings::{Config, Model, Surface},
};
pub(crate) use crate::presence::rig::{Catalog, Part, Player, Stats};
use crate::{
    app::{CollisionWorld, Screen, Session},
    presence::rig::{corners, on_model_ready, stats_for},
    scene::{presence, station::EXIT_GATE},
};
use ainda_vivo_core::collision::{Aabb, body_bounds};
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::RenderLayers,
    light::NotShadowCaster,
    mesh::{
        PrimitiveTopology, VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
};
use std::collections::{BTreeSet, HashSet};

#[derive(Clone)]
struct Piece {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
}

#[derive(Clone, Default)]
pub(crate) struct Record {
    parts: Vec<Piece>,
    colliders: Vec<Aabb>,
    stats: Stats,
}

impl Catalog {
    pub fn stats(&self, model: Model) -> Option<&Stats> {
        match model {
            Model::Presence => self.import.as_ref().map(|mesh| &mesh.stats),
            _ => self.record(model).map(|record| &record.stats),
        }
    }

    pub fn height(&self, model: Model) -> f32 {
        self.stats(model)
            .and_then(|s| s.meters)
            .map_or(2.20, |d| d.y)
    }

    fn record(&self, model: Model) -> Option<&Record> {
        let index = (model as usize).checked_sub(1)?;
        self.objects.as_ref()?.get(index)
    }
}

#[derive(Component)]
pub(super) struct Viewer {
    pub index: usize,
    model: Model,
    revision: u32,
}

#[derive(Component)]
pub(super) struct StationPreview;

type StationMeshes<'w, 's> = Query<
    'w,
    's,
    (
        &'static Name,
        &'static Mesh3d,
        &'static MeshMaterial3d<StandardMaterial>,
        &'static Transform,
    ),
    Without<Part>,
>;

pub(super) fn prepare_lab_models(
    mut commands: Commands,
    mut catalog: ResMut<Catalog>,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    station: StationMeshes,
    solids: Res<CollisionWorld>,
    options: Res<Options>,
    session: Res<Session>,
) {
    if catalog.objects.is_none() {
        catalog.objects = Some(station_records(
            &station,
            &solids,
            &mut meshes,
            &mut materials,
        ));
        catalog.overrides = [
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.59, 0.54, 0.47),
                perceptual_roughness: 0.91,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.10, 0.11, 0.12),
                unlit: true,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgba(0.63, 0.69, 0.65, 0.26),
                perceptual_roughness: 0.86,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        ];
    }
    if options.preview_station
        && session.screen != Screen::Menu
        && catalog.station_entity.is_none()
        && let Some(import) = &catalog.import
    {
        catalog.station_entity = Some(
            commands
                .spawn((
                    StationPreview,
                    Name::new("Prévia passiva · A Presença 3D"),
                    SceneRoot(import.scene.clone()),
                    Transform::from_xyz(0.43, 0.002, 3.63)
                        .with_rotation(Quat::from_rotation_y(-0.13)),
                ))
                .id(),
        );
    }
}

fn station_records(
    station: &StationMeshes,
    world: &CollisionWorld,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> [Record; 5] {
    let legacy = legacy_record(meshes, materials);
    let mut pump = Record::default();
    let mut door = Record::default();
    let mut pillar = Record::default();
    let mut human = Record::default();
    let pump_anchor = station
        .iter()
        .filter(|(name, ..)| name.as_str() == "Base chumbada")
        .max_by(|(_, _, _, a), (_, _, _, b)| a.translation.x.total_cmp(&b.translation.x))
        .map_or(Vec3::ZERO, |(_, _, _, t)| t.translation.with_y(0.0));
    let gate_anchor = EXIT_GATE.with_y(EXIT_GATE.y - 1.48);
    for (name, mesh, material, pose) in station {
        let name = name.as_str();
        let piece = |offset: Vec3| Piece {
            mesh: mesh.0.clone(),
            material: material.0.clone(),
            transform: Transform {
                translation: pose.translation - offset,
                ..*pose
            },
        };
        if matches!(name, "Comporta da saída" | "Nervura da comporta") {
            door.parts.push(piece(gate_anchor));
        } else if name == "Pilar da nave" && pillar.parts.is_empty() {
            pillar.parts.push(piece(
                pose.translation
                    .with_y(pose.translation.y - pose.scale.y * 0.5),
            ));
            pillar.colliders.push(Aabb::from_center_size(
                Vec3::Y * pose.scale.y * 0.5,
                pose.scale,
            ));
        } else if matches!(
            name,
            "Base chumbada"
                | "Apoio do motor"
                | "Motor da bomba"
                | "Anel de arrefecimento do motor"
                | "Carcaça da bomba"
                | "Flange de inspeção"
                | "Tubo de descarga"
                | "Suporte de instrumentos"
                | "Etiqueta da bomba"
        ) && (pose.translation - pump_anchor).length() < 4.5
        {
            pump.parts.push(piece(pump_anchor));
        }
    }
    for solid in &world.0 {
        let Aabb { min, max } = solid.bounds;
        let center = (min + max) * 0.5;
        if solid.gate {
            door.colliders.push(Aabb {
                min: min - gate_anchor,
                max: max - gate_anchor,
            });
        } else if (center - pump_anchor).length() < 3.5
            && (max - min).max_element() < 5.3
            && center.y < 2.2
        {
            pump.colliders.push(Aabb {
                min: min - pump_anchor,
                max: max - pump_anchor,
            });
        }
    }
    let suit = super::room::material(materials, Color::srgb(0.46, 0.423, 0.36));
    let padding = super::room::material(materials, Color::srgb(0.29, 0.303, 0.286));
    let skin = super::room::material(materials, Color::srgb(0.59, 0.54, 0.46));
    let sphere = meshes.add(Sphere::new(1.0).mesh().uv(24, 14));
    for (pos, dims, material) in [
        ([0., 1.67, 0.], [0.091, 0.11, 0.082], &skin),
        ([0., 1.425, 0.], [0.182, 0.215, 0.096], &suit),
        ([0., 1.058, 0.], [0.150, 0.18, 0.081], &padding),
    ] {
        human.parts.push(Piece {
            mesh: sphere.clone(),
            material: material.clone(),
            transform: Transform::from_translation(pos.into()).with_scale(dims.into()),
        });
    }
    for side in [-1.0, 1.0] {
        for (top, bottom, radius, material) in [
            (
                [0.155 * side, 1.52, 0.],
                [0.214 * side, 1.259, 0.004],
                0.053,
                &suit,
            ),
            (
                [0.214 * side, 1.259, 0.004],
                [0.233 * side, 1.035, 0.045],
                0.042,
                &suit,
            ),
            (
                [0.233 * side, 1.039, 0.045],
                [0.240 * side, 0.961, 0.059],
                0.034,
                &skin,
            ),
            (
                [0.09 * side, 1.04, 0.],
                [0.09 * side, 0.58, 0.012],
                0.076,
                &padding,
            ),
            (
                [0.09 * side, 0.58, 0.012],
                [0.10 * side, 0.099, 0.006],
                0.058,
                &padding,
            ),
            (
                [0.10 * side, 0.060, -0.010],
                [0.124 * side, 0.060, 0.117],
                0.059,
                &padding,
            ),
        ] {
            let (a, b) = (Vec3::from(top), Vec3::from(bottom));
            human.parts.push(Piece {
                mesh: meshes.add(Capsule3d::new(radius, a.distance(b))),
                material: material.clone(),
                transform: Transform::from_translation((a + b) * 0.5)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, (a - b).normalize())),
            });
        }
    }
    human.colliders.push(body_bounds(
        Vec3::ZERO,
        ainda_vivo_core::movement::STANDING_HEIGHT,
    ));
    [legacy, pump, door, pillar, human].map(|mut record| {
        record.stats = stats_for(
            record
                .parts
                .iter()
                .map(|p| (&p.mesh, Some(&p.material), p.transform.compute_affine())),
            meshes,
            materials,
        );
        record.stats.colliders = record.colliders.len();
        record
    })
}

fn legacy_record(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Record {
    // Retoma exatamente a máscara da figura antiga. Os trechos de cada linha
    // viram um recorte plano opaco para avaliação também em luz de estúdio.
    let mask = presence::mask();
    let Some(pixels) = mask.data.as_ref() else {
        return Record::default();
    };
    let (width, height) = (mask.width() as usize, mask.height() as usize);
    let mut verts = Vec::new();
    let x = |column: usize| (column as f32 / width as f32 - 0.5) * presence::WIDTH;
    let y = |row: usize| (1.0 - row as f32 / height as f32) * presence::HEIGHT;
    for (row, pixels) in pixels.chunks_exact(width * 4).enumerate() {
        let covered = |i: usize| i < width && pixels[i * 4 + 3] >= 35;
        let mut column = 0;
        while column < width {
            if !covered(column) {
                column += 1;
                continue;
            }
            let start = column;
            while covered(column) {
                column += 1;
            }
            let (a, b, c, d) = (
                [x(start), y(row + 1), 0.0],
                [x(column), y(row + 1), 0.0],
                [x(column), y(row), 0.0],
                [x(start), y(row), 0.0],
            );
            verts.extend([a, b, c, a, c, d]);
        }
    }
    let cutout = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    Record {
        parts: vec![Piece {
            mesh: meshes.add(cutout),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.026, 0.030, 0.033),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::IDENTITY,
        }],
        ..default()
    }
}

type PartControls<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Part,
        Has<NotShadowCaster>,
        &'static MeshMaterial3d<StandardMaterial>,
    ),
>;

type SeatView<'w, 's> =
    Query<'w, 's, (&'static mut Visibility, &'static mut Transform), (With<Seat>, Without<Viewer>)>;

pub(super) fn sync(
    mut commands: Commands,
    lab: Res<Lab>,
    config: Res<Config>,
    catalog: Res<Catalog>,
    mut viewers: Query<(Entity, &Viewer, &ChildOf, &mut Transform, &mut Visibility), Without<Part>>,
    parts: PartControls,
    mut seat: SeatView,
) {
    let Some(active) = &lab.active else { return };
    if let Ok((mut shown, mut current)) = seat.get_mut(active.room.seat) {
        let actor = config
            .selected()
            .filter(|actor| active.playback.seat && actor.visible);
        shown.set_if_neq(if actor.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if let Some(actor) = actor {
            let mut pose = preview_transform(active, config.selected, config.selected, actor);
            pose.translation = pose.transform_point(Vec3::new(0., 0., -0.199));
            current.set_if_neq(pose);
        }
    }
    let mut found = BTreeSet::new();
    for (entity, viewer, parent, mut transform, mut visibility) in &mut viewers {
        let current = config.instances.get(viewer.index);
        if parent.parent() != active.root
            || current.is_none_or(|spec| spec.model != viewer.model)
            || (viewer.model == Model::Presence && viewer.revision != catalog.revision())
        {
            commands.entity(entity).despawn();
            continue;
        }
        found.insert(viewer.index);
        let spec = current.expect("instância conferida");
        let pose = preview_transform(active, config.selected, viewer.index, spec);
        if *transform != pose {
            *transform = pose;
        }
        let shown = if spec.visible && !active.observer.enabled {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != shown {
            *visibility = shown;
        }
    }
    for (index, spec) in config
        .instances
        .iter()
        .enumerate()
        .filter(|(i, _)| !found.contains(i))
    {
        if (spec.model == Model::Presence && catalog.import.is_none()) || catalog.objects.is_none()
        {
            continue;
        }
        let pose = preview_transform(active, config.selected, index, spec);
        let root = commands
            .spawn((
                Name::new(spec.model.name()),
                Viewer {
                    index,
                    model: spec.model,
                    revision: catalog.revision(),
                },
                ChildOf(active.root),
                pose,
                if spec.visible && !active.observer.enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ))
            .id();
        match spec.model {
            Model::Presence => {
                let import = catalog.import.as_ref().expect("GLB conferido");
                commands
                    .entity(root)
                    .insert(SceneRoot(import.scene.clone()))
                    .observe(on_model_ready);
            }
            prop => {
                let record = catalog.record(prop).expect("biblioteca inicializada");
                for part in &record.parts {
                    commands.spawn((
                        ChildOf(root),
                        RenderLayers::layer(LAYER),
                        part.transform,
                        Mesh3d(part.mesh.clone()),
                        MeshMaterial3d(part.material.clone()),
                        Part {
                            owner: root,
                            original: part.material.clone(),
                        },
                    ));
                }
            }
        }
    }
    for (id, part, noshadow, material) in &parts {
        let Ok((_, view, _, _, _)) = viewers.get(part.owner) else {
            continue;
        };
        let Some(spec) = config.instances.get(view.index) else {
            continue;
        };
        if spec.model != view.model {
            continue;
        }
        let want_shadow = spec.shadow && spec.model != Model::Legacy;
        if noshadow == want_shadow {
            let mut entity = commands.entity(id);
            if want_shadow {
                entity.remove::<NotShadowCaster>();
            } else {
                entity.insert(NotShadowCaster);
            }
        }
        let desired = if view.index == config.selected
            && (config.wireframe || (config.skeleton && spec.model == Model::Presence))
        {
            &catalog.overrides[2]
        } else {
            match spec.surface {
                Surface::Original => &part.original,
                Surface::Clay => &catalog.overrides[0],
                Surface::Silhouette => &catalog.overrides[1],
            }
        };
        if material.0 != *desired {
            commands.entity(id).insert(MeshMaterial3d(desired.clone()));
        }
    }
}

fn preview_transform(
    active: &super::Active,
    selected: usize,
    index: usize,
    spec: &super::settings::ModelSpec,
) -> Transform {
    let mut pose = spec.transform();
    pose.translation.y += FLOOR;
    if index != selected || spec.model != Model::Presence {
        return pose;
    }
    if let Some((position, yaw)) = active.playback.run.as_ref().and_then(motion::Run::path) {
        return Transform::from_translation(position).with_rotation(Quat::from_rotation_y(yaw));
    }
    pose.translation += active.playback.offset;
    pose
}

#[derive(PartialEq)]
struct WirePart {
    mesh: AssetId<Mesh>,
    transform: Mat4,
    bones: Vec<Mat4>,
}

#[derive(Default)]
pub(super) struct WireCache {
    entity: Option<Entity>,
    parts: Vec<WirePart>,
}

type WireSources<'w, 's> = Query<
    'w,
    's,
    (
        &'static Part,
        &'static Mesh3d,
        &'static GlobalTransform,
        Option<&'static SkinnedMesh>,
    ),
>;

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct WireInputs<'w, 's> {
    sources: WireSources<'w, 's>,
    joints: Query<'w, 's, &'static GlobalTransform>,
    binds: Res<'w, Assets<SkinnedMeshInverseBindposes>>,
}

pub(super) fn wire_guides(
    config: Res<Config>,
    viewers: Query<(Entity, &Viewer)>,
    input: WireInputs,
    mut guides: GuideMeshes,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cached: Local<WireCache>,
) {
    let Some((guide_id, _, guide_mesh, mut visibility)) = guides
        .iter_mut()
        .find(|(_, guide, ..)| **guide == Guide::Wire)
    else {
        return;
    };
    let selected = config.wireframe && config.selected().is_some_and(|s| s.visible);
    let root = selected
        .then(|| {
            viewers
                .iter()
                .find(|(_, view)| view.index == config.selected)
        })
        .flatten();
    let Some((root, _)) = root else {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        cached.entity = None;
        cached.parts.clear();
        return;
    };
    let parts: Vec<_> = input
        .sources
        .iter()
        .filter(|(part, ..)| part.owner == root)
        .map(|(_, mesh, transform, skin)| {
            let bones = skin
                .and_then(|skin| {
                    let inverses = input.binds.get(&skin.inverse_bindposes)?;
                    skin.joints
                        .iter()
                        .zip(inverses.iter())
                        .map(|(joint, local)| {
                            input
                                .joints
                                .get(*joint)
                                .ok()
                                .map(|world| world.to_matrix() * *local)
                        })
                        .collect::<Option<Vec<_>>>()
                })
                .unwrap_or_default();
            WirePart {
                mesh: mesh.0.id(),
                transform: transform.to_matrix(),
                bones,
            }
        })
        .collect();
    if cached.entity != Some(guide_id) || cached.parts != parts {
        let lines = parts
            .iter()
            .flat_map(|part| {
                meshes
                    .get(part.mesh)
                    .map_or_else(Vec::new, |mesh| wire_vertices(mesh, part))
            })
            .collect::<Vec<_>>();
        if let Some(drawing) = meshes.get_mut(&guide_mesh.0) {
            drawing.insert_attribute(Mesh::ATTRIBUTE_POSITION, lines);
        }
        cached.entity = Some(guide_id);
        cached.parts = parts;
    }
    let shown = if cached.parts.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    if *visibility != shown {
        *visibility = shown;
    }
}

fn wire_vertices(mesh: &Mesh, part: &WirePart) -> Vec<[f32; 3]> {
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
        return Vec::new();
    }
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Vec::new();
    };
    let mut world = positions
        .iter()
        .map(|p| part.transform.transform_point3(Vec3::from(*p)))
        .collect::<Vec<_>>();
    if !part.bones.is_empty()
        && let (
            Some(VertexAttributeValues::Uint16x4(joints)),
            Some(VertexAttributeValues::Float32x4(weights)),
        ) = (
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX),
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT),
        )
    {
        for (((vertex, position), bones), blend) in
            world.iter_mut().zip(positions).zip(joints).zip(weights)
        {
            let mut sum = Vec3::ZERO;
            for (&joint, &influence) in bones.iter().zip(blend) {
                if influence > 0.0
                    && let Some(matrix) = part.bones.get(joint as usize)
                {
                    sum += matrix.transform_point3(Vec3::from(*position)) * influence;
                }
            }
            *vertex = sum;
        }
    }
    let vertices = mesh.indices().map_or_else(
        || (0..world.len()).collect::<Vec<_>>(),
        |indices| indices.iter().collect(),
    );
    let mut edges = HashSet::new();
    let mut lines = Vec::with_capacity(vertices.len());
    for tri in vertices.chunks_exact(3) {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            if edges.insert((a.min(b), a.max(b)))
                && let (Some(start), Some(end)) = (world.get(a), world.get(b))
            {
                lines.extend([start.to_array(), end.to_array()]);
            }
        }
    }
    lines
}

pub(super) fn animate(
    config: Res<Config>,
    catalog: Res<Catalog>,
    mut lab: ResMut<Lab>,
    mut animators: Query<(Entity, &mut AnimationPlayer, &mut Player)>,
    viewers: Query<&Viewer>,
    time: Res<Time>,
    (mut events, mut commands): (MessageWriter<MotionEvent>, Commands),
) {
    let (Some(active), Some(import)) = (&mut lab.active, &catalog.import) else {
        return;
    };
    let observer = active.camera_transform(&config).translation;
    let controls = &mut active.playback;
    controls.seat = false;
    let available = catalog.clips();
    for (entity, mut player, mut state) in &mut animators {
        let Ok(view) = viewers.get(state.owner) else {
            continue;
        };
        if view.revision != catalog.revision() {
            continue;
        }
        let Some(spec) = config.instances.get(view.index) else {
            continue;
        };
        let selected = view.index == config.selected;
        let staged = controls.run.as_ref().filter(|_| selected);
        let rewind = selected && std::mem::take(&mut controls.rewind);
        if rewind {
            controls.offset = Vec3::ZERO;
        }
        if rewind && staged.is_some() {
            let mode = controls.run.as_ref().expect("demonstração ativa").mode;
            controls.run = Some(motion::Run::new(mode));
        }
        let staged = controls.run.as_ref().filter(|_| selected);
        let index = staged
            .and_then(|run| catalog.number(run.current().0))
            .unwrap_or_else(|| spec.clip.min(available.len().saturating_sub(1)));
        let Some(clip) = available.get(index) else {
            continue;
        };
        let changed = state.clip != index;
        let mut evaluate = changed || rewind;
        if changed || rewind {
            let blend = if changed && selected && controls.running {
                available.get(state.clip).map_or(0.0, |outgoing| {
                    clip.motion.state.blend_seconds(outgoing.motion.state)
                })
            } else {
                0.0
            };
            state.fade.start(&mut player, clip.node, blend);
            state.clip = index;
            state.clock = Clock::default();
        }
        if !selected {
            state.evaluate(entity, evaluate, &import.graph, &mut commands);
            continue;
        }
        controls.seat = matches!(clip.label.get(..2), Some("20" | "21"));
        let seek_to = controls.seek.take().map(|f| clip.duration * f);
        let step = f32::from(std::mem::take(&mut controls.step)) / clip.motion.fps.max(1.0);
        let offset = step;
        let duration = clip.duration;
        let previous = state.clock.distance;
        let mut finished = 0;
        if seek_to.is_some() || offset != 0.0 {
            if !state.fade.from.is_empty() {
                state.fade.start(&mut player, clip.node, 0.0);
                evaluate = true;
            }
            let next = seek_to.unwrap_or(state.clock.time) + offset;
            state.clock.seek(next, &clip.motion, duration);
        } else if controls.follow_gaze && controls.running && catalog.number(12) == Some(index) {
            let target = motion::gaze_phase(spec.transform(), observer) * duration;
            let dt = time.delta_secs() * controls.speed;
            let difference = target - state.clock.time;
            let next = if difference.abs() < 0.0001 {
                target
            } else {
                state.clock.time + difference * (1.0 - (-4.0 * dt).exp())
            };
            state.clock.seek(next, &clip.motion, duration);
            evaluate |= !state.fade.from.is_empty() && dt > 0.0;
            state.fade.advance(&mut player, clip.node, dt);
        } else if controls.running && !staged.is_some_and(|run| run.complete) {
            let repeat = staged.map_or(controls.repeat, |run| run.cycles + 1 < run.current().1);
            let dt = time.delta_secs() * controls.speed;
            let dt = staged.map_or(dt, |run| {
                dt.min(clip.duration * f32::from(run.current().1 - run.cycles) - state.clock.time)
            });
            let actor = state.owner;
            finished = state
                .clock
                .advance(dt, &clip.motion, duration, repeat, |kind| {
                    events.write(MotionEvent { actor, kind });
                });
            evaluate |= !state.fade.from.is_empty() && dt > 0.0;
            state.fade.advance(&mut player, clip.node, dt);
        }
        if let Some(animation) = player.animation_mut(clip.node)
            && animation.seek_time() != state.clock.time
        {
            animation.set_seek_time(state.clock.time);
            evaluate = true;
        }
        state.evaluate(entity, evaluate, &import.graph, &mut commands);
        if let Some(run) = &mut controls.run {
            let delta = (state.clock.distance - previous) * controls.speed_scale;
            run.translate(delta);
            if finished > 0 {
                run.finish_cycles(
                    finished,
                    controls.repeat && run.mode == motion::Mode::Sequence,
                );
                if run.complete {
                    controls.running = false;
                }
            }
        } else if controls.displacement && spec.visible {
            let pose = spec.transform();
            let delta = (state.clock.distance - previous) * controls.speed_scale * spec.scale;
            let direction = (pose.rotation * clip.motion.direction.vector()).with_y(0.0);
            let proposed = pose.translation + controls.offset + delta * direction;
            let size = clip
                .motion
                .posture
                .dimensions(motion::phase(state.clock.time, duration))
                * spec.scale;
            let bounds = Aabb::from_center_size(proposed + Vec3::Y * (size.y * 0.5 + 0.041), size);
            let crosses_room = active
                .room
                .solids
                .iter()
                .skip(1)
                .any(|solid| solid.overlaps(bounds));
            if proposed.x.abs() <= 7.6 && (-16.1..=5.05).contains(&proposed.z) && !crosses_room {
                controls.offset = proposed - pose.translation;
            }
            if finished > 0 && !controls.repeat {
                controls.running = false;
            }
        } else {
            controls.offset = Vec3::ZERO;
            if finished > 0 && !controls.repeat {
                controls.running = false;
            }
        }
    }
}

pub(super) fn remember_events(
    mut events: MessageReader<MotionEvent>,
    mut lab: ResMut<Lab>,
    config: Res<Config>,
    viewers: Query<&Viewer>,
) {
    let Some(active) = &mut lab.active else {
        return;
    };
    for event in events.read() {
        if viewers
            .get(event.actor)
            .is_ok_and(|view| view.index == config.selected)
        {
            active.playback.last_event = Some(event.kind);
        }
    }
}

pub(super) fn timeline(
    selected: usize,
    catalog: &Catalog,
    animators: &Query<(&AnimationPlayer, &Player)>,
    viewers: &Query<&Viewer>,
) -> Option<(usize, f32, f32)> {
    animators.iter().find_map(|(player, state)| {
        let view = viewers.get(state.owner).ok()?;
        if view.index != selected {
            return None;
        }
        let clip = catalog.clips().get(state.clip)?;
        Some((
            state.clip,
            player.animation(clip.node)?.seek_time(),
            clip.duration,
        ))
    })
}

type GuideMeshes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Guide,
        &'static Mesh3d,
        &'static mut Visibility,
    ),
>;

pub(super) fn guides(
    config: Res<Config>,
    catalog: Res<Catalog>,
    lab: Res<Lab>,
    viewers: Query<(Entity, &Viewer, &GlobalTransform)>,
    rig: GuideRig,
    mut guides: GuideMeshes,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(active) = &lab.active else { return };
    let selected = config.selected().filter(|s| s.visible).and_then(|spec| {
        let (root, _, pose) = viewers
            .iter()
            .find(|(_, v, _)| v.index == config.selected)?;
        Some((spec, root, pose))
    });
    let track = active
        .playback
        .run
        .as_ref()
        .is_some_and(|run| run.mode == motion::Mode::Track);
    for (_, guide, asset, mut visible) in &mut guides {
        if *guide == Guide::Wire {
            continue;
        }
        if *guide == Guide::Track {
            visible.set_if_neq(if track {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
            continue;
        }
        let Some((spec, root, pose)) = selected else {
            visible.set_if_neq(Visibility::Hidden);
            continue;
        };
        let mut lines = Vec::new();
        let clip = || {
            let state = rig.players.iter().find(|p| p.owner == root)?;
            let clip = catalog.clips().get(state.clip)?;
            Some((&clip.motion, motion::phase(state.clock.time, clip.duration)))
        };
        match guide {
            Guide::Bones if config.skeleton && spec.model == Model::Presence => {
                for (_, skin) in rig.meshes.iter().filter(|(part, _)| part.owner == root) {
                    for joint in &skin.joints {
                        if let Ok((child, Some(parent), _)) = rig.hierarchy.get(*joint)
                            && skin.joints.contains(&parent.parent())
                            && let Ok((parent, ..)) = rig.hierarchy.get(parent.parent())
                        {
                            lines.extend([
                                child.translation().to_array(),
                                parent.translation().to_array(),
                            ]);
                        }
                    }
                }
            }
            Guide::Bounds if spec.bounds => {
                if spec.model == Model::Presence {
                    if let Some((motion, phase)) = clip() {
                        let size = motion.posture.dimensions(phase);
                        bounds_lines(
                            &mut lines,
                            pose,
                            Aabb::from_center_size(Vec3::Y * size.y * 0.5, size),
                        );
                    }
                } else if let Some(record) = catalog.record(spec.model) {
                    for collider in &record.colliders {
                        bounds_lines(&mut lines, pose, *collider);
                    }
                }
            }
            Guide::Planted | Guide::Lifted
                if active.playback.contacts && spec.model == Model::Presence =>
            {
                if let Some((data, phase)) = clip() {
                    let ground = pose.translation().y.min(FLOOR) + 0.008;
                    let use_hands = matches!(
                        data.posture,
                        motion::Posture::Quad | motion::Posture::Fold | motion::Posture::Rise
                    );
                    for (_, skin) in rig.meshes.iter().filter(|(part, _)| part.owner == root) {
                        for limb in Limb::ALL {
                            if !use_hands && matches!(limb, Limb::HandLeft | Limb::HandRight)
                                || data.grounded(limb, phase) != (*guide == Guide::Planted)
                            {
                                continue;
                            }
                            let joint = skin
                                .joints
                                .iter()
                                .filter_map(|joint| rig.hierarchy.get(*joint).ok())
                                .find(|(.., name)| {
                                    name.is_some_and(|n| n.as_str() == limb.joint())
                                });
                            if let Some((joint, ..)) = joint {
                                let end = joint.translation();
                                let point = end.with_y(ground);
                                let delta = 0.055;
                                for axis in [Vec3::X, Vec3::Z] {
                                    lines.extend([
                                        (point - axis * delta).to_array(),
                                        (point + axis * delta).to_array(),
                                    ]);
                                }
                                lines.extend([point.to_array(), end.to_array()]);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        if lines.is_empty() {
            visible.set_if_neq(Visibility::Hidden);
        } else {
            let unchanged = meshes.get(&asset.0).is_some_and(|mesh| {
                matches!(mesh.attribute(Mesh::ATTRIBUTE_POSITION), Some(VertexAttributeValues::Float32x3(previous)) if *previous == lines)
            });
            if !unchanged && let Some(mesh) = meshes.get_mut(&asset.0) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, lines);
            }
            visible.set_if_neq(Visibility::Inherited);
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct GuideRig<'w, 's> {
    meshes: Query<'w, 's, (&'static Part, &'static SkinnedMesh)>,
    hierarchy: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            Option<&'static ChildOf>,
            Option<&'static Name>,
        ),
    >,
    players: Query<'w, 's, &'static Player>,
}

fn bounds_lines(lines: &mut Vec<[f32; 3]>, pose: &GlobalTransform, collider: Aabb) {
    let points = corners(collider).map(|point| pose.transform_point(point).to_array());
    for i in 0..8 {
        for axis in 0..3 {
            if i & (1 << axis) == 0 {
                lines.extend([points[i], points[i | (1 << axis)]]);
            }
        }
    }
}
