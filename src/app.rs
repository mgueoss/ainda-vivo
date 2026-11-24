use ainda_vivo_core::{
    collision::Aabb,
    memory::{FieldMemory, ServiceTrail},
    movement::{PlayerBody, PlayerCommand},
    psychology::{Origin, Psychology, Stimulus},
    reality::{Awakening, Expedition, Outcome, SafePoint},
    story::{Action, Progress},
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
    pub expedition: Expedition,
    pub perception_revision: u32,
    pub wake_confirmation: bool,
    pub elapsed: f32,
    pub message: &'static str,
    pub message_left: f32,
    pub prompt: String,
    pub false_exit: Option<ExitPresentation>,
    pub psychology: Psychology,
    pub resting: bool,
    pub farewell: Option<f32>,
    pub critical_hint: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExitPresentation {
    Receipt { reveal_at: f32 },
    Contradiction,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            attempt: 0,
            expedition: Expedition::default(),
            perception_revision: 0,
            wake_confirmation: false,
            elapsed: 0.0,
            message: "",
            message_left: 0.0,
            prompt: String::new(),
            false_exit: None,
            psychology: Psychology::default(),
            resting: false,
            farewell: None,
            critical_hint: false,
        }
    }
}

impl Session {
    pub fn collapse_active(&self) -> bool {
        self.farewell.is_some()
    }

    pub fn collapse_finished(&self) -> bool {
        self.collapse_active() && self.screen == Screen::Complete
    }

    pub fn stimulate(&mut self, stimulus: Stimulus, origin: Origin, strength: f32) {
        if self.screen == Screen::Playing && !self.collapse_active() {
            let key = (u64::from(self.elapsed.to_bits()) << 8) | stimulus as u64;
            self.psychology.observe(key, stimulus, origin, strength);
        }
    }

    pub fn progress(&self) -> &Progress {
        self.expedition.view()
    }

    pub fn memory(&self) -> &FieldMemory {
        &self.expedition.observer().memory
    }

    pub fn memory_mut(&mut self) -> &mut FieldMemory {
        self.expedition.memory_mut()
    }

    pub fn act(&mut self, action: Action) {
        if self.collapse_active() {
            return;
        }
        if action == Action::RestAtPost {
            self.resting = true;
            let found = self.psychology.grant_reserve();
            self.say(if found {
                "Posto de pausa. Descanse sob a luz. Reserva de foco disponível com [R]."
            } else {
                "Posto de pausa. Permaneça aqui para recuperar o fôlego."
            });
            return;
        }
        let mut message = self
            .expedition
            .act(action, crate::scene::station::RETURN_POST);
        let now = self.elapsed;
        let can_review = self.progress().recording_reviewed();
        let memory = self.memory_mut();
        memory.note_action(action, now);
        match action {
            Action::CheckRelay
                if matches!(
                    memory.service,
                    ServiceTrail::Checked(_) | ServiceTrail::Reconciled
                ) =>
            {
                message = "Via do técnico, 03:19. Serviço concluído. A assinatura de Borges ainda borra. Conferência indicada na Câmera 02.";
            }
            Action::ReviewRecording if can_review && memory.reconcile(now) => {
                message = "Câmera 02, 03:17. A via já estava assinada na bancada. O relé ainda constava lacrado.";
            }
            _ => (),
        }
        if message.is_empty() {
            return;
        }
        self.say(message);
    }

    pub fn objective(&self) -> &'static str {
        match self.false_exit {
            Some(ExitPresentation::Receipt { .. }) => {
                "Vistoria concluída. Registro de saída recebido."
            }
            Some(ExitPresentation::Contradiction) => {
                "Ordem 07 pendente. O corredor permanece aberto."
            }
            None => self.progress().objective(),
        }
    }

    pub fn cross_exit(&mut self) {
        match self.expedition.cross_exit() {
            Some(Outcome::Escaped) => {
                self.screen = Screen::Complete;
                info!("Expedição concluída em {:.1}s", self.elapsed);
            }
            Some(Outcome::PerceivedExit) => {
                let linked = self.memory().service == ServiceTrail::Reconciled;
                self.false_exit = Some(ExitPresentation::Receipt {
                    reveal_at: self.elapsed + if linked { 5.5 } else { 4.0 },
                });
                self.say("Portaria: baixa recebida. Pode seguir.");
            }
            _ => (),
        }
    }

    pub fn age_exit_receipt(&mut self) {
        if matches!(self.false_exit, Some(ExitPresentation::Receipt { reveal_at }) if self.elapsed >= reveal_at)
        {
            self.false_exit = Some(ExitPresentation::Contradiction);
            self.stimulate(Stimulus::Hallucination, Origin::Presence, 0.66);
            let message = if self.memory().service == ServiceTrail::Reconciled {
                "Portaria, 03:17. A baixa foi emitida antes da entrada. A via de Borges continua em serviço."
            } else {
                "Portaria: Ordem 07. O seu turno ainda não começou."
            };
            self.say(message);
        }
    }

    pub fn say(&mut self, message: &'static str) {
        self.message = message;
        self.message_left = 8.0;
        info!("Registro: {message}");
    }
}

