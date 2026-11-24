use super::{
    Lab, Options,
    room::{FLOOR, Guide, LAYER},
    settings::{Config, Model, Surface},
};
use crate::{
    app::{CollisionWorld, Screen, Session},
    scene::{presence, station::EXIT_GATE},
};
use ainda_vivo_core::collision::{Aabb, body_bounds};
use bevy::{
    animation::{
        RepeatAnimation,
        graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex},
    },
    asset::{LoadState, RenderAssetUsages},
    camera::{
        primitives::MeshAabb,
        visibility::{NoFrustumCulling, RenderLayers},
    },
    gltf::{Gltf, GltfLoaderSettings, GltfMesh, GltfSkin},
    light::NotShadowCaster,
    math::Affine3A,
    mesh::{
        PrimitiveTopology, VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
    scene::SceneInstanceReady,
};
use std::collections::{BTreeSet, HashSet};

#[derive(Clone)]
struct Piece {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
}

#[derive(Clone, Default)]
struct Record {
    parts: Vec<Piece>,
    colliders: Vec<Aabb>,
    stats: Stats,
}

#[derive(Clone, Default)]
pub(super) struct Stats {
    pub meters: Option<Vec3>,
    pub vertices: usize,
    pub triangles: usize,
    pub materials: usize,
    pub textures: usize,
    pub bones: usize,
    pub colliders: usize,
}

pub(super) struct Clip {
    pub label: String,
    pub duration: f32,
    node: AnimationNodeIndex,
}

struct Imported {
    scene: Handle<Scene>,
    stats: Stats,
    graph: Handle<AnimationGraph>,
    clips: Vec<Clip>,
}

#[derive(Resource, Default)]
pub(super) struct Catalog {
    handle: Handle<Gltf>,
    import: Option<Imported>,
    objects: Option<[Record; 5]>,
    overrides: [Handle<StandardMaterial>; 3],
    revision: u32,
    station_entity: Option<Entity>,
    pub error: Option<String>,
}

impl Catalog {
    pub fn reload(&self, assets: &AssetServer) {
        if let Some(path) = self.handle.path() {
            assets.reload(path.clone());
        }
    }

    pub fn clips(&self) -> &[Clip] {
        self.import.as_ref().map_or(&[], |model| &model.clips)
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }

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
pub(super) struct Part {
    owner: Entity,
    original: Handle<StandardMaterial>,
}

#[derive(Component)]
pub(super) struct Player {
    owner: Entity,
    clip: usize,
}

#[derive(Component)]
pub(super) struct StationPreview;

pub(super) fn begin_import(
    mut catalog: ResMut<Catalog>,
    options: Res<Options>,
    server: Res<AssetServer>,
) {
    catalog.handle =
        server.load_with_settings(options.path(), |settings: &mut GltfLoaderSettings| {
            settings.load_cameras = false;
            settings.load_lights = false;
        });
}

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

type GltfAssets<'w> = (
    Res<'w, Assets<Gltf>>,
    Res<'w, Assets<GltfMesh>>,
    Res<'w, Assets<GltfSkin>>,
);

