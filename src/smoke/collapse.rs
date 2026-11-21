//! Native validation of recovery and the real fixed-tick deadline.
use super::{SmokeTest, place, turn_to};
use crate::{
    app::*,
    psychology::{Effects, presentation::BodySound},
    scene::station::{REST_POSTS, near_rest_post},
};
use ainda_vivo_core::{
    psychology::{Level, Recovery},
    reality::Reality,
    story::Action,
};
use bevy::{app::AppExit, audio::AudioSinkPlayback, prelude::*};

const OUTSIDE: Vec3 = Vec3::new(0.0, 0.002, 3.5);

#[derive(Resource, Default)]
pub struct Check {
    frame: u32,
    paused: f32,
    reached: Option<f32>,
    used: Option<f32>,
    terminal: u8,
    audio: [bool; 2],
}

pub fn inject_reserve(
    smoke: Res<SmokeTest>,
    mut check: ResMut<Check>,
    session: Res<Session>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    if !smoke.collapse {
        return;
    }
    if check.used.is_none()
        && session.expedition.observer().reality() == Reality::False
        && let Some(seconds) = session.psychology.critical_remaining()
        && (0.60..=0.75).contains(&seconds)
    {
        check.used = Some(seconds);
        keys.press(KeyCode::KeyR);
    } else {
        keys.release(KeyCode::KeyR);
    }
}

#[allow(clippy::too_many_arguments, reason = "Native Bevy smoke parameters.")]
pub fn drive(
    mut smoke: ResMut<SmokeTest>,
    mut check: ResMut<Check>,
    (mut session, mut player, mut comfort): (ResMut<Session>, ResMut<LocalPlayer>, ResMut<Comfort>),
    mut keys: ResMut<ButtonInput<KeyCode>>,
    effects: Res<Effects>,
    voices: Query<(&BodySound, &AudioSink)>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if !smoke.collapse {
        return;
    }
    check.frame += 1;
    if (20..800).contains(&check.frame) {
        for (sound, sink) in &voices {
            if !sink.is_paused() && !sink.position().is_zero() {
                check.audio[match sound {
                    BodySound::Breath => 0,
                    BodySound::Heart => 1,
                }] = true;
            }
        }
    }
    match check.frame {
        20 => {
            restart(&mut session, &mut player, &mut comfort);
            comfort.psycho_indicator = true;
            comfort.diagnostics = false;
            comfort.vhs = crate::vhs::VhsMode::Off;
            let post = REST_POSTS[0];
            place(&mut player, post.with_y(0.002) + Vec3::Z * 1.65, 0.0, 0.0);
            turn_to(&mut player, post);
            session.psychology.set_for_development(80.0);
        }
        40 => keys.press(KeyCode::KeyE),
        42 => {
            keys.release(KeyCode::KeyE);
            require(
                &mut smoke,
                session.psychology.has_reserve() && session.psychology.recovery() == Recovery::Rest,
                "posto_E_descanso_reserva",
            );
            smoke.capture(&mut commands, "01-posto-de-pausa");
        }
        75 => {
            place(&mut player, OUTSIDE, 0.0, 0.0);
            session.resting = false;
            session.psychology.set_for_development(100.0);
        }
        108 => {
            check.paused = session.psychology.critical_remaining().unwrap_or_default();
            session.screen = Screen::Paused;
        }
        155 => {
            require(
                &mut smoke,
                session.psychology.critical_remaining() == Some(check.paused),
                "pausa_congela_temporizador",
            );
            if smoke.check_audio {
                require(
                    &mut smoke,
                    voices.iter().all(|(_, sink)| sink.is_paused()),
                    "pausa_audio",
                );
            }
            session.screen = Screen::Playing;
            player.command.movement = Vec2::Y;
        }
        175 => smoke.capture(&mut commands, "02-urgencia-opcional"),
        879 => {
            require(
                &mut smoke,
                check.reached.is_some_and(|left| left > 45.0)
                    && session.psychology.critical_remaining().is_none(),
                "caminhada_recupera_em_tempo",
            );
            place(&mut player, OUTSIDE, 0.0, 0.0);
            session.psychology.set_for_development(47.0);
            for action in [Action::TakeFuse, Action::InspectPump, Action::RestorePower] {
                session.act(action);
            }
            let now = session.elapsed;
            let captured = session.expedition.capture(now);
            require(
                &mut smoke,
                captured && session.psychology.value() == 47.0,
                "captura_continuidade",
            );
            session.psychology.set_for_development(100.0);
        }
        _ => (),
    }
    if (155..879).contains(&check.frame)
        && near_rest_post(player.body.feet)
        && check.reached.is_none()
    {
        check.reached = session.psychology.critical_remaining();
        player.command = default();
    }
    if check.used.is_some() && check.terminal == 0 {
        require(
            &mut smoke,
            session.psychology.critical_remaining().is_none()
                && !session.psychology.has_reserve()
                && session.psychology.value() < 85.0,
            "R_ultimo_segundo",
        );
        session.psychology.set_for_development(100.0);
        restart(&mut session, &mut player, &mut comfort);
        require(
            &mut smoke,
            session.psychology.level() == Level::Stable
                && session.psychology.critical_remaining().is_none()
                && session.psychology.records().count() == 0,
            "reinicio_durante_colapso",
        );
        place(&mut player, OUTSIDE, 0.0, 0.0);
        session.psychology.set_for_development(100.0);
        check.terminal = 1;
    }
    if let Some(age) = effects.farewell {
        match check.terminal {
            1 if age >= 2.9 => {
                require(
                    &mut smoke,
                    player.body.crouched && player.body.feet.distance(OUTSIDE) < 0.05,
                    "ajoelha_preservando_posicao",
                );
                smoke.capture(&mut commands, "03-presenca-imovel");
                check.terminal = 2;
            }
            2 if age >= 4.32 => {
                smoke.capture(&mut commands, "04-inclinacao");
                check.terminal = 3;
            }
            3 if age >= 5.52 => {
                smoke.capture(&mut commands, "05-aceno");
                check.terminal = 4;
            }
            4 if session.collapse_finished() => {
                smoke.capture(&mut commands, "06-registro-encerrado");
                check.terminal = 5;
            }
            _ => (),
        }
    }
    if (check.terminal == 5 && (smoke.capture_dir.is_none() || smoke.captures >= 6))
        || check.frame >= 9300
    {
        require(&mut smoke, check.frame < 9300, "tempo_limite");
        if smoke.check_audio {
            require(
                &mut smoke,
                check.audio == [true, true],
                "audio_corporal_nativo",
            );
        }
        let report = format!(
            "AINDA VIVO v{VERSION}\nsmoke-collapse: {}\nframes: {}\nRefuge reached with {:.2}s remaining\nR at {:.2}s remaining in False Reality\nBreathing / heartbeat sinks: {:?}\nFarewell stages: {}\nCaptures: {}\nFailures: {:?}\n",
            if smoke.failures.is_empty() {
                "PASS"
            } else {
                "FAIL"
            },
            check.frame,
            check.reached.unwrap_or_default(),
            check.used.unwrap_or_default(),
            check.audio,
            check.terminal,
            smoke.captures,
            smoke.failures
        );
        info!("{report}");
        let written = smoke
            .report
            .as_ref()
            .is_none_or(|path| std::fs::write(path, report).is_ok());
        exit.write(if written && smoke.failures.is_empty() {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}

fn require(smoke: &mut SmokeTest, condition: bool, name: &'static str) {
    if !condition {
        smoke.failures.push(name);
        error!("Colapso: {name}");
    }
}
