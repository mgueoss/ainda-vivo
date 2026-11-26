use super::{
    motion::{self, ClipMotion, Clock},
    observer::Pose,
};
#[cfg(feature = "dev-lab")]
use crate::lab::{Lab, library::Record, room::LAYER};
use ainda_vivo_core::collision::Aabb;
#[cfg(feature = "dev-lab")]
use bevy::camera::visibility::RenderLayers;
use bevy::mesh::PrimitiveTopology;
use bevy::{
    animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex},
    asset::LoadState,
    camera::{primitives::MeshAabb, visibility::NoFrustumCulling},
    gltf::{Gltf, GltfLoaderSettings, GltfMesh, GltfSkin},
    math::Affine3A,
    prelude::*,
    scene::SceneInstanceReady,
};
use std::collections::HashSet;

pub(crate) const EMBEDDED: &str = "embedded://ainda_vivo/presenca.glb";
#[derive(Resource)]
pub(crate) struct Source(pub(crate) &'static str);

#[derive(Clone, Default)]
pub(crate) struct Stats {
    pub meters: Option<Vec3>,
    pub vertices: usize,
    pub triangles: usize,
    pub materials: usize,
    pub textures: usize,
    pub bones: usize,
    #[cfg(feature = "dev-lab")]
    pub colliders: usize,
}

pub(crate) struct Clip {
    pub label: String,
    pub duration: f32,
    pub motion: ClipMotion,
    pub(crate) node: AnimationNodeIndex,
}

pub(crate) struct Imported {
    pub(crate) scene: Handle<Scene>,
    #[cfg(feature = "dev-lab")]
    pub(crate) stats: Stats,
    pub(crate) graph: Handle<AnimationGraph>,
    pub(crate) clips: Vec<Clip>,
}

#[derive(Resource, Default)]
pub(crate) struct Catalog {
    handle: Handle<Gltf>,
    pub(crate) import: Option<Imported>,
    #[cfg(feature = "dev-lab")]
    pub(crate) objects: Option<[Record; 5]>,
    #[cfg(feature = "dev-lab")]
    pub(crate) overrides: [Handle<StandardMaterial>; 3],
    revision: u32,
    #[cfg(feature = "dev-lab")]
    pub(crate) station_entity: Option<Entity>,
    pub error: Option<String>,
}

impl Catalog {
    pub fn spawn_observer(
        &self,
        commands: &mut Commands,
        pose: Pose,
        transform: Transform,
        parent: Option<Entity>,
    ) -> Option<Entity> {
        let scene = &self.import.as_ref()?.scene;
        let mut root = commands.spawn((
            Name::new("A Presença · Observador"),
            SceneRoot(scene.clone()),
            pose,
            transform,
            Visibility::Inherited,
        ));
        if let Some(parent) = parent {
            root.insert(ChildOf(parent));
        }
        Some(root.observe(on_model_ready).id())
    }

    #[cfg(feature = "dev-lab")]
    pub fn reload(&self, assets: &AssetServer) {
        if let Some(path) = self.handle.path() {
            assets.reload(path.clone());
        }
    }

    pub fn clips(&self) -> &[Clip] {
        self.import.as_ref().map_or(&[], |model| &model.clips)
    }

    pub fn number(&self, id: u8) -> Option<usize> {
        self.clips().iter().position(|clip| {
            clip.motion.duration > 0.0
                && clip.label.get(..2).and_then(|n| n.parse().ok()) == Some(id)
        })
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }
}

#[cfg(feature = "dev-lab")]
#[derive(Component)]
pub(crate) struct Part {
    pub(crate) owner: Entity,
    pub(crate) original: Handle<StandardMaterial>,
}

#[derive(Component)]
pub(crate) struct Player {
    pub(crate) owner: Entity,
    pub(crate) clip: usize,
    pub(crate) clock: Clock,
    pub(crate) fade: Fade,
    pub(crate) evaluating: bool,
}

impl Player {
    pub(crate) fn evaluate(
        &mut self,
        animator: Entity,
        enabled: bool,
        graph: &Handle<AnimationGraph>,
        commands: &mut Commands,
    ) {
        if enabled == self.evaluating {
            return;
        }
        self.evaluating = enabled;
        if enabled {
            commands
                .entity(animator)
                .insert(AnimationGraphHandle(graph.clone()));
        } else {
            commands.entity(animator).remove::<AnimationGraphHandle>();
        }
    }
}

#[derive(Default)]
pub(crate) struct Fade {
    pub(crate) from: Vec<(AnimationNodeIndex, f32)>,
    time_left: f32,
    duration: f32,
}

impl Fade {
    pub(crate) fn start(
        &mut self,
        player: &mut AnimationPlayer,
        node: AnimationNodeIndex,
        duration: f32,
    ) {
        self.from.clear();
        if duration > 0.0 {
            self.from.extend(
                player
                    .playing_animations()
                    .map(|(index, animation)| (*index, animation.weight()))
                    .filter(|(_, weight)| *weight > 0.001),
            );
        }
        self.duration = duration;
        self.time_left = duration;
        if self.from.is_empty() {
            player.stop_all();
        }
        let initial = if self.from.is_empty() {
            1.0
        } else {
            self.from
                .iter()
                .find(|(index, _)| *index == node)
                .map_or(0.0, |(_, w)| *w)
        };
        player.start(node).pause().set_weight(initial);
    }

    pub(crate) fn advance(
        &mut self,
        player: &mut AnimationPlayer,
        node: AnimationNodeIndex,
        dt: f32,
    ) {
        if self.from.is_empty() || dt <= 0.0 {
            return;
        }
        self.time_left = (self.time_left - dt).max(0.0);
        let linear = 1.0 - self.time_left / self.duration;
        let incoming = linear * linear * (3.0 - 2.0 * linear);
        let mut weight = incoming;
        for (index, old) in self.from.iter().copied() {
            if index == node {
                weight += old * (1.0 - incoming);
            } else if self.time_left == 0.0 {
                player.stop(index);
            } else if let Some(outgoing) = player.animation_mut(index) {
                outgoing.set_weight(old * (1.0 - incoming));
            }
        }
        if let Some(current) = player.animation_mut(node) {
            current.set_weight(weight);
        }
        if self.time_left == 0.0 {
            self.from.clear();
        }
    }
}

pub(crate) fn begin_import(
    mut catalog: ResMut<Catalog>,
    source: Res<Source>,
    server: Res<AssetServer>,
) {
    catalog.handle = server.load_with_settings(source.0, |settings: &mut GltfLoaderSettings| {
        settings.load_cameras = false;
        settings.load_lights = false;
    });
}

type GltfAssets<'w> = (
    Res<'w, Assets<Gltf>>,
    Res<'w, Assets<GltfMesh>>,
    Res<'w, Assets<GltfSkin>>,
);