pub fn awaken(session: &mut Session, player: &mut LocalPlayer) {
    if session.screen != Screen::Paused || !session.wake_confirmation || session.collapse_active() {
        return;
    }
    session.wake_confirmation = false;
    match session.expedition.awaken() {
        Awakening::Recovered(post) => {
            session.resting = false;
            *player = player_at_post(post);
            session.screen = Screen::Playing;
            session.perception_revision += 1;
            session.false_exit = None;
            session.prompt.clear();
            session.say("Registro retomado no posto da Elétrica.");
        }
        Awakening::Interrupted => {
            session.screen = Screen::Complete;
            player.command = default();
            player.look_armed = false;
        }
        Awakening::Unavailable => (),
    }
}

fn player_at_post(post: SafePoint) -> LocalPlayer {
    LocalPlayer {
        body: PlayerBody {
            feet: post.feet,
            previous_feet: post.feet,
            ..default()
        },
        yaw: post.yaw,
        pitch: post.pitch,
        eye_height: 1.62,
        ..default()
    }
}

pub fn continue_or_restart(session: &mut Session, player: &mut LocalPlayer, comfort: &mut Comfort) {
    if let Some((expedition, post)) = session.expedition.resume_interrupted() {
        *session = Session {
            screen: Screen::Playing,
            attempt: session.attempt.wrapping_add(1),
            expedition,
            ..default()
        };
        *player = player_at_post(post);
        session.say("Nova vistoria retomada no posto da Elétrica.");
    } else {
        restart(session, player, comfort);
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
    pub vhs: crate::vhs::VhsMode,
    pub psycho_av: f32,
    pub psycho_motion: bool,
    pub psycho_indicator: bool,
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
            vhs: crate::vhs::VhsMode::Soft,
            psycho_av: 1.0,
            psycho_motion: true,
            psycho_indicator: false,
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
    pub fn active(&self, gate_open: bool) -> impl Iterator<Item = Aabb> + '_ {
        self.0
            .iter()
            .filter(move |s| !s.gate || !gate_open)
            .map(|s| s.bounds)
    }

    pub fn occludes(&self, origin: Vec3, target: Vec3, gate_open: bool) -> bool {
        let offset = target - origin;
        let distance = offset.length() - 0.07;
        let direction = offset.normalize_or_zero();
        distance > 0.0
            && self
                .0
                .iter()
                .filter(|solid| !solid.gate || !gate_open)
                .any(|solid| solid.bounds.ray(origin, direction, distance).is_some())
    }
}

