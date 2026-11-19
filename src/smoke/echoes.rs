//! Windows integration route for observer memory, the OS 17 and the post.
use super::{SmokeTest, follow, place, turn_to};
use crate::{
    app::*,
    atmosphere::{Circuit, CircuitLight, director::Director, sound::Emitter},
    scene::{
        illusion::{FILE, Impression},
        recording::{Recording, TERMINAL},
        station::{DESK_SWITCH, PANEL, PUMP_02_PANEL, RETURN_POST, SERVICE_ORDER, SERVICE_RELAY},
    },
    ui,
};
use ainda_vivo_core::{
    memory::{Prop, ServiceTrail},
    reality::{Outcome, Reality},
    story::{Action, Phase},
};
use bevy::{app::AppExit, audio::AudioSinkPlayback, ecs::system::SystemParam, prelude::*};
use std::{f32::consts::PI, time::Instant};

const RELAY_VIEW: Vec3 = Vec3::new(32.85, 0.002, SERVICE_RELAY.z);
const TERMINAL_VIEW: Vec3 = Vec3::new(TERMINAL.x, 0.002, -81.42);
const WALK: &[Vec2] = &[Vec2::new(29.60, -50.80)];
const EXPECTED_CAPTURES: u32 = 6;

#[derive(Debug, Default)]
enum Stage {
    #[default]
    Boot,
    Order,
    Lamp,
    Folder,
    Rearm,
    RelayOriginal,
    Walk,
    Echo,
    WaitForRelay,
    CheckRelay,
    OriginalFrame,
    LaterFrame,
    ReadPump,
    Exit,
    Interrupt,
    Resume,
    Finish,
}

#[derive(Resource, Default)]
pub struct EchoSmoke {
    stage: Stage,
    frame: u32,
    start: u32,
    waypoint: usize,
    footsteps: Vec<Entity>,
    measured: Vec<f64>,
    last_frame: Option<Instant>,
}

impl EchoSmoke {
    fn next(&mut self, stage: Stage) {
        debug!("Rota da OS 17: {stage:?} no quadro {}", self.frame);
        self.stage = stage;
        self.start = self.frame;
    }

    fn sample_frame(&mut self) {
        let now = Instant::now();
        if let Some(before) = self.last_frame.replace(now) {
            self.measured.push((now - before).as_secs_f64() * 1000.0);
        }
    }

