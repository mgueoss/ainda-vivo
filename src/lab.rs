mod library;
mod motion;
mod motion_qa;
pub(crate) mod observer;
mod panel;
mod qa;
mod room;
mod sound_lab;
mod sound_qa;
pub(crate) use room::LabCamera;
mod settings;

use crate::{
    app::{Comfort, MainCamera},
    preferences::Preferences,
    scene::recording::ArchiveCamera,
};
use ainda_vivo_core::movement::PlayerBody;
use bevy::{
    asset::{
        AssetApp,
        io::{AssetSourceBuilder, embedded::EmbeddedAssetRegistry, file::FileAssetReader},
    },
    audio::AudioSinkPlayback,
    camera::visibility::VisibilitySystems,
    input::{
        InputSystems,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    },
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    transform::TransformSystems,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use room::{Room, walker};
use settings::{Config, Lens, Lighting, Model, ModelSpec, Preset, Tab};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

const EMBEDDED_PATH: &str = "ainda_vivo/presenca.glb";

pub(crate) fn cognition_cli() -> Option<bevy::app::AppExit> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if !args.iter().any(|arg| arg == "--check-cognition") {
        return None;
    }
    let value = |key| args.windows(2).find(|p| p[0] == key).map(|p| &p[1]);
    let seed = value("--ai-seed").and_then(|v| v.to_str()?.parse().ok());
    let path =
        value("--smoke-report").map_or_else(|| PathBuf::from("inteligencia.txt"), PathBuf::from);
    let (passed, report) = observer::scenario::matrix(seed);
    let content = format!("AINDA VIVO v{}\n{report}", crate::app::VERSION);
    let saved = std::fs::write(path, content).is_ok();
    Some(if passed && saved {
        bevy::app::AppExit::Success
    } else {
        bevy::app::AppExit::error()
    })
}

#[derive(Clone, Default, Resource)]
pub(super) struct Options {
    pub(super) enabled: bool,
    preview_station: bool,
    physical: bool,
    source: Option<PathBuf>,
    settings: Option<PathBuf>,
    smoke: bool,
    motion_smoke: bool,
    observer_smoke: bool,
    hunt_smoke: bool,
    sound_smoke: bool,
    cognitive_smoke: bool,
    ai_seed: Option<u32>,
    record_motion: bool,
    captures: Option<PathBuf>,
}

impl Options {
    pub(super) fn from_args() -> Self {
        let args: Vec<_> = std::env::args_os().collect();
        let has = |key| args.iter().any(|arg| arg == key);
        let value = |key| {
            args.windows(2)
                .find(|pair| pair[0] == key)
                .map(|pair| PathBuf::from(&pair[1]))
        };
        let motion_smoke = has("--smoke-motion");
        let observer_smoke = has("--smoke-observer");
        let hunt_smoke = has("--smoke-hunt");
        let sound_smoke = has("--smoke-sonoro");
        let cognitive_smoke = has("--smoke-inteligencia");
        let physical =
            has("--presence-physical") || observer_smoke || hunt_smoke || cognitive_smoke;
        let smoke = has("--smoke-lab")
            || motion_smoke
            || observer_smoke
            || hunt_smoke
            || sound_smoke
            || cognitive_smoke;
        let enabled = has("--dev-lab") || smoke || physical;
        let local =
            std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("AindaVivo/dev-lab"));
        Self {
            enabled,
            preview_station: enabled
                && !physical
                && (has("--smoke-lab") || has("--lab-campaign-preview")),
            physical,
            source: enabled.then(|| value("--lab-assets")).flatten(),
            settings: value("--lab-config").or_else(|| {
                (!smoke)
                    .then(|| local.as_ref().map(|p| p.join("settings.json")))
                    .flatten()
            }),
            captures: value("--capture-dir").or_else(|| {
                (!smoke)
                    .then(|| local.as_ref().map(|p| p.join("captures")))
                    .flatten()
            }),
            smoke,
            motion_smoke,
            observer_smoke,
            hunt_smoke,
            sound_smoke,
            cognitive_smoke,
            ai_seed: value("--ai-seed").and_then(|p| p.to_str()?.parse::<u32>().ok()),
            record_motion: motion_smoke && has("--lab-record-motion"),
        }
    }

    pub(super) fn configure_source(&self, app: &mut App) {
        if let Some(directory) = &self.source {
            let root = std::path::absolute(directory).unwrap_or_else(|_| directory.clone());
            app.register_asset_source(
                "lab",
                AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(&root))),
            );
        }
    }

    fn path(&self) -> &'static str {
        if self.source.is_some() {
            "lab://models/presenca.glb"
        } else {
            "embedded://ainda_vivo/presenca.glb"
        }
    }
}