#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct Flashlight;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub struct BeforeCampaign;

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
    session.say("Vistoria 07 iniciada.");
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
    #[cfg(feature = "dev-lab")]
    let lab = crate::lab::Options::from_args();
    #[cfg(feature = "dev-lab")]
    {
        lab.configure_source(&mut app);
        app.insert_resource(lab.clone());
    }

    app.insert_resource(ClearColor(crate::scene::DISTANCE_DARK))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Time::<Virtual>::from_max_delta(
            std::time::Duration::from_millis(100),
        ))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.74, 0.82, 0.83),
            brightness: 25.0,
            ..default()
        })
        .insert_resource(LocalPlayer {
            eye_height: 1.62,
            ..default()
        })
        .insert_resource(smoke)
        .insert_resource(crate::reality::Developer::from_args())
        .init_resource::<Session>()
        .insert_resource(comfort)
        .insert_resource(preferences)
        .init_resource::<CollisionWorld>()
        .init_resource::<crate::atmosphere::Atmosphere>()
        .init_resource::<crate::atmosphere::director::Director>()
        .init_resource::<crate::scene::presence::Presence>()
        .init_resource::<crate::scene::illusion::Illusions>()
        .init_resource::<crate::reality::Threat>()
        .init_resource::<crate::diagnostics::Performance>()
        .init_resource::<crate::smoke::continuity::ContinuitySmoke>()
        .init_resource::<crate::smoke::echoes::EchoSmoke>()
        .init_resource::<crate::ui::Menu>()
        .init_resource::<crate::psychology::Effects>()
        .init_resource::<crate::psychology::ForceCue>()
        .init_resource::<crate::smoke::collapse::Check>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("AINDA VIVO | v{VERSION}"),
                resolution: (1280, 720).into(),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                resize_constraints: bevy::window::WindowResizeConstraints {
                    min_width: 560.0,
                    min_height: 430.0,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_audio_source::<crate::atmosphere::sound::Voice>()
        .add_plugins(crate::vhs::VhsPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(
            Startup,
            (
                crate::scene::setup,
                crate::scene::presence::setup,
                crate::scene::illusion::setup,
                crate::scene::recording::setup,
                crate::scene::water::setup,
                crate::ui::setup,
                crate::atmosphere::sound::setup,
                crate::scene::station::setup_rest_posts,
                crate::psychology::presentation::setup,
            ),
        )
        .add_systems(
            PreUpdate,
            (
                crate::smoke::collapse::inject_reserve,
                crate::psychology::reserve,
            )
                .chain()
                .after(bevy::input::InputSystems)
                .after(BeforeCampaign)
                .run_if(campaign_running),
        )
        .add_systems(
            FixedUpdate,
            crate::desktop::simulate.run_if(campaign_running),
        )
        .add_systems(
            Update,
            (
                (crate::smoke::menu::drive, crate::ui::menu_input).chain(),
                crate::desktop::read_input,
                (
                    crate::smoke::drive,
                    crate::smoke::continuity::drive,
                    crate::smoke::echoes::drive,
                    crate::smoke::collapse::drive,
                )
                    .chain(),
                (
                    crate::psychology::input,
                    crate::desktop::update_camera,
                    crate::psychology::perception,
                    crate::psychology::presentation::update,
                    crate::psychology::presentation::update_figure,
                    crate::psychology::presentation::update_audio,
                )
                    .chain(),
                crate::vhs::update,
                crate::interaction::interact,
                crate::scene::sync_scene,
                crate::atmosphere::update,
                crate::scene::water::update,
                crate::scene::recording::update,
                crate::scene::station::animate,
                (
                    crate::atmosphere::director::update,
                    crate::scene::presence::update,
                    crate::reality::update,
                    crate::reality::tools,
                    crate::scene::illusion::update,
                )
                    .chain(),
                crate::atmosphere::sound::emit,
                crate::atmosphere::sound::emit_incidents,
                crate::atmosphere::sound::mix,
                crate::diagnostics::update,
                (crate::ui::refresh_menu, crate::ui::refresh_hud).chain(),
                crate::desktop::sync_cursor,
                crate::preferences::save,
            )
                .chain()
                .run_if(campaign_running),
        );
    #[cfg(feature = "dev-lab")]
    if lab.enabled {
        app.add_plugins(crate::lab::LabPlugin);
    }
    app.run()
}

#[cfg(feature = "dev-lab")]
use crate::lab::campaign_running;

#[cfg(not(feature = "dev-lab"))]
fn campaign_running() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::station::RETURN_POST;

    #[test]
    fn the_reconciled_receipt_delays_its_contradiction_without_changing_mission_truth() {
        let mut session = Session {
            screen: Screen::Playing,
            ..default()
        };
        for action in [Action::TakeFuse, Action::InspectPump, Action::RestorePower] {
            session.act(action);
        }
        session.elapsed = 20.0;
        assert!(session.expedition.capture(session.elapsed));
        session.act(Action::InspectPump);
        session.memory_mut().service = ServiceTrail::Reconciled;
        session.cross_exit();
        assert_eq!(session.screen, Screen::Playing);
        assert!(!session.expedition.truth().exit_unlocked());
        session.elapsed = 25.49;
        session.age_exit_receipt();
        assert!(matches!(
            session.false_exit,
            Some(ExitPresentation::Receipt { .. })
        ));
        session.elapsed = 25.5;
        session.age_exit_receipt();
        assert_eq!(session.false_exit, Some(ExitPresentation::Contradiction));
        assert!(session.message.contains("03:17"));

        let mut player = LocalPlayer::default();
        session.screen = Screen::Paused;
        crate::ui::request_awakening(&mut session);
        awaken(&mut session, &mut player);
        assert_eq!(player.body.feet, RETURN_POST.feet);
        assert_eq!(session.memory().service, ServiceTrail::Reconciled);
        assert!(session.false_exit.is_none());
        assert!(!session.expedition.can_capture());
    }
}