    fn report(&mut self, smoke: &mut SmokeTest, seed: u32, mut exit: MessageWriter<AppExit>) {
        if smoke.capture_dir.is_some() && smoke.captures != EXPECTED_CAPTURES {
            smoke.failures.push("capturas_da_os_17");
        }
        if smoke.check_audio && self.footsteps.len() != 3 {
            smoke.failures.push("tres_passos_espaciais_da_os_17");
        }
        let passed = smoke.failures.is_empty();
        self.measured.sort_unstable_by(f64::total_cmp);
        let n = self.measured.len();
        let median = self.measured.get(n / 2).copied().unwrap_or_default();
        let p95 = self.measured.get(n * 95 / 100).copied().unwrap_or_default();
        let report = format!(
            "AINDA VIVO v{VERSION}\nsmoke-echoes: {}\nframes: {}\nseed: {seed}\nroute: Archive (order, switched lamp), workshop (known folder), Electrical (real rearm, physical footsteps, relay, relocated folder), Control (two native camera frames), pump, exit, post retry\nchecks: elapsed gates; real observer throughout chain; optional chain preserves truth and does not lock the gate; one post retry with clean perception\nrequested audio playback check: {}\nspatial echo sinks heard: {} of 3\nBMP captures: {}\nwalk-and-wait wall frame intervals: {n} samples, median {median:.2} ms, p95 {p95:.2} ms\nfailures: {:?}\n",
            if passed { "PASS" } else { "FAIL" },
            self.frame,
            smoke.check_audio,
            self.footsteps.len(),
            smoke.captures,
            smoke.failures,
        );
        info!("{report}");
        let written = smoke.report.as_ref().is_none_or(|path| {
            std::fs::write(path, report)
                .map_err(|error| error!("Falha ao escrever relatório: {error}"))
                .is_ok()
        });
        exit.write(if passed && written {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}

#[derive(SystemParam)]
pub struct Views<'w, 's> {
    director: Res<'w, Director>,
    recording: Res<'w, Recording>,
    props: Query<'w, 's, (&'static Impression, &'static Transform)>,
    lights: Query<'w, 's, (&'static CircuitLight, &'static SpotLight)>,
    voices: Query<'w, 's, (Entity, &'static Emitter, &'static AudioSink)>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy integration system parameters."
)]
pub fn drive(
    mut smoke: ResMut<SmokeTest>,
    mut test: ResMut<EchoSmoke>,
    mut session: ResMut<Session>,
    mut player: ResMut<LocalPlayer>,
    mut comfort: ResMut<Comfort>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    views: Views,
    mut commands: Commands,
    exit: MessageWriter<AppExit>,
) {
    if !smoke.echoes {
        return;
    }
    test.frame += 1;
    let age = test.frame - test.start;
    if matches!(test.stage, Stage::Walk | Stage::Echo) {
        test.sample_frame();
    }
    if smoke.check_audio {
        for (entity, emitter, sink) in &views.voices {
            if emitter.service_echo()
                && !sink.is_paused()
                && !sink.position().is_zero()
                && !test.footsteps.contains(&entity)
            {
                test.footsteps.push(entity);
            }
        }
    }
    if age > 2250 || test.frame > 5300 {
        error!(
            "Etapa {:?}, {:.2}s, {:?}, {:?}, interação {:?}",
            test.stage,
            session.elapsed,
            player.body.feet,
            session.memory().service,
            session.prompt
        );
        smoke.failures.push("tempo_excedido_na_os_17");
        test.report(&mut smoke, views.director.initial_seed(), exit);
        return;
    }
    match test.stage {
        Stage::Boot if age == 24 => {
            restart(&mut session, &mut player, &mut comfort);
            comfort.vhs = crate::vhs::VhsMode::Off;
            comfort.diagnostics = false;
            face(
                &mut player,
                SERVICE_ORDER.with_y(0.002) + Vec3::Z * 1.60,
                SERVICE_ORDER,
            );
            test.next(Stage::Order);
        }
        Stage::Order if age == 12 => keys.press(KeyCode::KeyE),
        Stage::Order if age == 13 => {
            keys.release(KeyCode::KeyE);
            require(
                &mut smoke,
                session.memory().read(Action::ReadServiceOrder)
                    && matches!(session.memory().service, ServiceTrail::Briefed(_))
                    && session.message.contains("02:52"),
                "segunda_via_lida",
            );
            turn_to(&mut player, DESK_SWITCH);
            test.next(Stage::Lamp);
        }
        Stage::Lamp if age == 14 => keys.press(KeyCode::KeyE),
        Stage::Lamp if age == 15 => keys.release(KeyCode::KeyE),
        Stage::Lamp if age == 60 => {
            require(
                &mut smoke,
                session.memory().desk_light
                    && views.lights.iter().any(|(circuit, light)| {
                        matches!(circuit.circuit, Circuit::Desk) && light.intensity > 1000.0
                    }),
                "luminaria_interativa_e_visivel",
            );
            smoke.capture(&mut commands, "01-arquivo-luminaria");
        }
        Stage::Lamp if age == 72 => keys.press(KeyCode::KeyE),
        Stage::Lamp if age == 73 => {
            keys.release(KeyCode::KeyE);
            require(
                &mut smoke,
                !session.memory().desk_light,
                "luminaria_pode_ser_apagada",
            );
            face(&mut player, FILE.with_y(0.002) + Vec3::Z * 1.84, FILE);
            test.next(Stage::Folder);
        }
        Stage::Folder if age == 48 => {
            require(
                &mut smoke,
                session.memory().knows_variant(Prop::File, 0),
                "ficha_observada",
            );
            begin_rearm(&mut session, &mut player);
            test.next(Stage::Rearm);
        }
        Stage::Rearm if age == 10 => keys.press(KeyCode::KeyE),
        Stage::Rearm if age == 11 => {
            keys.release(KeyCode::KeyE);
            require(
                &mut smoke,
                session.progress().phase() == Phase::Return,
                "rearme_real",
            );
            face(&mut player, RELAY_VIEW, SERVICE_RELAY);
            test.next(Stage::RelayOriginal);
        }
        Stage::RelayOriginal if age == 47 => {
            require(
                &mut smoke,
                session.memory().knows_variant(Prop::Relay, 0),
                "placa_original_observada",
            );
            smoke.capture(&mut commands, "02-rele-cancelado");
        }
        Stage::RelayOriginal if age == 59 => {
            place(&mut player, Vec3::new(29.60, 0.002, -58.25), PI, 0.0);
            test.next(Stage::Walk);
        }
        Stage::Walk => {
            follow(&mut player, &mut test.waypoint, WALK);
            if test.waypoint == WALK.len() {
                test.next(Stage::Echo);
            }
        }
        Stage::Echo if matches!(session.memory().service, ServiceTrail::Footfalls(_)) => {
            require(
                &mut smoke,
                session.expedition.observer().reality() == Reality::Real
                    && session.expedition.truth().phase() == Phase::Return,
                "eco_contextual_ainda_em_realidade",
            );
            face(&mut player, RELAY_VIEW, SERVICE_RELAY);
            player.yaw += PI;
            player.pitch = 0.0;
            test.next(Stage::WaitForRelay);
        }
        Stage::WaitForRelay if matches!(session.memory().service, ServiceTrail::Relay(at) if session.elapsed - at > 0.30) =>
        {
            let transferred = views.props.iter().any(|(prop, pose)| {
                prop.id == Prop::File && pose.translation.distance(SERVICE_RELAY) < 0.65
            });
            require(&mut smoke, transferred, "ficha_reencontrada_junto_ao_rele");
            require(
                &mut smoke,
                views
                    .props
                    .iter()
                    .any(|(prop, _)| prop.id == Prop::Relay && prop.stage() == 1),
                "placa_alterada_fora_do_campo_visual",
            );
            turn_to(&mut player, SERVICE_RELAY);
            test.next(Stage::CheckRelay);
        }
        Stage::CheckRelay if age == 14 => keys.press(KeyCode::KeyE),
        Stage::CheckRelay if age == 15 => {
            keys.release(KeyCode::KeyE);
            require(
                &mut smoke,
                matches!(session.memory().service, ServiceTrail::Checked(_))
                    && session.message.contains("03:19"),
                "assinatura_0319",
            );
            smoke.capture(&mut commands, "03-rele-assinado");
        }
        Stage::CheckRelay if age == 30 => {
            face(&mut player, TERMINAL_VIEW, TERMINAL);
            test.next(Stage::OriginalFrame);
        }
        Stage::OriginalFrame if age == 16 => keys.press(KeyCode::KeyE),
        Stage::OriginalFrame if age == 17 => keys.release(KeyCode::KeyE),
        Stage::OriginalFrame if age == 38 => {
            require(
                &mut smoke,
                views.recording.displayed() && !views.recording.showing_echo(),
                "primeira_gravacao_ainda_disponivel",
            );
            smoke.capture(&mut commands, "04-camera-original");
            test.next(Stage::LaterFrame);
        }
        Stage::LaterFrame if age == 480 => keys.press(KeyCode::KeyE),
        Stage::LaterFrame if age == 481 => keys.release(KeyCode::KeyE),
        Stage::LaterFrame if age == 526 => {
            require(
                &mut smoke,
                views.recording.showing_echo()
                    && session.memory().service == ServiceTrail::Reconciled
                    && session.message.contains("03:17")
                    && session.expedition.truth().phase() == Phase::Return,
                "segunda_gravacao_e_progresso_intacto",
            );
            smoke.capture(&mut commands, "05-camera-0317");
        }
        Stage::LaterFrame if age == 543 => {
            face(
                &mut player,
                PUMP_02_PANEL.with_x(PUMP_02_PANEL.x - 1.66).with_y(0.002),
                PUMP_02_PANEL,
            );
            test.next(Stage::ReadPump);
        }
        Stage::ReadPump if age == 18 => keys.press(KeyCode::KeyE),
        Stage::ReadPump if age == 19 => {
            keys.release(KeyCode::KeyE);
            require(
                &mut smoke,
                session.expedition.truth().exit_unlocked(),
                "vistoria_libera_saida",
            );
            place(&mut player, Vec3::new(0.0, 0.002, -84.46), 0.0, 0.0);
            player.command.movement = Vec2::Y;
            player.command.sprint = true;
            test.next(Stage::Exit);
        }
        Stage::Exit if session.screen == Screen::Complete => {
            require(
                &mut smoke,
                session.expedition.observer().outcome() == Some(Outcome::Escaped)
                    && session.memory().service == ServiceTrail::Reconciled,
                "desfecho_real_com_descoberta",
            );
            restart(&mut session, &mut player, &mut comfort);
            begin_rearm(&mut session, &mut player);
            test.next(Stage::Interrupt);
        }
        Stage::Interrupt if age == 12 => keys.press(KeyCode::KeyE),
        Stage::Interrupt if age == 13 => {
            keys.release(KeyCode::KeyE);
            session.screen = Screen::Paused;
            ui::request_awakening(&mut session);
            awaken(&mut session, &mut player);
            require(
                &mut smoke,
                session.screen == Screen::Complete && session.expedition.can_resume(),
                "interrupcao_oferece_posto",
            );
            smoke.capture(&mut commands, "06-retomada-oferecida");
            test.next(Stage::Resume);
        }
        Stage::Resume if age == 18 => continue_or_restart(&mut session, &mut player, &mut comfort),
        Stage::Resume if age == 36 => {
            require(
                &mut smoke,
                session.screen == Screen::Playing
                    && session.expedition.observer().reality() == Reality::Real
                    && session.progress().phase() == Phase::Return
                    && !session.progress().exit_unlocked()
                    && player.body.feet.distance(RETURN_POST.feet) < 0.04
                    && session.memory().service == ServiceTrail::Unknown
                    && !session.memory().desk_light
                    && !views.recording.displayed(),
                "retomada_limpa_no_posto",
            );
            session.screen = Screen::Paused;
            ui::request_awakening(&mut session);
            awaken(&mut session, &mut player);
            require(
                &mut smoke,
                session.screen == Screen::Complete && !session.expedition.can_resume(),
                "retomada_unica",
            );
            test.next(Stage::Finish);
        }
        Stage::Finish if age == 18 => test.report(&mut smoke, views.director.initial_seed(), exit),
        _ => (),
    }
}

fn require(smoke: &mut SmokeTest, condition: bool, label: &'static str) {
    if !condition {
        error!("Falha na rota da OS 17: {label}");
        smoke.failures.push(label);
    }
}

fn face(player: &mut LocalPlayer, feet: Vec3, target: Vec3) {
    place(player, feet, 0.0, 0.0);
    turn_to(player, target);
}

fn begin_rearm(session: &mut Session, player: &mut LocalPlayer) {
    session.act(Action::TakeFuse);
    session.act(Action::InspectPump);
    face(player, RETURN_POST.feet, PANEL);
}