#[allow(
    clippy::too_many_arguments,
    reason = "Asset types and original equipment remain independently scoped."
)]
pub(super) fn finish_import(
    mut commands: Commands,
    mut catalog: ResMut<Catalog>,
    server: Res<AssetServer>,
    mut events: MessageReader<AssetEvent<Gltf>>,
    data: GltfAssets,
    animation: (Res<Assets<AnimationClip>>, ResMut<Assets<AnimationGraph>>),
    spawning: (Res<Assets<Scene>>, Res<AppTypeRegistry>),
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    let id = catalog.handle.id();
    let changed = events.read().any(|event| matches!(event,
        AssetEvent::LoadedWithDependencies { id: loaded } | AssetEvent::Modified { id: loaded } if *loaded == id));
    if (changed || (catalog.import.is_none() && catalog.error.is_none()))
        && server.is_loaded_with_dependencies(id)
        && let Some(gltf) = data.0.get(id)
        && let Some(scene) = gltf
            .default_scene
            .clone()
            .or_else(|| gltf.scenes.first().cloned())
    {
        if let Some(asset) = spawning.0.get(&scene)
            && let Err(error) = asset.clone_with(&spawning.1)
        {
            let message = format!("Cena GLB incompatível: {error}");
            warn!("{message}");
            catalog.error = Some(message);
            return;
        }
        let mut stats = stats_for(
            gltf.meshes
                .iter()
                .filter_map(|handle| data.1.get(handle))
                .flat_map(|gltf_mesh| {
                    gltf_mesh
                        .primitives
                        .iter()
                        .map(|prim| (&prim.mesh, prim.material.as_ref(), Affine3A::IDENTITY))
                }),
            &meshes,
            &materials,
        );
        stats.bones = gltf
            .skins
            .iter()
            .filter_map(|handle| data.2.get(handle))
            .map(|skin| skin.joints.len())
            .sum();
        let mut ordered = gltf.named_animations.iter().collect::<Vec<_>>();
        ordered.sort_unstable_by_key(|(name, _)| *name);
        let mut graphs = animation.1;
        let (graph, nodes) =
            AnimationGraph::from_clips(ordered.iter().map(|(_, clip)| (*clip).clone()));
        let clips = ordered
            .into_iter()
            .zip(nodes)
            .map(|((label, clip), node)| Clip {
                label: label.to_string(),
                duration: animation.0.get(clip).map_or(0.0, AnimationClip::duration),
                node,
            })
            .collect();
        info!(
            "Dev Lab: GLB importado ({} triângulos, {} vértices, {} ossos, {} texturas).",
            stats.triangles, stats.vertices, stats.bones, stats.textures
        );
        catalog.import = Some(Imported {
            scene,
            stats,
            graph: graphs.add(graph),
            clips,
        });
        catalog.revision = catalog.revision.wrapping_add(1);
        catalog.error = None;
        if let Some(entity) = catalog.station_entity.take() {
            commands.entity(entity).despawn();
        }
    } else if let Some(LoadState::Failed(error)) = server.get_load_state(id) {
        catalog.error = Some(format!("Erro ao carregar GLB: {error}"));
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

fn stats_for<'a>(
    pieces: impl IntoIterator<
        Item = (
            &'a Handle<Mesh>,
            Option<&'a Handle<StandardMaterial>>,
            Affine3A,
        ),
    >,
    meshes: &Assets<Mesh>,
    materials: &Assets<StandardMaterial>,
) -> Stats {
    let mut stats = Stats::default();
    let mut material_ids = HashSet::new();
    let mut images = HashSet::new();
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = -low;
    for (asset, material, transform) in pieces {
        if let Some(mesh) = meshes.get(asset) {
            let vertices = mesh.count_vertices();
            let points = mesh.indices().map_or(vertices, |indices| indices.len());
            stats.vertices += vertices;
            stats.triangles += match mesh.primitive_topology() {
                PrimitiveTopology::TriangleList => points / 3,
                PrimitiveTopology::TriangleStrip => points.saturating_sub(2),
                _ => 0,
            };
            if let Some(aabb) = mesh.compute_aabb() {
                for point in corners(Aabb {
                    min: aabb.min().into(),
                    max: aabb.max().into(),
                }) {
                    let point = transform.transform_point3(point);
                    low = low.min(point);
                    high = high.max(point);
                }
            }
        }
        if let Some(material) = material
            && material_ids.insert(material.id())
            && let Some(mat) = materials.get(material)
        {
            images.extend(
                [
                    &mat.base_color_texture,
                    &mat.normal_map_texture,
                    &mat.metallic_roughness_texture,
                    &mat.occlusion_texture,
                    &mat.emissive_texture,
                ]
                .into_iter()
                .flatten()
                .map(Handle::id),
            );
        }
    }
    stats.meters = low.is_finite().then_some(high - low);
    stats.materials = material_ids.len();
    stats.textures = images.len();
    stats
}

fn corners(aabb: Aabb) -> [Vec3; 8] {
    std::array::from_fn(|i| {
        let mut point = Vec3::ZERO;
        for axis in 0..3 {
            point[axis] = if i & (1 << axis) != 0 {
                aabb.max[axis]
            } else {
                aabb.min[axis]
            };
        }
        point
    })
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

pub(super) fn sync(
    mut commands: Commands,
    lab: Res<Lab>,
    config: Res<Config>,
    catalog: Res<Catalog>,
    mut viewers: Query<(Entity, &Viewer, &ChildOf, &mut Transform, &mut Visibility), Without<Part>>,
    parts: PartControls,
) {
    let Some(active) = &lab.active else { return };
    let mut found = BTreeSet::new();
    for (entity, viewer, parent, mut transform, mut visibility) in &mut viewers {
        let current = config.instances.get(viewer.index);
        if parent.parent() != active.root
            || current.is_none_or(|spec| spec.model != viewer.model)
            || (viewer.model == Model::Presence && viewer.revision != catalog.revision)
        {
            commands.entity(entity).despawn();
            continue;
        }
        found.insert(viewer.index);
        let spec = current.expect("instância conferida");
        let mut pose = spec.transform();
        pose.translation.y += FLOOR;
        if *transform != pose {
            *transform = pose;
        }
        let shown = if spec.visible {
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
        let mut pose = spec.transform();
        pose.translation.y += FLOOR;
        let root = commands
            .spawn((
                Name::new(spec.model.name()),
                Viewer {
                    index,
                    model: spec.model,
                    revision: catalog.revision,
                },
                ChildOf(active.root),
                pose,
                if spec.visible {
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

fn on_model_ready(
    ready: On<SceneInstanceReady>,
    mut commands: Commands,
    hierarchy: Query<&Children>,
    materials: Query<&MeshMaterial3d<StandardMaterial>>,
    animators: Query<&AnimationPlayer>,
    catalog: Res<Catalog>,
) {
    let Some(import) = &catalog.import else {
        return;
    };
    for descendant in hierarchy.iter_descendants(ready.entity) {
        if let Ok(material) = materials.get(descendant) {
            commands.entity(descendant).insert((
                RenderLayers::layer(LAYER),
                NoFrustumCulling,
                Part {
                    owner: ready.entity,
                    original: material.0.clone(),
                },
            ));
        }
        if animators.contains(descendant) {
            commands.entity(descendant).insert((
                AnimationGraphHandle(import.graph.clone()),
                Player {
                    owner: ready.entity,
                    clip: usize::MAX,
                },
            ));
        }
    }
}

pub(super) fn animate(
    config: Res<Config>,
    catalog: Res<Catalog>,
    mut lab: ResMut<Lab>,
    mut animators: Query<(&mut AnimationPlayer, &mut Player)>,
    viewers: Query<&Viewer>,
) {
    let Some(active) = &mut lab.active else {
        return;
    };
    let controls = &mut active.playback;
    let available = catalog.clips();
    for (mut player, mut state) in &mut animators {
        let Ok(view) = viewers.get(state.owner) else {
            continue;
        };
        let Some(spec) = config.instances.get(view.index) else {
            continue;
        };
        let index = spec.clip.min(available.len().saturating_sub(1));
        let Some(clip) = available.get(index) else {
            continue;
        };
        if state.clip != index {
            player.stop_all().start(clip.node);
            state.clip = index;
        }
        let Some(animation) = player.animation_mut(clip.node) else {
            continue;
        };
        if view.index != config.selected {
            if !animation.is_paused() {
                animation.pause();
            }
            continue;
        }
        if std::mem::take(&mut controls.rewind) {
            animation.replay();
        }
        let seek_to = controls.seek.take().map(|f| clip.duration * f);
        let offset = std::mem::take(&mut controls.seek_by);
        if seek_to.is_some() || offset != 0.0 {
            let next =
                (seek_to.unwrap_or(animation.seek_time()) + offset).clamp(0.0, clip.duration);
            animation.set_seek_time(next);
        }
        animation.set_repeat(if controls.repeat {
            RepeatAnimation::Forever
        } else {
            RepeatAnimation::Never
        });
        animation.set_speed(controls.speed);
        if controls.running {
            animation.resume();
        } else {
            animation.pause();
        }
    }
}

pub(super) fn timeline(
    selected: usize,
    catalog: &Catalog,
    animators: &Query<(&AnimationPlayer, &Player)>,
    viewers: &Query<&Viewer>,
) -> Option<(f32, f32)> {
    animators.iter().find_map(|(player, state)| {
        let view = viewers.get(state.owner).ok()?;
        if view.index != selected {
            return None;
        }
        let clip = catalog.clips().get(state.clip)?;
        Some((player.animation(clip.node)?.seek_time(), clip.duration))
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
    viewers: Query<(Entity, &Viewer, &GlobalTransform)>,
    mesh_poses: Query<(&Part, &SkinnedMesh)>,
    hierarchy: Query<(&GlobalTransform, Option<&ChildOf>)>,
    mut guides: GuideMeshes,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (_, guide, _, mut visible) in &mut guides {
        if *guide == Guide::Wire {
            continue;
        }
        *visible = Visibility::Hidden;
        if *guide == Guide::Bones && !config.skeleton {
            continue;
        }
    }
    let Some(spec) = config.selected().filter(|s| s.visible) else {
        return;
    };
    if !config.skeleton && !spec.bounds {
        return;
    }
    let Some((root, _, pose)) = viewers.iter().find(|(_, v, _)| v.index == config.selected) else {
        return;
    };
    for (_, guide, asset, mut visible) in &mut guides {
        let mut lines = Vec::new();
        match guide {
            Guide::Bones if config.skeleton && spec.model == Model::Presence => {
                for (_, skin) in mesh_poses.iter().filter(|(part, _)| part.owner == root) {
                    for joint in &skin.joints {
                        if let Ok((child, Some(parent))) = hierarchy.get(*joint)
                            && skin.joints.contains(&parent.parent())
                            && let Ok((parent, _)) = hierarchy.get(parent.parent())
                        {
                            lines.extend([
                                child.translation().to_array(),
                                parent.translation().to_array(),
                            ]);
                        }
                    }
                }
            }
            Guide::Bounds if spec.bounds && spec.model != Model::Presence => {
                if let Some(record) = catalog.record(spec.model) {
                    for collider in &record.colliders {
                        let points =
                            corners(*collider).map(|point| pose.transform_point(point).to_array());
                        for i in 0..8 {
                            for axis in 0..3 {
                                if i & (1 << axis) == 0 {
                                    lines.extend([points[i], points[i | (1 << axis)]]);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        if !lines.is_empty()
            && let Some(mesh) = meshes.get_mut(&asset.0)
        {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, lines);
            *visible = Visibility::Inherited;
        }
    }
}