#[allow(
    clippy::too_many_arguments,
    reason = "Asset types and original equipment remain independently scoped."
)]
pub(crate) fn finish_import(
    #[cfg(feature = "dev-lab")] mut commands: Commands,
    mut catalog: ResMut<Catalog>,
    server: Res<AssetServer>,
    mut events: MessageReader<AssetEvent<Gltf>>,
    data: GltfAssets,
    animation: (Res<Assets<AnimationClip>>, ResMut<Assets<AnimationGraph>>),
    spawning: (Res<Assets<Scene>>, Res<AppTypeRegistry>),
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) {
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
        let manifest = ClipMotion::load().unwrap_or_else(|error| {
            warn!("Metadados da Presença inválidos: {error}");
            Vec::new()
        });
        let clips: Vec<_> = ordered
            .into_iter()
            .zip(nodes)
            .map(|((label, clip), node)| {
                let duration = animation.0.get(clip).map_or(0.0, AnimationClip::duration);
                let motion = manifest
                    .iter()
                    .find(|meta| meta.label == **label && (meta.duration - duration).abs() < 0.001)
                    .cloned()
                    .unwrap_or_else(|| ClipMotion::legacy(label));
                Clip {
                    label: label.to_string(),
                    duration,
                    motion,
                    node,
                }
            })
            .collect();
        let required = [
            8, 10, 12, 13, 14, 17, 19, 23, 25, 26, 27, 28, 31, 32, 34, 37,
        ];
        if let Some(missing) = required.into_iter().find(|id| {
            !clips.iter().any(|clip| {
                clip.motion.duration > 0.0
                    && clip
                        .label
                        .get(..2)
                        .and_then(|prefix| prefix.parse::<u8>().ok())
                        == Some(*id)
            })
        }) {
            let message =
                format!("GLB da Presença sem clipe obrigatório {missing:02} com meta válida.");
            warn!("{message}");
            catalog.error = Some(message);
            return;
        }
        info!(
            "Presença: GLB importado ({} triângulos, {} vértices, {} ossos, {} texturas).",
            stats.triangles, stats.vertices, stats.bones, stats.textures
        );
        catalog.import = Some(Imported {
            scene,
            #[cfg(feature = "dev-lab")]
            stats,
            graph: graphs.add(graph),
            clips,
        });
        catalog.revision = catalog.revision.wrapping_add(1);
        catalog.error = None;
        #[cfg(feature = "dev-lab")]
        if let Some(entity) = catalog.station_entity.take() {
            commands.entity(entity).despawn();
        }
    } else if let Some(LoadState::Failed(error)) = server.get_load_state(id) {
        catalog.error = Some(format!("Erro ao carregar GLB: {error}"));
    }
}

pub(crate) fn stats_for<'a>(
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