pub(super) struct LabPlugin;

impl Plugin for LabPlugin {
    fn build(&self, app: &mut App) {
        let settings = app.world().resource::<Options>().settings.as_deref();
        let (config, status) = match settings {
            Some(path) if path.is_file() => match Config::load(path) {
                Ok(config) => (config, "Configuração local carregada.".into()),
                Err(error) => (
                    Config::default(),
                    format!("Configuração inválida. Padrões carregados. {error}"),
                ),
            },
            _ => (Config::default(), "Sala de inspeção pronta.".into()),
        };
        app.world()
            .resource::<EmbeddedAssetRegistry>()
            .insert_asset(
                PathBuf::from(file!()),
                Path::new(EMBEDDED_PATH),
                include_bytes!("../assets/models/presenca.glb").as_slice(),
            );
        let physical = app.world().resource::<Options>().physical;
        app.insert_resource(config)
            .insert_resource(Lab {
                status,
                ..default()
            })
            .init_resource::<library::Catalog>()
            .insert_resource(observer::Campaign::new(physical))
            .add_message::<Action>()
            .add_message::<motion::MotionEvent>()
            .add_systems(Startup, library::begin_import)
            .add_systems(
                PreUpdate,
                (
                    qa::inject_keys,
                    motion_qa::open,
                    observer::qa::keys,
                    observer::hunt_qa::keys,
                    sound_qa::keys,
                    observer::cognition_qa::keys,
                    switch.in_set(crate::app::BeforeCampaign),
                )
                    .chain()
                    .after(InputSystems),
            )
            .add_systems(FixedUpdate, room::walk.run_if(active))
            .add_systems(Update, library::finish_import)
            .add_systems(
                Update,
                observer::update_campaign
                    .after(crate::atmosphere::director::update)
                    .before(crate::scene::presence::update)
                    .run_if(campaign_running),
            )
            .add_systems(Update, library::animate_observer)
            .add_systems(
                Update,
                (
                    panel::input,
                    shortcuts,
                    actions,
                    camera_input,
                    observer::update_lab,
                    sound_lab::update,
                    library::animate,
                    library::sync,
                    library::remember_events,
                    room::present,
                    hold_campaign_audio,
                    panel::refresh,
                    panel::timeline_display,
                    cursor,
                )
                    .chain()
                    .run_if(active),
            )
            .add_systems(
                PostUpdate,
                (library::guides, library::wire_guides)
                    .chain()
                    .after(TransformSystems::Propagate)
                    .before(VisibilitySystems::VisibilityPropagate)
                    .run_if(active),
            )
            .add_systems(Last, (qa::verify, motion_qa::verify));
        let options = app.world().resource::<Options>();
        if options.cognitive_smoke {
            app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ))
            .init_resource::<observer::cognition_qa::Check>()
            .add_systems(Last, observer::cognition_qa::verify);
        } else if options.sound_smoke {
            app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ))
            .init_resource::<sound_qa::Check>()
            .add_systems(Last, sound_qa::verify);
        } else if options.hunt_smoke {
            app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ))
            .insert_resource(observer::hunt_qa::Check::new())
            .add_systems(Last, observer::hunt_qa::verify);
        } else if options.observer_smoke {
            app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ))
            .insert_resource(observer::qa::Check::new())
            .add_systems(Last, observer::qa::verify);
        } else if options.motion_smoke {
            app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ))
            .insert_resource(motion_qa::Check::from_args());
        } else if options.smoke {
            app.insert_resource(qa::Check::from_args());
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct Lab {
    active: Option<Active>,
    parked: Option<(Entity, Entity, Room)>,
    resume_hold: bool,
    search_focus: bool,
    status: String,
}

struct Active {
    root: Entity,
    interface: Entity,
    room: Room,
    suspended_cameras: Vec<(Entity, bool)>,
    resume_audio: HashSet<Entity>,
    captured: bool,
    look_armed: bool,
    jump: bool,
    walker: PlayerBody,
    eye: f32,
    orbit: Option<Orbit>,
    playback: Playback,
    observer: observer::TestLab,
    sound: sound_lab::SoundLab,
}

impl Lab {
    pub(crate) fn audio_geometry(
        &self,
    ) -> Option<(
        &[ainda_vivo_core::collision::Aabb],
        Option<ainda_vivo_core::collision::Aabb>,
    )> {
        let active = self.active.as_ref()?;
        Some((&active.room.solids, active.observer.solid()))
    }
    pub(crate) fn tune_sound(
        &self,
        request: &mut crate::atmosphere::sound::presence_audio::Request,
    ) {
        if let Some(active) = &self.active {
            active.sound.apply(request);
        }
    }
}

#[derive(Clone, Copy)]
struct Orbit {
    target: Vec3,
    radius: f32,
}

struct Playback {
    running: bool,
    repeat: bool,
    speed: f32,
    rewind: bool,
    seek: Option<f32>,
    step: i8,
    contacts: bool,
    displacement: bool,
    follow_gaze: bool,
    seat: bool,
    speed_scale: f32,
    offset: Vec3,
    run: Option<motion::Run>,
    last_event: Option<motion::EventKind>,
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            running: false,
            repeat: true,
            speed: 1.0,
            rewind: false,
            seek: None,
            step: 0,
            contacts: false,
            displacement: false,
            follow_gaze: false,
            seat: false,
            speed_scale: 1.0,
            offset: Vec3::ZERO,
            run: None,
            last_event: None,
        }
    }
}

