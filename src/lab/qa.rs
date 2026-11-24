//! Windows renderer integration checks for the prototype asset and the sandbox boundary.
use super::{
    Action, Active, Knob, Lab, Options,
    library::{self, Catalog, Part, StationPreview, Viewer},
    panel::Interface,
    room::{Guide, LabCamera},
    settings::{Config, Model, ModelSpec, Preset, Tab},
};
use crate::app::{Comfort, LocalPlayer, Screen, Session, VERSION, restart};
use crate::atmosphere::director::Director;
use ainda_vivo_core::story::Action as MissionAction;
use bevy::{
    app::AppExit,
    light::NotShadowCaster,
    mesh::skinning::SkinnedMesh,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

const PHOTO_START: u32 = 420;
const PHOTO_INTERVAL: u32 = 46;
const PHOTO_END: u32 = PHOTO_START + PHOTO_INTERVAL * 16;
const RETURN: u32 = PHOTO_END + 20;
const STRESS_START: u32 = RETURN + 360;
const STRESS_PERIOD: u32 = 80;
const STRESS_CYCLES: u32 = 32;
const STRESS_END: u32 = STRESS_START + STRESS_PERIOD * STRESS_CYCLES;
const SWITCHES: [u32; 12] = [
    40,
    81,
    100,
    121,
    282,
    RETURN,
    RETURN + 70,
    RETURN + 101,
    RETURN + 151,
    RETURN + 181,
    RETURN + 231,
    RETURN + 261,
];

#[derive(Default, Resource)]
pub(super) struct Check {
    frame: u32,
    ticks: u32,
    report: Option<PathBuf>,
    restoration: Option<Snapshot>,
    baseline: Option<Counts>,
    disk_revision: Option<u32>,
    occupancy: Vec<Counts>,
    stress: bool,
    stress_counts: Vec<Counts>,
    samples: Vec<f64>,
    previous_frame: Option<Instant>,
    first_pose_height: Option<f32>,
    first_person_start: f32,
    fly_start: f32,
    failures: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    position: Vec3,
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    screen: Screen,
    elapsed: f32,
    attempt: u64,
    revision: u32,
    progress: String,
    instability: f32,
    critical_remaining: Option<f32>,
    pacing: (f32, [u32; 3]),
    lantern: bool,
}

impl Snapshot {
    fn now(
        session: &Session,
        player: &LocalPlayer,
        comfort: &Comfort,
        director: &Director,
    ) -> Self {
        Self {
            position: player.body.feet,
            velocity: player.body.velocity,
            yaw: player.yaw,
            pitch: player.pitch,
            screen: session.screen,
            elapsed: session.elapsed,
            attempt: session.attempt,
            revision: session.perception_revision,
            progress: format!("{:?}", session.progress().phase()),
            instability: session.psychology.value(),
            critical_remaining: session.psychology.critical_remaining(),
            pacing: (director.intensity(), director.tally.cues),
            lantern: comfort.flashlight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Counts {
    entities: u32,
    meshes: usize,
    materials: usize,
    images: usize,
    labs: usize,
    models: usize,
}

impl Check {
    pub fn from_args() -> Self {
        let args = std::env::args_os().collect::<Vec<_>>();
        let report = args
            .windows(2)
            .find(|p| p[0] == "--smoke-report")
            .map(|p| PathBuf::from(&p[1]));
        Self {
            report,
            stress: args.iter().any(|arg| arg == "--lab-stress"),
            ..default()
        }
    }

    fn require(&mut self, condition: bool, invariant: &'static str) {
        if !condition && !self.failures.contains(&invariant) {
            error!("Dev Lab smoke: {invariant}");
            self.failures.push(invariant);
        }
    }
}

pub(super) fn inject_keys(check: Option<Res<Check>>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    let Some(check) = check else { return };
    let stress = check.stress && (STRESS_START..STRESS_END).contains(&check.frame);
    let phase = check.frame.saturating_sub(STRESS_START) % STRESS_PERIOD;
    match check.frame {
        frame if SWITCHES.contains(&frame) || stress && matches!(phase, 0 | 40) => {
            keys.press(KeyCode::ControlLeft);
            keys.press(KeyCode::KeyI);
        }
        frame
            if SWITCHES.contains(&frame.saturating_sub(1)) || stress && matches!(phase, 1 | 41) =>
        {
            keys.release(KeyCode::KeyI);
            keys.release(KeyCode::ControlLeft);
        }
        311 | 350 => keys.press(KeyCode::KeyW),
        335 | 368 => keys.release(KeyCode::KeyW),
        _ => {}
    }
}

struct Shot {
    name: &'static str,
    camera: Vec3,
    target: Vec3,
    fov: f32,
    clip: usize,
    lighting: Preset,
    detail: u8,
}

fn shots() -> [Shot; 16] {
    use Preset::*;
    let p = |x, y, z| Vec3::new(x, y, z);
    let middle = p(0.0, 1.14, 0.0);
    [
        Shot {
            name: "01-frente",
            camera: p(0., 1.38, 3.56),
            target: middle,
            fov: 45.,
            clip: 0,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "02-perfil",
            camera: p(3.54, 1.38, 0.03),
            target: middle,
            fov: 45.,
            clip: 0,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "03-costas",
            camera: p(0.10, 1.44, -3.54),
            target: middle,
            fov: 45.,
            clip: 0,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "04-rosto",
            camera: p(0.10, 2.115, 0.63),
            target: p(-0.019, 2.06, 0.137),
            fov: 40.,
            clip: 0,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "05-mao",
            camera: p(0.57, 0.96, 0.52),
            target: p(0.316, 0.86, 0.007),
            fov: 34.,
            clip: 0,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "06-cabeca-inclinada",
            camera: p(0.54, 1.63, 2.93),
            target: p(0.02, 1.42, 0.0),
            fov: 43.,
            clip: 1,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "07-espiada",
            camera: p(-3.3, 1.55, 1.53),
            target: p(-2.60, 1.41, -1.76),
            fov: 43.,
            clip: 2,
            lighting: Studio,
            detail: 1,
        },
        Shot {
            name: "08-quadrupede",
            camera: p(2.12, 0.89, 2.49),
            target: p(0.0, 0.54, 0.0),
            fov: 39.,
            clip: 3,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "09-despedida",
            camera: p(-0.83, 1.52, 3.43),
            target: middle,
            fov: 44.,
            clip: 4,
            lighting: Studio,
            detail: 0,
        },
        Shot {
            name: "10-esqueleto",
            camera: p(0.83, 1.36, 3.32),
            target: middle,
            fov: 46.,
            clip: 5,
            lighting: Studio,
            detail: 2,
        },
        Shot {
            name: "11-wireframe",
            camera: p(0.63, 1.42, 3.34),
            target: middle,
            fov: 44.,
            clip: 0,
            lighting: Studio,
            detail: 3,
        },
        Shot {
            name: "12-comparacao-escala",
            camera: p(0., 1.43, 4.41),
            target: p(0.0, 1.24, 0.),
            fov: 49.,
            clip: 0,
            lighting: Studio,
            detail: 4,
        },
        Shot {
            name: "13-luz-estacao",
            camera: p(0.72, 1.49, 3.52),
            target: middle,
            fov: 46.,
            clip: 0,
            lighting: Station,
            detail: 0,
        },
        Shot {
            name: "14-emergencia",
            camera: p(-0.49, 1.52, 3.50),
            target: middle,
            fov: 46.,
            clip: 0,
            lighting: Emergency,
            detail: 0,
        },
        Shot {
            name: "15-lanterna",
            camera: p(0.35, 1.45, 3.65),
            target: middle,
            fov: 45.,
            clip: 0,
            lighting: Torch,
            detail: 0,
        },
        Shot {
            name: "16-interface-dev-lab",
            camera: p(0.31, 1.43, 3.74),
            target: middle,
            fov: 44.,
            clip: 2,
            lighting: Studio,
            detail: 5,
        },
    ]
}

fn frame_shot(shot: &Shot, config: &mut Config, active: &mut Active) {
    config.hidden = shot.detail != 5;
    config.tab = Tab::Animation;
    config.skeleton = shot.detail == 2;
    config.wireframe = shot.detail == 3;
    config.grid = !matches!(shot.detail, 2 | 3);
    config.lighting = shot.lighting.into();
    config.instances = vec![ModelSpec {
        clip: shot.clip,
        position: if shot.detail == 1 {
            [-2.95, 0., -1.96]
        } else {
            [0.; 3]
        },
        ..default()
    }];
    if shot.detail == 4 {
        config.instances[0].position[0] = -1.24;
        config
            .instances
            .push(ModelSpec::new(Model::Legacy, Vec3::ZERO));
        config
            .instances
            .push(ModelSpec::new(Model::Human, Vec3::X * 1.22));
    }
    config.selected = 0;
    active.captured = false;
    active.orbit = None;
    active.playback = default();
    let look = (shot.target - shot.camera).normalize();
    config.lens.free = true;
    config.lens.position = shot.camera.into();
    config.lens.yaw = (-look.x).atan2(-look.z);
    config.lens.pitch = look.y.asin();
    config.lens.fov = shot.fov;
    match shot.name {
        "03-costas" => {
            config.lighting.azimuth = 150.0;
        }
        "04-rosto" | "05-mao" => {
            config.lighting.shadow = false;
        }
        _ => {}
    }
}

fn screenshot(dir: Option<&PathBuf>, commands: &mut Commands, name: &str) {
    if let Some(dir) = dir {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(dir.join(format!("{name}.png"))));
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct WorldData<'w, 's> {
    catalog: Res<'w, Catalog>,
    director: Res<'w, Director>,
    all: Query<'w, 's, Entity>,
    meshes: Res<'w, Assets<Mesh>>,
    materials: Res<'w, Assets<StandardMaterial>>,
    images: Res<'w, Assets<Image>>,
    cameras: Query<'w, 's, &'static Camera, With<LabCamera>>,
    models: Query<'w, 's, &'static Viewer>,
    skin: Query<'w, 's, &'static SkinnedMesh>,
    names: Query<'w, 's, (&'static Name, &'static GlobalTransform)>,
    guides: Query<'w, 's, (&'static Guide, &'static Visibility)>,
    animations: Query<'w, 's, (&'static AnimationPlayer, &'static library::Player)>,
    interface: Query<'w, 's, (&'static ComputedNode, &'static Node), With<Interface>>,
    station: Query<'w, 's, (), With<StationPreview>>,
    no_shadow: Query<'w, 's, (), (With<Part>, With<NotShadowCaster>)>,
}

impl WorldData<'_, '_> {
    fn counts(&self) -> Counts {
        Counts {
            entities: self.all.iter().len() as u32,
            meshes: self.meshes.len(),
            materials: self.materials.len(),
            images: self.images.len(),
            labs: self
                .cameras
                .iter()
                .filter(|camera| camera.is_active)
                .count(),
            models: self.models.iter().len(),
        }
    }

    fn head(&self) -> Option<f32> {
        self.names
            .iter()
            .filter(|(name, place)| name.as_str() == "cabeca" && place.translation().z.abs() < 2.0)
            .map(|(_, place)| place.translation().y)
            .next()
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Renderer, campaign and scene are observed independently."
)]
pub(super) fn verify(
    check: Option<ResMut<Check>>,
    options: Res<Options>,
    mut lab: ResMut<Lab>,
    mut config: ResMut<Config>,
    world: WorldData,
    mut commands: Commands,
    mut actions: MessageWriter<Action>,
    (mut session, mut player, mut comfort): (ResMut<Session>, ResMut<LocalPlayer>, ResMut<Comfort>),
    mut exit: MessageWriter<AppExit>,
) {
    let Some(mut check) = check else { return };
    let now = Instant::now();
    let wall_ms = check
        .previous_frame
        .replace(now)
        .map(|before| (now - before).as_secs_f64() * 1000.0);
    check.ticks += 1;
    if check.ticks == 1
        && let Some(dir) = &options.captures
        && std::fs::create_dir_all(dir).is_err()
    {
        check.require(false, "diretório de capturas");
    }
    let model = world.catalog.stats(Model::Presence);
    if model.is_none() && check.ticks < 800 {
        return;
    }
    if model.is_none() {
        check.require(false, "importação do GLB");
        check.frame = if check.stress {
            STRESS_END + 5
        } else {
            RETURN + 320
        };
    }
    check.frame += 1;
    let frame = check.frame;
    match frame {
        20 => {
            let stats = model.expect("GLB conferido");
            check.require(
                (8000..=20000).contains(&stats.triangles)
                    && stats.vertices > 8000
                    && stats.textures == 3
                    && stats.materials == 1
                    && stats.bones == 52
                    && stats.meters.is_some_and(|d| (2.17..2.23).contains(&d.y))
                    && world.catalog.clips().len() >= 7,
                "geometria, escala, rig, materiais, texturas e clipes",
            );
            check.require(
                [Model::Pump, Model::Door, Model::Structure, Model::Human]
                    .into_iter()
                    .all(|prop| {
                        world
                            .catalog
                            .stats(prop)
                            .is_some_and(|s| s.triangles > 0 && s.colliders > 0)
                    }),
                "biblioteca e colisores existentes",
            );
            check.require(
                world.catalog.stats(Model::Legacy).is_some_and(|legacy| {
                    legacy.triangles > 0 && legacy.meters.is_some_and(|extent| extent.y > 1.9)
                }),
                "recorte de comparação da figura anterior",
            );
            check.baseline = Some(world.counts());
        }
        66 => {
            check.require(
                lab.active.is_some()
                    && world.cameras.iter().len() == 1
                    && world.models.iter().len() == 1
                    && session.screen == Screen::Menu,
                "entrada no menu e instância única",
            );
            check.require(
                world.skin.iter().any(|skin| skin.joints.len() == 52)
                    && world
                        .animations
                        .iter()
                        .any(|(player, _)| player.playing_animations().next().is_some()),
                "cena com skin e player em Bevy",
            );
            check.require(
                world.interface.single().is_ok_and(|(computed, _)| {
                    computed.size().x > 300.0 && computed.size().y > 200.0
                }),
                "layout calculado para a interface",
            );
            check.first_pose_height = world.head();
            actions.write(Action::Add(Model::Pump));
        }
        70 => {
            check.require(
                world.models.iter().len() == 2,
                "instanciação de equipamento",
            );
            actions.write(Action::ModelBounds);
            actions.write(Action::ModelShadow);
        }
        73 => {
            check.require(
                world.guides.iter().any(|(g, visibility)| {
                    *g == Guide::Bounds && *visibility != Visibility::Hidden
                }),
                "colisores no visor",
            );
            check.require(world.no_shadow.iter().len() > 0, "comando de sombras");
            actions.write(Action::Duplicate);
        }
        76 => {
            check.require(world.models.iter().len() == 3, "duplicação");
            actions.write(Action::Remove);
        }
        79 => {
            check.require(world.models.iter().len() == 2, "remoção");
            actions.write(Action::Reset);
        }
        95 | 165 => {
            check.require(
                lab.active.is_none()
                    && session.screen == Screen::Menu
                    && world.counts().labs == 0
                    && world
                        .interface
                        .single()
                        .is_ok_and(|(_, node)| node.display == Display::None)
                    && world.models.iter().len() == 0,
                "retorno ao menu",
            );
        }
        104 if options.source.is_some() => {
            check.disk_revision = Some(world.catalog.revision());
            actions.write(Action::Reload);
        }
        205 if options.source.is_some() => {
            let reloaded = check
                .disk_revision
                .is_some_and(|revision| world.catalog.revision() > revision);
            check.require(
                reloaded && world.catalog.error.is_none(),
                "recarga explícita do GLB em disco",
            );
        }
        260 => {
            restart(&mut session, &mut player, &mut comfort);
            session.act(MissionAction::TakeFuse);
            session.act(MissionAction::InspectPump);
            session.psychology.set_for_development(100.0);
        }
        282 => {
            check.require(
                session.psychology.critical_remaining().is_some(),
                "crise ativa para o teste de suspensão",
            );
            check.restoration = Some(Snapshot::now(&session, &player, &comfort, &world.director));
        }
        305 => {
            if let Some(active) = &lab.active {
                check.first_person_start = active.walker.feet.z;
            }
            actions.write(Action::CaptureCursor);
        }
        339 => {
            let moved = lab.active.as_ref().is_some_and(|a| {
                a.walker.feet.z < check.first_person_start - 0.36 && a.walker.grounded
            });
            check.require(moved, "WASD e gravidade em primeira pessoa");
            actions.write(Action::FreeCamera);
        }
        345 => check.fly_start = config.lens.position[2],
        372 => {
            let moved = config.lens.free && config.lens.position[2] < check.fly_start - 0.55;
            check.require(moved, "Voo com WASD");
            actions.write(Action::CaptureCursor);
            actions.write(Action::Preset(Preset::Emergency));
            actions.write(Action::Adjust(Knob::Fov, -5.0));
            actions.write(Action::Adjust(Knob::Translation(0), 0.3));
            actions.write(Action::Save);
        }
        378 => {
            if let Some(path) = &options.settings {
                check.require(
                    Config::load(path).is_ok_and(|loaded| {
                        loaded == *config && loaded.lighting.preset == Preset::Emergency
                    }),
                    "preferências persistidas e relidas",
                );
            }
            actions.write(Action::Reset);
        }
        386 => {
            actions.write(Action::Clip(6));
            actions.write(Action::Play);
        }
        402 => {
            check.require(
                world
                    .animations
                    .iter()
                    .any(|(p, _)| p.playing_animations().any(|(_, a)| a.seek_time() > 0.10)),
                "reprodução do aceno",
            );
            actions.write(Action::Play);
            actions.write(Action::SeekFraction(0.5));
        }
        409 => {
            let duration = world
                .catalog
                .clips()
                .get(6)
                .map_or(0.0, |clip| clip.duration);
            check.require(
                duration >= 1.0
                    && world.animations.iter().any(|(p, _)| {
                        p.playing_animations().any(|(_, a)| {
                            a.is_paused() && (a.seek_time() - duration * 0.5).abs() < 0.01
                        })
                    }),
                "pausa e busca na linha do tempo",
            );
        }
        PHOTO_START..PHOTO_END => {
            let index = ((frame - PHOTO_START) / PHOTO_INTERVAL) as usize;
            let elapsed = (frame - PHOTO_START) % PHOTO_INTERVAL;
            if let Some(shot) = shots().get(index) {
                if elapsed == 0 {
                    if let Some(active) = &mut lab.active {
                        frame_shot(shot, &mut config, active);
                    } else {
                        check.require(false, "cena de captura ativa");
                    }
                } else if elapsed == 36 {
                    screenshot(options.captures.as_ref(), &mut commands, shot.name);
                    if shot.clip == 3 {
                        let head = world.head();
                        let applied = head
                            .zip(check.first_pose_height)
                            .is_some_and(|(here, initial)| initial - here > 0.55);
                        check.require(applied, "quadrúpede realmente aplicada à hierarquia");
                    }
                    if shot.detail == 2 {
                        check.require(
                            world.guides.iter().any(|(g, visibility)| {
                                *g == Guide::Bones && *visibility != Visibility::Hidden
                            }),
                            "linhas do esqueleto",
                        );
                    }
                    if shot.detail == 3 {
                        check.require(
                            world.guides.iter().any(|(g, visible)| {
                                *g == Guide::Wire && *visible != Visibility::Hidden
                            }),
                            "wireframe no recurso selecionado",
                        );
                    }
                }
                if index == 0
                    && (10..35).contains(&elapsed)
                    && let Some(ms) = wall_ms
                {
                    check.samples.push(ms);
                }
            }
        }
        frame if frame == RETURN - 6 => {
            let same = check.restoration.as_ref().is_some_and(|before| {
                *before == Snapshot::now(&session, &player, &comfort, &world.director)
            });
            check.require(
                same,
                "posição, orientação, fase, cronômetro e instabilidade congelados",
            );
        }
        frame if frame == RETURN + 8 => {
            let same = check.restoration.as_ref().is_some_and(|before| {
                before.position.distance(player.body.feet) < 0.01
                    && before.yaw == player.yaw
                    && before.progress == format!("{:?}", session.progress().phase())
                    && session.elapsed > before.elapsed
                    && before
                        .critical_remaining
                        .zip(session.psychology.critical_remaining())
                        .is_some_and(|(before, after)| after < before)
            });
            check.require(
                same && session.screen == Screen::Playing && lab.active.is_none(),
                "retomada da campanha",
            );
            player.body.feet = Vec3::new(0.0, 0.002, 7.25);
            player.body.previous_feet = player.body.feet;
            player.yaw = -0.11;
            player.pitch = -0.095;
            comfort.flashlight = true;
            session.psychology.set_for_development(0.0);
        }
        frame if frame == RETURN + 33 => {
            check.require(
                world.station.iter().len() == 1,
                "prévia experimental na Estação 07",
            );
            screenshot(
                options.captures.as_ref(),
                &mut commands,
                "17-estacao-07-campanha",
            );
        }
        frame if [63, 142, 221, 306].contains(&frame.saturating_sub(RETURN)) => {
            let now = world.counts();
            check.require(
                now.labs == 0 && now.models == 0,
                "ciclos devolvem todas as entidades do Lab",
            );
            check.occupancy.push(now);
        }
        frame if check.stress && frame == STRESS_START - 5 => {
            session.screen = Screen::Paused;
            info!("Dev Lab stress: início das {STRESS_CYCLES} alternâncias adicionais.");
        }
        STRESS_START..STRESS_END
            if check.stress && (frame - STRESS_START) % STRESS_PERIOD == 72 =>
        {
            let state = world.counts();
            check.require(
                state.labs == 0 && state.models == 0 && session.screen == Screen::Paused,
                "laboratório inativo nas pausas do teste prolongado",
            );
            check.stress_counts.push(state);
        }
        frame
            if frame
                >= if check.stress {
                    STRESS_END + 5
                } else {
                    RETURN + 316
                } =>
        {
            let clean = check.occupancy.windows(2).all(|pair| pair[0] == pair[1]);
            check.require(
                clean,
                "contagens de entidades e recursos estáveis após ciclos",
            );
            if let Some(dir) = &options.captures {
                let complete = shots()
                    .iter()
                    .map(|shot| shot.name)
                    .chain(["17-estacao-07-campanha"])
                    .all(|name| {
                        dir.join(format!("{name}.png"))
                            .metadata()
                            .is_ok_and(|file| file.len() > 1000)
                    });
                check.require(complete, "capturas PNG do renderizador");
            }
            if check.stress {
                let steady = check.stress_counts.len() == STRESS_CYCLES as usize
                    && check
                        .stress_counts
                        .windows(2)
                        .all(|pair| pair[0] == pair[1]);
                check.require(steady, "contagens estáveis no teste prolongado");
            }
            let success = check.failures.is_empty();
            check.samples.sort_by(f64::total_cmp);
            let view_ms = check.samples.get(check.samples.len() / 2).copied();
            let report = format!(
                "AINDA VIVO v{VERSION}\nDev Lab Windows: {}\nquadros do teste: {frame}\nquadros totais: {}\nGLB externo: {}\nestúdio (mediana ms, {} amostras): {:?}\ncontagem inicial: {:?}\ncontagens em campanha após saídas repetidas: {:?}\nverificações adicionais em pausa: {}, última: {:?}\nfalhas: {:?}\n",
                if success { "PASS" } else { "FAIL" },
                check.ticks,
                options.source.is_some(),
                check.samples.len(),
                view_ms,
                check.baseline,
                check.occupancy,
                check.stress_counts.len(),
                check.stress_counts.last(),
                check.failures
            );
            info!("{report}");
            if let Some(path) = &check.report
                && let Err(error) = std::fs::write(path, &report)
            {
                error!("{error}");
                exit.write(AppExit::error());
                return;
            }
            exit.write(if success {
                AppExit::Success
            } else {
                AppExit::error()
            });
        }
        _ => {}
    }
}
