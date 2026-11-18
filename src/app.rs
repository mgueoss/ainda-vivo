use ainda_vivo_core::{
    collision::Aabb, movement::PlayerBody, movement::PlayerCommand, story::Progress,
};
use bevy::{
    audio::AddAudioSource, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, window::PresentMode,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Playing,
    Paused,
    Complete,
}

#[derive(Resource)]
pub struct Session {
    pub screen: Screen,
    pub attempt: u64,
    pub progress: Progress,
    pub elapsed: f32,
    pub message: &'static str,
    pub message_left: f32,
    pub prompt: String,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            attempt: 0,
            progress: Progress::default(),
            elapsed: 0.0,
            message: "",
            message_left: 0.0,
            prompt: String::new(),
        }
    }
}

impl Session {
    pub fn say(&mut self, message: &'static str) {
        self.message = message;
        self.message_left = 8.0;
        info!("Registro: {message}");
    }
}

#[derive(Resource)]
pub struct Comfort {
    pub sensitivity: f32,
    pub fov_degrees: f32,
    pub bodycam: bool,
    pub flashlight: bool,
    pub vsync: bool,
    pub diagnostics: bool,
    pub ambience_volume: f32,
    pub ambience_muted: bool,
}

impl Default for Comfort {
    fn default() -> Self {
        Self {
            sensitivity: 0.0018,
            fov_degrees: 65.0,
            bodycam: false,
            flashlight: true,
            vsync: true,
            diagnostics: true,
            ambience_volume: 0.65,
            ambience_muted: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct LocalPlayer {
    pub body: PlayerBody,
    pub command: PlayerCommand,
    pub yaw: f32,
    pub pitch: f32,
    pub eye_height: f32,
    pub look_armed: bool,
}

#[derive(Clone, Copy)]
pub struct Solid {
    pub bounds: Aabb,
    pub gate: bool,
}

#[derive(Resource, Default)]
pub struct CollisionWorld(pub Vec<Solid>);

impl CollisionWorld {
    pub fn active(&self, power: bool) -> Vec<Aabb> {
        self.0
            .iter()
            .filter(|s| !s.gate || !power)
            .map(|s| s.bounds)
            .collect()
    }
}

#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct Flashlight;

pub fn restart(session: &mut Session, player: &mut LocalPlayer, comfort: &mut Comfort) {
    *session = Session {
        screen: Screen::Playing,
        attempt: session.attempt.wrapping_add(1),
        ..default()
    };
    *player = LocalPlayer {
        eye_height: 1.62,
        ..default()
    };
    comfort.flashlight = true;
    session.say("Registro iniciado. Encontre o fusível na sala à esquerda.");
}

pub fn run() -> bevy::app::AppExit {
    let smoke = crate::smoke::SmokeTest::from_args();
    info!("AINDA VIVO {VERSION}");
    let (preferences, mut comfort) = crate::preferences::Preferences::load(smoke.enabled);
    if smoke.enabled && !smoke.check_audio {
        comfort.ambience_muted = true;
    }
    let mut app = App::new();
    if smoke.enabled {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
    }
    app.insert_resource(ClearColor(Color::srgb(0.008, 0.012, 0.014)))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Time::<Virtual>::from_max_delta(
            std::time::Duration::from_millis(100),
        ))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.74, 0.82, 0.83),
            brightness: 68.0,
            ..default()
        })
        .insert_resource(LocalPlayer {
            eye_height: 1.62,
            ..default()
        })
        .insert_resource(smoke)
        .init_resource::<Session>()
        .insert_resource(comfort)
        .insert_resource(preferences)
        .init_resource::<CollisionWorld>()
        .init_resource::<crate::atmosphere::Atmosphere>()
        .init_resource::<crate::diagnostics::Performance>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("AINDA VIVO | v{VERSION}"),
                resolution: (1280, 720).into(),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_audio_source::<crate::atmosphere::sound::Voice>()
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(
            Startup,
            (
                crate::scene::setup,
                crate::ui::setup,
                crate::atmosphere::sound::setup,
            ),
        )
        .add_systems(FixedUpdate, crate::desktop::simulate)
        .add_systems(
            Update,
            (
                crate::ui::buttons,
                crate::desktop::read_input,
                crate::smoke::drive,
                crate::desktop::update_camera,
                crate::interaction::interact,
                crate::scene::sync_scene,
                crate::atmosphere::update,
                crate::atmosphere::sound::emit,
                crate::atmosphere::sound::mix,
                crate::diagnostics::update,
                crate::ui::refresh,
                crate::desktop::sync_cursor,
                crate::preferences::save,
            )
                .chain(),
        )
        .run()
}