impl Playback {
    fn cancel_run(&mut self) {
        self.offset = Vec3::ZERO;
        self.run = None;
        self.follow_gaze = false;
        self.seat = false;
        self.last_event = None;
    }
}

impl Active {
    fn reset_view(&mut self, lens: &Lens) {
        self.walker =
            walker(Vec3::from(lens.position).with_y((lens.position[1] - 1.62).max(0.002)));
        self.eye = 1.62;
        self.orbit = None;
        self.look_armed = false;
        self.jump = false;
    }

    fn camera_transform(&self, config: &Config) -> Transform {
        let lens = &config.lens;
        let orientation = Quat::from_rotation_y(lens.yaw) * Quat::from_rotation_x(lens.pitch);
        let translation = if let Some(orbit) = self.orbit {
            orbit.target + orientation * Vec3::Z * orbit.radius
        } else if lens.free {
            Vec3::from(lens.position)
        } else {
            self.walker.feet + Vec3::Y * self.eye
        };
        Transform::from_translation(translation).with_rotation(orientation)
    }

    fn set_free(&mut self, lens: &mut Lens, free: bool) {
        if free == lens.free {
            return;
        }
        lens.free = free;
        if free {
            lens.position = (self.walker.feet + Vec3::Y * self.eye).to_array();
        }
        if let Some(orbit) = self.orbit.take()
            && free
        {
            lens.position = (orbit.target
                + Quat::from_euler(EulerRot::YXZ, lens.yaw, lens.pitch, 0.0)
                    * Vec3::Z
                    * orbit.radius)
                .to_array();
        }
        self.look_armed = false;
    }
}

pub(super) fn campaign_running(lab: Option<Res<Lab>>) -> bool {
    lab.is_none_or(|lab| lab.active.is_none() && !lab.resume_hold)
}

fn active(lab: Res<Lab>) -> bool {
    lab.active.is_some()
}

fn ctrl(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
}

fn shift(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

fn chord_down(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([
        KeyCode::KeyI,
        KeyCode::KeyH,
        KeyCode::KeyG,
        KeyCode::KeyL,
        KeyCode::KeyB,
        KeyCode::KeyW,
        KeyCode::KeyP,
        KeyCode::KeyR,
        KeyCode::KeyF,
        KeyCode::KeyS,
    ])
}

type CampaignCameras<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Camera), Or<(With<MainCamera>, With<ArchiveCamera>)>>;
type LabCameraSwitch<'w, 's> = Query<
    'w,
    's,
    &'static mut Camera,
    (
        With<room::LabCamera>,
        Without<MainCamera>,
        Without<ArchiveCamera>,
    ),
>;

