//! Native integration route for capture, the perceived exit and the two awakening outcomes.
use super::{SmokeTest, follow, place, turn_to};
use crate::{
    app::*,
    atmosphere::{
        director::{Cue, Director},
        sound::Emitter,
    },
    reality::Threat,
    scene::{
        illusion::{self, Illusions, Impression, ImpressionId},
        presence::{Presence, WEST_MARK, WEST_WITNESS},
        station::{PANEL, PUMP_02_PANEL, RETURN_MEMO, RETURN_POST},
    },
    ui,
    vhs::VhsMode,
};
use ainda_vivo_core::{
    movement::SPAWN,
    reality::{CONTACT_RADIUS, Outcome, Reality},
    story::{Action, Phase},
};
use bevy::{app::AppExit, audio::AudioSinkPlayback, ecs::system::SystemParam, prelude::*};

const APPROACH: &[Vec2] = &[Vec2::new(-1.48, -52.34)];
const PUMP_VIEW: Vec3 = Vec3::new(PUMP_02_PANEL.x - 1.66, 0.002, PUMP_02_PANEL.z);
const CLOCK_VIEW: Vec3 = Vec3::new(22.10, 0.002, -84.28);

#[derive(Debug, Default)]
enum Stage {
    #[default]
    Boot,
    Rearm,
    Memo,
    WaitForFigure,
    Approach,
    Retreat,
    LoseSight,
    KeepWalking,
    LookAtClock,
    WaitForClock,
    ReadPump(bool),
    WalkToExit(bool),
    Receipt,
    PausedReceipt,
    WaitForContradiction,
    ConfirmRecovery,
    CheckRecovery,
    AwakenAlive,
    Finish,
}

#[derive(Clone, Copy)]
struct Motion {
    feet: Vec3,
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    light: bool,
    when: f32,
    message: &'static str,
}

#[derive(Resource, Default)]
pub struct ContinuitySmoke {
    stage: Stage,
    frame: u32,
    start: u32,
    route: usize,
    previous: Option<Motion>,
    captured_from: Vec3,
    paused_at: f32,
    evaded: bool,
    first_witness_time: Option<f32>,
    reproduced: bool,
    continuity: bool,
    clock: bool,
    fake_exit: bool,
    pause: bool,
    recovered: bool,
    real_exit: bool,
    interrupted: bool,
    reset: bool,
    memo: bool,
    sounds: u8,
}

impl ContinuitySmoke {
    fn next(&mut self, stage: Stage) {
        self.stage = stage;
        self.start = self.frame;
    }