pub(crate) fn corners(aabb: Aabb) -> [Vec3; 8] {
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

pub(crate) fn on_model_ready(
    ready: On<SceneInstanceReady>,
    mut commands: Commands,
    hierarchy: Query<&Children>,
    materials: Query<&MeshMaterial3d<StandardMaterial>>,
    animators: Query<&AnimationPlayer>,
    catalog: Res<Catalog>,
    #[cfg(feature = "dev-lab")] physical: Query<&Pose>,
) {
    let Some(import) = &catalog.import else {
        return;
    };
    #[cfg(feature = "dev-lab")]
    let in_lab = physical.get(ready.entity).map_or(true, |pose| pose.lab);
    for descendant in hierarchy.iter_descendants(ready.entity) {
        if let Ok(_material) = materials.get(descendant) {
            commands.entity(descendant).insert(NoFrustumCulling);
            #[cfg(feature = "dev-lab")]
            {
                commands.entity(descendant).insert(Part {
                    owner: ready.entity,
                    original: _material.0.clone(),
                });
                if in_lab {
                    commands
                        .entity(descendant)
                        .insert(RenderLayers::layer(LAYER));
                }
            }
        }
        if animators.contains(descendant) {
            commands.entity(descendant).insert((
                AnimationGraphHandle(import.graph.clone()),
                Player {
                    owner: ready.entity,
                    clip: usize::MAX,
                    clock: Clock::default(),
                    fade: Fade::default(),
                    evaluating: true,
                },
            ));
        }
    }
}

pub(crate) fn animate_observer(
    mut commands: Commands,
    catalog: Res<Catalog>,
    #[cfg(feature = "dev-lab")] lab: Option<Res<Lab>>,
    time: Res<Time>,
    mut animators: Query<(Entity, &mut AnimationPlayer, &mut Player)>,
    roots: Query<(&Pose, &Transform)>,
    mut signals: MessageWriter<super::observer::hunt::Signal>,
) {
    let Some(import) = &catalog.import else {
        return;
    };
    for (entity, mut player, mut state) in &mut animators {
        let Ok((pose, transform)) = roots.get(state.owner) else {
            continue;
        };
        #[cfg(feature = "dev-lab")]
        let paused_in_lab = !pose.lab && lab.as_ref().is_some_and(|lab| lab.is_open());
        #[cfg(not(feature = "dev-lab"))]
        let paused_in_lab = false;
        if pose.paused || paused_in_lab {
            state.evaluate(entity, false, &import.graph, &mut commands);
            continue;
        }
        let Some(index) = catalog.number(pose.clip) else {
            continue;
        };
        let clip = &import.clips[index];
        let mut dirty = index != state.clip;
        if dirty {
            let blend = import
                .clips
                .get(state.clip)
                .map_or(0.0, |old| clip.motion.state.blend_seconds(old.motion.state));
            state.fade.start(&mut player, clip.node, blend);
            state.clip = index;
            state.clock = Clock::default();
        }
        let dt = time.delta_secs();
        let mut marker = |event| {
            use super::observer::hunt::{Cue, Signal};
            if !matches!(pose.clip, 13 | 14 | 19 | 26..=28) {
                return;
            }
            let (cue, lateral, depth) = match event {
                motion::EventKind::FootLeft => (Cue::Foot, 0.23, -0.75),
                motion::EventKind::FootRight => (Cue::Foot, -0.23, -0.75),
                motion::EventKind::HandLeft => (Cue::Hand, 0.27, 0.47),
                motion::EventKind::HandRight => (Cue::Hand, -0.27, 0.47),
                _ => return,
            };
            let cue = match pose.clip {
                13 => Cue::Walk,
                14 => Cue::Hurry,
                19 => Cue::Approach,
                _ => cue,
            };
            signals.write(Signal {
                cue,
                at: transform.transform_point(Vec3::new(lateral, 0.08, depth)),
                lab: pose.lab,
            });
        };
        if let Some(phase) = pose.sample {
            let at = clip.duration * phase;
            if pose.clip == 26 && at > state.clock.time {
                let delta = at - state.clock.time;
                state
                    .clock
                    .advance(delta, &clip.motion, clip.duration, false, &mut marker);
            } else {
                state.clock.seek(at, &clip.motion, clip.duration);
            }
        } else {
            state.clock.advance(
                dt * pose.speed,
                &clip.motion,
                clip.duration,
                clip.motion.repeat,
                marker,
            );
        }
        dirty |= !state.fade.from.is_empty();
        state.fade.advance(&mut player, clip.node, dt);
        if let Some(animation) = player.animation_mut(clip.node)
            && animation.seek_time() != state.clock.time
        {
            animation.set_seek_time(state.clock.time);
            dirty = true;
        }
        state.evaluate(entity, dirty, &import.graph, &mut commands);
    }
}