#[derive(bevy::ecs::system::SystemParam)]
struct Views<'w, 's> {
    campaign: CampaignCameras<'w, 's>,
    lab: LabCameraSwitch<'w, 's>,
    interface: Query<'w, 's, &'static mut Node, With<panel::Interface>>,
    models: Query<'w, 's, Entity, With<library::Viewer>>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Campaign camera and sink snapshots are distinct."
)]
fn switch(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    options: Res<Options>,
    config: Res<Config>,
    mut lab: ResMut<Lab>,
    mut views: Views,
    audio: Query<
        (Entity, &AudioSink),
        Without<crate::atmosphere::sound::presence_audio::PresenceEmitter>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if lab.resume_hold && !ctrl(&keys) && !keys.pressed(KeyCode::KeyI) {
        lab.resume_hold = false;
    }
    if lab.search_focus || !ctrl(&keys) || !keys.just_pressed(KeyCode::KeyI) {
        return;
    }
    if let Some(mut old) = lab.active.take() {
        old.observer.dispose(&mut commands);
        old.sound.dispose(&mut commands);
        for model in &views.models {
            commands.entity(model).despawn();
        }
        commands.entity(old.root).insert(Visibility::Hidden);
        if let Ok(mut camera) = views.lab.get_mut(old.room.camera) {
            camera.is_active = false;
        }
        if let Ok(mut node) = views.interface.get_mut(old.interface) {
            node.display = Display::None;
        }
        for (id, was_active) in old.suspended_cameras {
            if let Ok((_, mut camera)) = views.campaign.get_mut(id) {
                camera.is_active = was_active;
            }
        }
        for id in old.resume_audio {
            if let Ok((_, sink)) = audio.get(id) {
                sink.play();
            }
        }
        lab.parked = Some((old.root, old.interface, old.room));
        lab.resume_hold = !options.smoke;
        return;
    }
    let (root, interface, room) = if let Some((root, interface, room)) = lab.parked.take() {
        commands.entity(root).insert(Visibility::Inherited);
        if let Ok(mut camera) = views.lab.get_mut(room.camera) {
            camera.is_active = true;
        }
        (root, interface, room)
    } else {
        let root = commands
            .spawn((
                Name::new("Dev Lab"),
                Transform::default(),
                Visibility::Inherited,
            ))
            .id();
        let room = room::spawn(&mut commands, root, &mut meshes, &mut materials);
        let interface = panel::spawn(&mut commands, room.camera, &config);
        (root, interface, room)
    };
    let mut suspended_cameras = Vec::new();
    for (entity, mut camera) in &mut views.campaign {
        suspended_cameras.push((entity, camera.is_active));
        camera.is_active = false;
    }
    let resume_audio = audio
        .iter()
        .filter(|(_, sink)| !sink.is_paused())
        .map(|(entity, sink)| {
            sink.pause();
            entity
        })
        .collect();
    let mut current = Active {
        root,
        interface,
        room,
        suspended_cameras,
        resume_audio,
        captured: false,
        look_armed: false,
        jump: false,
        walker: PlayerBody::default(),
        eye: 1.62,
        orbit: None,
        playback: Playback::default(),
        observer: observer::TestLab::with_seed(options.ai_seed),
        sound: sound_lab::SoundLab::default(),
    };
    current.reset_view(&config.lens);
    lab.active = Some(current);
}