    fn report(&self, smoke: &mut SmokeTest, director: &Director, mut exit: MessageWriter<AppExit>) {
        if smoke.capture_dir.is_some() && smoke.captures != 10 {
            smoke.failures.push("capturas_incompletas");
        }
        if smoke.check_audio && self.sounds & 3 != 3 {
            smoke.failures.push("reproducao_nativa_dos_dois_sons");
        }
        let passed = smoke.failures.is_empty();
        let report = format!(
            "AINDA VIVO v{VERSION}\nsmoke-reality: {}\nframes: {}\nseed: {}\nroute escaped warning: {}\nseed reproduced the same sighting time: {}\ncontact kept input, torch, clock, position and speed: {}\nControl clock changed behind the observer: {}\nperceived exit showed receipt then contradiction: {}\npausing suspended the receipt and audio: {}\nrecovery kept real progress and returned to the same post: {}\ntrue exit after recovery: {}\nawakening while alive ended the attempt: {}\nrestart cleared the private state: {}\noptional return memo read via interaction ray: {}\nrequested audio playback check: {}\ncontact/reception playback mask: {:02b}\ncaptures: {}\nfailures: {:?}\n",
            if passed { "PASS" } else { "FAIL" },
            self.frame,
            director.initial_seed(),
            self.evaded,
            self.reproduced,
            self.continuity,
            self.clock,
            self.fake_exit,
            self.pause,
            self.recovered,
            self.real_exit,
            self.interrupted,
            self.reset,
            self.memo,
            smoke.check_audio,
            self.sounds & 3,
            smoke.captures,
            smoke.failures
        );
        info!("{report}");
        let written = smoke.report.as_ref().is_none_or(|path| {
            std::fs::write(path, &report)
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
    presence: Res<'w, Presence>,
    threat: Res<'w, Threat>,
    illusions: Res<'w, Illusions>,
    director: ResMut<'w, Director>,
    world: Res<'w, CollisionWorld>,
    impressions: Query<'w, 's, &'static Impression>,
    voices: Query<'w, 's, (&'static Emitter, &'static AudioSink)>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy integration system parameters."
)]
pub fn drive(
    mut smoke: ResMut<SmokeTest>,
    mut test: ResMut<ContinuitySmoke>,
    mut session: ResMut<Session>,
    mut player: ResMut<LocalPlayer>,
    mut comfort: ResMut<Comfort>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut world: Views,
    mut commands: Commands,
    exit: MessageWriter<AppExit>,
) {
    if !smoke.reality {
        return;
    }
    test.frame += 1;
    if smoke.check_audio {
        for (emitter, sink) in &world.voices {
            if !sink.is_paused() && !sink.position().is_zero() {
                test.sounds |= emitter.reality_sound();
            }
        }
    }
    let age = test.frame - test.start;
    if age > 2700 || test.frame > 13000 {
        error!(
            "Etapa {:?}, relógio {:.2}s, posição {:?}",
            test.stage, session.elapsed, player.body.feet
        );
        smoke.failures.push("tempo_excedido");
        test.report(&mut smoke, &world.director, exit);
        return;
    }
    match test.stage {
        Stage::Boot if age >= 18 => {
            start_return(&mut session, &mut player, &mut comfort);
            test.next(Stage::Rearm);
        }
        Stage::Rearm if age == 10 => keys.press(KeyCode::KeyE),
        Stage::Rearm if age == 11 => {
            keys.release(KeyCode::KeyE);
            verify(
                &mut smoke,
                session.progress().phase() == Phase::Return,
                "rearme_por_interacao",
            );
            place(
                &mut player,
                RETURN_MEMO.with_x(32.85).with_y(0.002),
                0.0,
                0.0,
            );
            turn_to(&mut player, RETURN_MEMO);
            test.next(Stage::Memo);
        }
        Stage::Memo if age == 10 => keys.press(KeyCode::KeyE),
        Stage::Memo if age == 11 => {
            keys.release(KeyCode::KeyE);
            test.memo = session.message.contains("Posto da Elétrica");
            verify(&mut smoke, test.memo, "instrução_do_posto");
        }
        Stage::Memo if age >= 30 => {
            place(&mut player, WEST_WITNESS.with_y(0.002), 0.0, 0.0);
            turn_to(&mut player, WEST_MARK + Vec3::Y * 1.65);
            session.message_left = 0.0;
            test.next(Stage::WaitForFigure);
        }
        Stage::WaitForFigure => {
            if let Some(trace) = world.presence.contact
                && trace.position == WEST_MARK
            {
                if !test.evaded {
                    test.first_witness_time = Some(trace.at);
                } else {
                    test.reproduced = test
                        .first_witness_time
                        .is_some_and(|at| (at - trace.at).abs() <= 0.017);
                    verify(&mut smoke, test.reproduced, "seed_da_aparicao");
                    smoke.capture(&mut commands, "retorno-observado");
                }
                test.route = 0;
                test.next(Stage::Approach);
            }
        }
        Stage::Approach => {
            follow(&mut player, &mut test.route, APPROACH);
            if test.route == APPROACH.len() {
                verify(
                    &mut smoke,
                    world.threat.active() && world.threat.warnings == 1,
                    "aviso_na_aproximacao",
                );
                turn_to(&mut player, WEST_MARK + Vec3::Y * 1.40);
                if test.evaded {
                    player.yaw += std::f32::consts::PI;
                    player.pitch = 0.0;
                    test.next(Stage::LoseSight);
                } else {
                    test.next(Stage::Retreat);
                }
            }
        }
        Stage::Retreat => {
            player.command.yaw = player.yaw;
            player.command.movement = Vec2::NEG_Y;
            player.command.sprint = true;
            if age > 20
                && !world.threat.active()
                && player.body.feet.xz().distance(WEST_MARK.xz()) > CONTACT_RADIUS
            {
                test.evaded = session.expedition.observer().reality() == Reality::Real
                    && world.threat.captures == 0;
                verify(&mut smoke, test.evaded, "recuo_evade_captura");
                start_return(&mut session, &mut player, &mut comfort);
                test.next(Stage::Rearm);
            }
        }
        Stage::LoseSight => {
            // Keep creeping sideways while looking away. Compare neighboring
            // physics samples across the actual (never forced) transition.
            player.command.yaw = player.yaw;
            player.command.movement = Vec2::new(0.32, 0.0);
            player.command.crouch = true;
            if session.expedition.observer().reality() == Reality::False {
                test.continuity = test.previous.is_some_and(|before| {
                    let distance = player.body.feet.distance(before.feet);
                    distance < 0.065
                        && player.body.velocity.distance(before.velocity) < 0.12
                        && player.body.velocity.xz().length() > 0.28
                        && player.yaw == before.yaw
                        && player.pitch == before.pitch
                        && comfort.flashlight == before.light
                        && (0.0..=0.018).contains(&(session.elapsed - before.when))
                        && session.message == before.message
                }) && session.screen == Screen::Playing
                    && session.progress().phase() == Phase::Return
                    && world.threat.captures == 1;
                verify(&mut smoke, test.continuity, "continuidade_na_captura");
                test.captured_from = player.body.feet;
                test.next(Stage::KeepWalking);
            } else {
                test.previous = Some(Motion {
                    feet: player.body.feet,
                    velocity: player.body.velocity,
                    yaw: player.yaw,
                    pitch: player.pitch,
                    light: comfort.flashlight,
                    when: session.elapsed,
                    message: session.message,
                });
            }
        }
        Stage::KeepWalking if age == 18 => {
            smoke.capture(&mut commands, "continuidade-apos-contato")
        }
        Stage::KeepWalking if age >= 29 => {
            let moved = player.body.feet.distance(test.captured_from) > 0.10;
            test.continuity &= moved;
            verify(&mut smoke, moved, "movimento_apos_captura");
            place(&mut player, CLOCK_VIEW, 0.0, 0.0);
            turn_to(&mut player, illusion::CLOCK);
            test.next(Stage::LookAtClock);
        }
        Stage::LookAtClock if age == 30 => smoke.capture(&mut commands, "relogio-antes"),
        Stage::LookAtClock if age >= 44 => {
            player.yaw += std::f32::consts::PI;
            player.pitch = 0.0;
            world.director.request(Cue::Illusion);
            test.next(Stage::WaitForClock);
        }
        Stage::WaitForClock => {
            if world
                .impressions
                .iter()
                .any(|prop| prop.id == ImpressionId::Clock && prop.stage() == 1)
            {
                test.clock = session.expedition.observer().reality() == Reality::False;
                verify(&mut smoke, test.clock, "relogio_perceptivo");
                turn_to(&mut player, illusion::CLOCK);
                test.next(Stage::ReadPump(false));
            }
        }
        Stage::ReadPump(false) if age == 18 => smoke.capture(&mut commands, "relogio-depois"),
        Stage::ReadPump(_) if age == 34 => {
            place(&mut player, PUMP_VIEW, 0.0, 0.0);
            turn_to(&mut player, PUMP_02_PANEL);
        }
        Stage::ReadPump(_) if age == 52 => keys.press(KeyCode::KeyE),
        Stage::ReadPump(real) if age == 53 => {
            keys.release(KeyCode::KeyE);
            verify(
                &mut smoke,
                session.progress().exit_unlocked()
                    && session.expedition.truth().exit_unlocked() == real,
                "isolamento_da_segunda_leitura",
            );
            place(&mut player, Vec3::new(0.0, 0.002, -84.46), 0.0, 0.0);
            player.command.movement = Vec2::Y;
            player.command.sprint = true;
            test.next(Stage::WalkToExit(real));
        }
        Stage::WalkToExit(real) => {
            let expected = if real {
                Outcome::Escaped
            } else {
                Outcome::PerceivedExit
            };
            if session.expedition.observer().outcome() == Some(expected) {
                turn_to(&mut player, illusion::RECEIPT_DISPLAY);
                if real {
                    test.real_exit = session.screen == Screen::Complete;
                    verify(&mut smoke, test.real_exit, "desfecho_real");
                    test.next(Stage::AwakenAlive);
                } else {
                    test.fake_exit =
                        matches!(session.false_exit, Some(ExitPresentation::Receipt { .. }))
                            && session.screen == Screen::Playing;
                    verify(&mut smoke, test.fake_exit, "recibo_da_portaria");
                    test.next(Stage::Receipt);
                }
            }
        }
        Stage::Receipt if age == 22 => smoke.capture(&mut commands, "portaria-baixa"),
        Stage::Receipt if age >= 32 => {
            test.paused_at = session.elapsed;
            session.screen = Screen::Paused;
            test.next(Stage::PausedReceipt);
        }
        Stage::PausedReceipt if age >= 48 => {
            test.pause = session.elapsed == test.paused_at
                && matches!(session.false_exit, Some(ExitPresentation::Receipt { .. }))
                && (!smoke.check_audio
                    || !world.voices.is_empty()
                        && world.voices.iter().all(|(_, sink)| sink.is_paused()));
            verify(&mut smoke, test.pause, "pausa_do_recibo_e_audio");
            session.screen = Screen::Playing;
            test.next(Stage::WaitForContradiction);
        }
        Stage::WaitForContradiction => {
            if session.false_exit == Some(ExitPresentation::Contradiction) {
                test.fake_exit &= session.screen == Screen::Playing
                    && session.message.contains("ainda não começou");
                verify(&mut smoke, test.fake_exit, "contradicao_da_portaria");
                test.next(Stage::ConfirmRecovery);
            }
        }
        Stage::ConfirmRecovery if age == 15 => smoke.capture(&mut commands, "portaria-pendente"),
        Stage::ConfirmRecovery if age == 66 => {
            session.screen = Screen::Paused;
            awaken(&mut session, &mut player);
            verify(
                &mut smoke,
                session.screen == Screen::Paused
                    && session.expedition.observer().reality() == Reality::False,
                "despertar_sem_confirmacao",
            );
            ui::request_awakening(&mut session);
        }
        Stage::ConfirmRecovery if age == 78 => {
            smoke.capture(&mut commands, "confirmacao-despertar")
        }
        Stage::ConfirmRecovery if age >= 92 => {
            awaken(&mut session, &mut player);
            test.next(Stage::CheckRecovery);
        }
        Stage::CheckRecovery if age == 12 => {
            let gate_blocks = world.world.occludes(
                Vec3::new(0.0, 1.56, -84.50),
                Vec3::new(0.0, 1.56, -90.0),
                session.progress().exit_unlocked(),
            );
            test.recovered = session.expedition.observer().reality() == Reality::Real
                && player.body.feet.distance(RETURN_POST.feet) < 0.04
                && session.progress().phase() == Phase::Return
                && session.false_exit.is_none()
                && session.expedition.observer().outcome().is_none()
                && world.illusions.altered == 0
                && world.threat.captures == 0
                && gate_blocks
                && !session.expedition.can_capture();
            verify(
                &mut smoke,
                test.recovered,
                "recuperacao_limitada_com_porta_real",
            );
            smoke.capture(&mut commands, "posto-retomado");
        }
        Stage::CheckRecovery if age >= 30 => test.next(Stage::ReadPump(true)),
        Stage::AwakenAlive if age == 21 => smoke.capture(&mut commands, "saida-real"),
        Stage::AwakenAlive if age == 36 => restart(&mut session, &mut player, &mut comfort),
        Stage::AwakenAlive if age == 54 => {
            test.reset = session.expedition.observer().reality() == Reality::Real
                && session.expedition.observer().outcome().is_none()
                && session.progress().phase() == Phase::Fuse
                && session.perception_revision == 0
                && world.illusions.altered == 0
                && world.threat.warnings == 0
                && player.body.feet.distance(SPAWN) < 0.06;
            verify(&mut smoke, test.reset, "reinicio_completo");
            session.screen = Screen::Paused;
            ui::request_awakening(&mut session);
            awaken(&mut session, &mut player);
            test.interrupted = session.screen == Screen::Complete
                && session.expedition.observer().reality() == Reality::Eliminated
                && session.expedition.observer().outcome() == Some(Outcome::Interrupted);
            verify(&mut smoke, test.interrupted, "despertar_na_realidade");
            test.next(Stage::Finish);
        }
        Stage::Finish if age == 20 => smoke.capture(&mut commands, "registro-interrompido"),
        Stage::Finish if age >= 95 => test.report(&mut smoke, &world.director, exit),
        _ => (),
    }
}

fn start_return(session: &mut Session, player: &mut LocalPlayer, comfort: &mut Comfort) {
    restart(session, player, comfort);
    comfort.vhs = VhsMode::Off;
    comfort.diagnostics = false;
    // The independent legacy smoke walks this full initial route.
    session.act(Action::TakeFuse);
    session.act(Action::InspectPump);
    place(player, RETURN_POST.feet, 0.0, 0.0);
    turn_to(player, PANEL);
}

fn verify(smoke: &mut SmokeTest, passed: bool, label: &'static str) {
    if !passed {
        error!("Falhou: {label}");
        smoke.failures.push(label);
    }
}