fn hold_campaign_audio(mut lab: ResMut<Lab>, audio: sound_lab::CampaignSinks) {
    if let Some(active) = &mut lab.active {
        for (id, sink) in &audio {
            if !sink.is_paused() {
                sink.pause();
                active.resume_audio.insert(id);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Knob {
    Translation(usize),
    Angles(usize),
    Scale,
    Key,
    Fill,
    Azimuth,
    Elevation,
    Exposure,
    Fov,
    Speed,
    Playback,
    Displacement,
}

#[derive(Clone, Copy, Debug, PartialEq, Message)]
enum Action {
    CaptureCursor,
    SetTab(Tab),
    Category,
    Search,
    Add(Model),
    Select(usize),
    Remove,
    Duplicate,
    Reload,
    Compare,
    ResetModel,
    CenterModel,
    Framing,
    Orbit,
    Face,
    Hands,
    Preset(Preset),
    CycleLights,
    Color,
    Fog,
    Shadows,
    Torch,
    Tape,
    Grid,
    Skeleton,
    Wireframe,
    Visible,
    ModelShadow,
    ModelBounds,
    Material,
    Clip(usize),
    CompareClips,
    MotionDemo(motion::Mode),
    StopDemo,
    Contacts,
    Displace,
    FollowGaze,
    Observer(observer::Command),
    Hunt(observer::HuntCommand),
    Sound(sound_lab::Command),
    Step(i8),
    Play,
    Repeat,
    Rewind,
    SeekFraction(f32),
    Adjust(Knob, f32),
    FreeCamera,
    Reset,
    Save,
    Screenshot,
    HideUi,
}

fn shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<Preferences>,
    options: Res<Options>,
    lab: Res<Lab>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut actions: MessageWriter<Action>,
) {
    if options.smoke || lab.search_focus || !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    if ctrl(&keys) {
        for (key, action) in [
            (KeyCode::KeyH, Action::HideUi),
            (KeyCode::KeyG, Action::Grid),
            (KeyCode::KeyL, Action::CycleLights),
            (KeyCode::KeyB, Action::Skeleton),
            (KeyCode::KeyW, Action::Wireframe),
            (KeyCode::KeyP, Action::Play),
            (KeyCode::KeyR, Action::Reset),
            (KeyCode::KeyF, Action::FreeCamera),
        ] {
            if keys.just_pressed(key) {
                actions.write(action);
            }
        }
        if shift(&keys) && keys.just_pressed(KeyCode::KeyS) {
            actions.write(Action::Screenshot);
        }
    } else if keys.just_pressed(KeyCode::Tab)
        || (keys.just_pressed(KeyCode::Escape) && lab.active.as_ref().is_some_and(|a| a.captured))
    {
        actions.write(Action::CaptureCursor);
    } else if lab.active.as_ref().is_some_and(|current| current.captured)
        && keys.just_pressed(bindings.key("flashlight", KeyCode::KeyF))
    {
        actions.write(Action::Torch);
    }
}

fn actions(
    mut actions: MessageReader<Action>,
    mut config: ResMut<Config>,
    mut lab: ResMut<Lab>,
    catalog: Res<library::Catalog>,
    assets: Res<AssetServer>,
    options: Res<Options>,
    mut commands: Commands,
) {
    for action in actions.read().copied() {
        let Some(active) = lab.active.as_mut() else {
            break;
        };
        match action {
            Action::CaptureCursor => {
                active.captured = !active.captured;
                active.look_armed = false;
                lab.search_focus = false;
            }
            Action::Search => {
                lab.search_focus = true;
            }
            Action::SetTab(tab) => {
                config.tab = tab;
            }
            Action::Category => {
                let next = (config.category as usize + 1) % settings::Category::ALL.len();
                config.category = settings::Category::ALL[next].0;
            }
            Action::Select(index) => {
                if index < config.instances.len() {
                    config.selected = index;
                    active.playback = Playback::default();
                }
            }
            Action::Remove => {
                config.remove();
                active.playback.cancel_run();
            }
            Action::Add(model) => {
                let spot = config.next_spot();
                if !config.append(ModelSpec::new(model, spot)) {
                    lab.status = format!("Limite de {} instâncias.", settings::MAX_INSTANCES);
                }
            }
            Action::Duplicate => {
                if let Some(mut copy) = config.selected().cloned() {
                    copy.position = config.next_spot().into();
                    if !config.append(copy) {
                        lab.status = "Limite de instâncias.".into();
                    }
                }
            }
            Action::Compare => {
                config.instances = [Model::Presence, Model::Legacy, Model::Human]
                    .into_iter()
                    .enumerate()
                    .map(|(i, model)| ModelSpec::new(model, Vec3::X * (i as f32 - 1.0) * 1.13))
                    .collect();
                config.selected = 0;
                config.lens = Lens::default();
                active.reset_view(&config.lens);
                active.playback = Playback::default();
            }
            Action::CompareClips => {
                if let Some(original) = config.selected().filter(|s| s.model == Model::Presence) {
                    let mut a = original.clone();
                    a.position = [-0.95, 0.0, 0.0];
                    a.visible = true;
                    let mut b = a.clone();
                    b.position[0] *= -1.0;
                    b.clip = if a.clip == 0 { 12 } else { 0 };
                    config.instances = vec![a, b];
                    config.selected = 0;
                    config.lens = Lens::default();
                    active.reset_view(&config.lens);
                    active.playback = Playback::default();
                    lab.status =
                        "A à esquerda, B à direita. Selecione cada instância e ajuste o clipe."
                            .into();
                }
            }
            Action::MotionDemo(mode) => {
                if !mode
                    .steps()
                    .iter()
                    .all(|&(id, _)| catalog.number(id).is_some())
                {
                    lab.status = "Os clipes da demonstração precisam estar carregados.".into();
                    continue;
                }
                let index = config.selected;
                let Some(actor) = config
                    .instances
                    .get_mut(index)
                    .filter(|s| s.model == Model::Presence)
                else {
                    lab.status = "Selecione a Presença para iniciar a demonstração.".into();
                    continue;
                };
                actor.visible = true;
                active.playback.cancel_run();
                active.playback.rewind = true;
                active.playback.running = true;
                active.playback.repeat = false;
                active.playback.run = Some(motion::Run::new(mode));
                config.lighting.vhs = 0;
                match mode {
                    motion::Mode::Approach => {
                        config.lens = Lens {
                            free: false,
                            position: [motion::APPROACH_X, 1.622, motion::APPROACH_VIEW_Z],
                            yaw: 0.0,
                            pitch: -0.21,
                            ..default()
                        };
                        active.reset_view(&config.lens);
                    }
                    motion::Mode::Track => {
                        config.lens = Lens {
                            free: true,
                            position: [-7.70, 1.85, -3.00],
                            yaw: -0.16,
                            pitch: -0.21,
                            ..default()
                        };
                        active.reset_view(&config.lens);
                    }
                    motion::Mode::Sequence => {}
                }
                lab.status = format!("{} em reprodução.", mode.label());
            }
            Action::StopDemo => {
                active.playback.cancel_run();
                active.playback.running = false;
            }
            Action::ResetModel => {
                let index = config.selected;
                if let Some(spec) = config.instances.get_mut(index) {
                    *spec = ModelSpec::new(spec.model, Vec3::ZERO);
                    active.playback = Playback::default();
                }
            }
            Action::CenterModel => {
                let index = config.selected;
                if let Some(spec) = config.instances.get_mut(index) {
                    spec.position = [0.; 3];
                }
            }
            Action::Reload => {
                if options.source.is_some() {
                    catalog.reload(&assets);
                    lab.status = "Recarregando o GLB externo.".into();
                } else {
                    lab.status = "GLB embutido. Use --lab-assets para recarregar do disco.".into();
                }
            }
            Action::Orbit | Action::Framing | Action::Face | Action::Hands => {
                if let Some(spec) = config.selected() {
                    let height = catalog.height(spec.model);
                    let mut target = Vec3::from(spec.position)
                        + Vec3::Y * (room::FLOOR + height * spec.scale * 0.51);
                    let mut radius = (height * spec.scale * 1.30).clamp(0.5, 16.0);
                    if action == Action::Face && spec.model == Model::Presence {
                        target = spec
                            .transform()
                            .transform_point(Vec3::new(-0.019, 2.053, 0.14))
                            + Vec3::Y * room::FLOOR;
                        radius = (0.55 * spec.scale).clamp(0.28, 4.0);
                    } else if action == Action::Hands && spec.model == Model::Presence {
                        target = spec
                            .transform()
                            .transform_point(Vec3::new(0.322, 0.855, 0.0))
                            + Vec3::Y * room::FLOOR;
                        radius = (0.56 * spec.scale).clamp(0.28, 4.0);
                    }
                    active.set_free(&mut config.lens, true);
                    config.lens.yaw = 0.0;
                    config.lens.pitch = if action == Action::Hands {
                        -0.09
                    } else {
                        -0.05
                    };
                    if action == Action::Orbit {
                        active.orbit = if active.orbit.is_some() {
                            None
                        } else {
                            Some(Orbit { target, radius })
                        };
                    } else {
                        active.orbit = None;
                    }
                    config.lens.position =
                        (target + Vec3::Z * radius + Vec3::Y * (radius * 0.05)).into();
                }
            }
            Action::Preset(preset) => config.lighting = preset.into(),
            Action::CycleLights => config.lighting = config.lighting.preset.next().into(),
            Action::Color => {
                config.lighting.color = (config.lighting.color + 1) % Lighting::COLORS.len()
            }
            Action::Fog => config.lighting.fog = !config.lighting.fog,
            Action::Shadows => config.lighting.shadow = !config.lighting.shadow,
            Action::Torch => config.lighting.torch = !config.lighting.torch,
            Action::Tape => config.lighting.vhs = (config.lighting.vhs + 1) % 3,
            Action::Grid => config.grid = !config.grid,
            Action::Skeleton => config.skeleton = !config.skeleton,
            Action::Contacts => active.playback.contacts = !active.playback.contacts,
            Action::Displace => {
                active.playback.displacement = !active.playback.displacement;
            }
            Action::Observer(command) => observer::command(active, &mut config, command),
            Action::Hunt(command) => observer::hunt_command(active, &mut config, command),
            Action::Sound(command) => sound_lab::command(active, &mut config, command),
            Action::FollowGaze => {
                if active.playback.follow_gaze {
                    active.playback.follow_gaze = false;
                    active.playback.running = false;
                } else if let Some(index) = catalog.number(12) {
                    let selected = config.selected;
                    if let Some(spec) = config
                        .instances
                        .get_mut(selected)
                        .filter(|spec| spec.model == Model::Presence)
                    {
                        spec.clip = index;
                        active.playback.cancel_run();
                        active.playback.follow_gaze = true;
                        active.playback.rewind = true;
                        active.playback.running = true;
                        active.playback.repeat = false;
                    }
                }
            }
            Action::Wireframe => config.wireframe = !config.wireframe,
            Action::HideUi => config.hidden = !config.hidden,
            Action::Visible
            | Action::ModelShadow
            | Action::ModelBounds
            | Action::Material
            | Action::Clip(_) => {
                let index = config.selected;
                if let Some(spec) = config.instances.get_mut(index) {
                    match action {
                        Action::Visible => spec.visible = !spec.visible,
                        Action::ModelShadow => spec.shadow = !spec.shadow,
                        Action::ModelBounds => spec.bounds = !spec.bounds,
                        Action::Material => spec.surface = spec.surface.next(),
                        Action::Clip(clip) => {
                            spec.clip = clip;
                            active.playback.cancel_run();
                            active.playback.rewind = true;
                            if let Some(clip) = catalog.clips().get(clip) {
                                active.playback.repeat = clip.motion.repeat;
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Action::Play => {
                active.playback.running = !active.playback.running;
                if active.playback.run.as_ref().is_some_and(|r| r.complete) {
                    active.playback.rewind = true;
                }
            }
            Action::Repeat => active.playback.repeat = !active.playback.repeat,
            Action::Rewind => {
                active.playback.rewind = true;
                active.playback.last_event = None;
            }
            Action::SeekFraction(fraction) => active.playback.seek = Some(fraction.clamp(0.0, 1.0)),
            Action::Step(frames) => {
                active.playback.running = false;
                active.playback.step = active.playback.step.saturating_add(frames);
            }
            Action::Adjust(knob, delta) => {
                let selected = config.selected;
                match knob {
                    Knob::Translation(axis) => {
                        if let Some(spec) = config.instances.get_mut(selected) {
                            spec.position[axis] += delta;
                        }
                    }
                    Knob::Angles(axis) => {
                        if let Some(spec) = config.instances.get_mut(selected) {
                            spec.angles[axis] += delta;
                        }
                    }
                    Knob::Scale => {
                        if let Some(spec) = config.instances.get_mut(selected) {
                            spec.scale += delta;
                        }
                    }
                    Knob::Key => config.lighting.key += delta,
                    Knob::Fill => config.lighting.fill += delta,
                    Knob::Azimuth => config.lighting.azimuth += delta,
                    Knob::Elevation => config.lighting.elevation += delta,
                    Knob::Exposure => config.lighting.exposure += delta,
                    Knob::Fov => config.lens.fov += delta,
                    Knob::Speed => config.lens.speed += delta,
                    Knob::Playback => {
                        active.playback.speed = (active.playback.speed + delta).clamp(0.1, 3.0);
                    }
                    Knob::Displacement => {
                        active.playback.speed_scale =
                            (active.playback.speed_scale + delta).clamp(0.0, 2.0);
                    }
                }
                config.validate();
            }
            Action::FreeCamera => {
                let next = !config.lens.free;
                active.set_free(&mut config.lens, next);
            }
            Action::Reset => {
                active.observer.dispose(&mut commands);
                active.observer.remove_obstacle(&mut active.room.solids);
                active.observer = observer::TestLab::with_seed(options.ai_seed);
                active.sound.dispose(&mut commands);
                active.sound = default();
                *config = Config::default();
                active.reset_view(&config.lens);
                active.playback = Playback::default();
                lab.status = "Padrões restaurados na cena. Salve para manter.".into();
            }
            Action::Save => match &options.settings {
                Some(path) => match config.save(path) {
                    Ok(()) => lab.status = "Preferências do Dev Lab salvas.".into(),
                    Err(error) => lab.status = format!("Falha ao salvar: {error}"),
                },
                None => lab.status = "Caminho de preferências indisponível.".into(),
            },
            Action::Screenshot => {
                if let Some(directory) = &options.captures {
                    match std::fs::create_dir_all(directory) {
                        Ok(()) => {
                            let id = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis();
                            let path = directory.join(format!("estacao-07-dev-lab-{id}.png"));
                            commands
                                .spawn(Screenshot::primary_window())
                                .observe(save_to_disk(path.clone()));
                            lab.status = format!("Captura solicitada: {}", path.display());
                        }
                        Err(error) => lab.status = format!("Falha ao preparar a captura: {error}"),
                    }
                }
            }
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy injects camera and independent input resources."
)]
fn camera_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    wheel: Res<AccumulatedMouseScroll>,
    comfort: Res<Comfort>,
    bindings: Res<Preferences>,
    (mut config, mut lab): (ResMut<Config>, ResMut<Lab>),
    windows: Query<&Window, With<PrimaryWindow>>,
    options: Res<Options>,
) {
    let Some(active) = lab.active.as_mut() else {
        return;
    };
    if !options.smoke && !windows.single().is_ok_and(|window| window.focused) {
        active.captured = false;
        active.look_armed = false;
        return;
    }
    if !active.captured || ctrl(&keys) && chord_down(&keys) {
        return;
    }
    if active.look_armed && mouse.delta != Vec2::ZERO {
        let sensitivity = comfort.sensitivity;
        config.lens.yaw =
            (config.lens.yaw - mouse.delta.x * sensitivity).rem_euclid(std::f32::consts::TAU);
        config.lens.pitch = (config.lens.pitch - mouse.delta.y * sensitivity).clamp(-1.52, 1.52);
    }
    active.look_armed = true;
    if let Some(orbit) = &mut active.orbit {
        orbit.radius = (orbit.radius * (-wheel.delta.y * 0.12).exp()).clamp(0.19, 19.0);
        let rotation =
            Quat::from_rotation_y(config.lens.yaw) * Quat::from_rotation_x(config.lens.pitch);
        config.lens.position = (orbit.target + rotation * Vec3::Z * orbit.radius).into();
        return;
    }
    let movement = crate::desktop::movement_input(&keys, &bindings);
    if !config.lens.free {
        active.jump |= movement.jump;
        if active
            .walker
            .feet
            .distance_squared(Vec3::from(config.lens.position).with_y(active.walker.feet.y))
            > 1e-8
        {
            config.lens.position = (active.walker.feet + Vec3::Y * 1.62).into();
        }
        return;
    }
    let vertical = i8::from(keys.pressed(KeyCode::Space)) - i8::from(ctrl(&keys));
    let rotation = Quat::from_euler(EulerRot::YXZ, config.lens.yaw, config.lens.pitch, 0.0);
    let direction = (rotation * Vec3::new(movement.movement.x, 0., -movement.movement.y)
        + Vec3::Y * f32::from(vertical))
    .clamp_length_max(1.0);
    if direction != Vec3::ZERO {
        let speed = config.lens.speed * if shift(&keys) { 2.2 } else { 1.0 };
        config.lens.position = (Vec3::from(config.lens.position)
            + direction * speed * time.delta_secs())
        .clamp(Vec3::new(-30., -3., -29.), Vec3::new(30., 20., 28.))
        .into();
    }
}

fn cursor(
    lab: Res<Lab>,
    options: Res<Options>,
    mut windows: Query<(&mut CursorOptions, &Window), With<PrimaryWindow>>,
) {
    if let Some(active) = &lab.active
        && let Ok((mut cursor, window)) = windows.single_mut()
    {
        let captured = active.captured && window.focused && !options.smoke;
        let mode = if captured {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        if cursor.grab_mode != mode {
            cursor.grab_mode = mode;
        }
        if cursor.visible == captured {
            cursor.visible = !captured;
        }
    }
}
