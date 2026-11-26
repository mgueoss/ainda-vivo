use super::{BotStyle, Campaign, Command, RelayClick, brain::STEP, hunt::Phase};
use crate::{
    app::{Comfort, LocalPlayer, Session, VERSION, restart},
    lab::{Action, Lab, Options, library::Catalog},
    scene::station::SERVICE_RELAY,
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

#[derive(Default, Resource)]
pub(crate) struct Check {
    frames: u32,
    ticks: u32,
    errors: Vec<&'static str>,
    before: f32,
    arm_time: Option<f32>,
    relay_time: Option<f32>,
    pulses: u8,
    position: Option<Vec2>,
    excursion: f32,
    mission: Option<ainda_vivo_core::story::Phase>,
    frozen: f32,
}

impl Check {
    fn need(&mut self, satisfied: bool, label: &'static str) {
        if !satisfied {
            self.errors.push(label);
            error!("Inteligência: {label}");
        }
    }
}

pub(crate) fn keys(check: Option<Res<Check>>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    let Some(check) = check else {
        return;
    };
    match check.frames {
        50 => keys.press(KeyCode::KeyE),
        51 => keys.release(KeyCode::KeyE),
        178 | 516 => {
            keys.press(KeyCode::ControlLeft);
            keys.press(KeyCode::KeyI);
        }
        179 | 517 => {
            keys.release(KeyCode::ControlLeft);
            keys.release(KeyCode::KeyI);
        }
        _ => {}
    }
}

pub(crate) fn verify(
    (mut check, options, catalog, lab): (ResMut<Check>, Res<Options>, Res<Catalog>, Res<Lab>),
    (mut session, mut player, mut comfort, campaign): (
        ResMut<Session>,
        ResMut<LocalPlayer>,
        ResMut<Comfort>,
        Res<Campaign>,
    ),
    (mut actions, mut relay, mut commands, mut exit): (
        MessageWriter<Action>,
        MessageReader<RelayClick>,
        Commands,
        MessageWriter<AppExit>,
    ),
) {
    let c = &mut *check;
    c.ticks += 1;
    if catalog.number(28).is_none() && c.ticks < 900 {
        return;
    }
    c.frames += 1;
    let f = c.frames;
    for pulse in relay.read() {
        c.pulses += 1;
        if pulse.0.distance(SERVICE_RELAY) < 0.42 {
            c.relay_time = Some(session.elapsed);
        }
    }
    if let Some(bot) = lab.active.as_ref().and_then(|a| a.observer.bot.as_ref()) {
        let initial = c.position.get_or_insert(bot.at);
        c.excursion = c.excursion.max(bot.at.distance(*initial));
    }
    let action = match f {
        10 => {
            restart(&mut session, &mut player, &mut comfort);
            let feet = Vec3::new(32.85, 0.002, SERVICE_RELAY.z);
            player.body.feet = feet;
            player.body.previous_feet = feet;
            let delta = SERVICE_RELAY - (feet + Vec3::Y * 1.62);
            player.yaw = (-delta.x).atan2(-delta.z);
            player.pitch = delta.y.atan2(delta.xz().length());
            None
        }
        50 => {
            c.need(
                campaign.relay_ready(session.elapsed),
                "relé elegível na posição de teste",
            );
            c.arm_time = Some(session.elapsed);
            None
        }
        145 => {
            c.need(
                c.pulses == 1
                    && c.relay_time
                        .zip(c.arm_time)
                        .is_some_and(|(pulse, armed)| (1.42..=1.53).contains(&(pulse - armed))),
                "a interação E dispara um único estalo físico após a espera",
            );
            None
        }
        182 => {
            c.need(lab.active.is_some(), "Ctrl+I abriu o laboratório");
            c.mission = Some(session.progress().phase());
            c.frozen = session.elapsed;
            Some(Command::Enable)
        }
        201 => Some(Command::Obstacle),
        216 => {
            c.need(
                lab.active.as_ref().is_some_and(|a| a.observer.obstacle),
                "caixa colocada",
            );
            Some(Command::Bot(BotStyle::Side))
        }
        230 => Some(Command::Pause),
        238 => {
            c.before = lab.active.as_ref().map_or(0., |a| a.observer.brain.clock);
            Some(Command::Step)
        }
        250 => {
            c.need(
                lab.active.as_ref().is_some_and(|a| {
                    a.observer.paused
                        && a.observer.bot_running()
                        && ((a.observer.brain.clock - c.before) - STEP).abs() < 0.0002
                }),
                "pausa mantém o bot e avança um único passo",
            );
            Some(Command::ClearMemory)
        }
        258 => {
            c.need(
                lab.active.as_ref().is_some_and(|a| {
                    a.observer.brain.seen.is_none() && a.observer.brain.memory.sightings == 0
                }),
                "memória pode ser limpa com a IA pausada",
            );
            Some(Command::Pause)
        }
        286 => Some(Command::Obstacle),
        410 => {
            c.need(c.excursion >= 0.60, "bot executa desvio visível");
            c.need(
                lab.active.as_ref().is_some_and(|a| {
                    !a.observer.obstacle && a.observer.brain.hunt.phase != Phase::Idle
                }),
                "pista sem caixa com a caçada em execução",
            );
            if let Some(dir) = &options.captures
                && std::fs::create_dir_all(dir).is_ok()
            {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(dir.join("01-laboratorio-cognitivo.png")));
            }
            Some(Command::Matrix)
        }
        420 => {
            c.need(
                lab.active.as_ref().is_some_and(|a| {
                    a.observer
                        .result
                        .as_ref()
                        .is_some_and(|r| r.contains("PASS"))
                }),
                "matriz acionada por interface",
            );
            Some(Command::Export)
        }
        444 => Some(Command::Replay),
        453 => {
            c.need(
                lab.active.as_ref().is_some_and(|a| {
                    a.observer.brain.hunt.phase == Phase::Prepare
                        && a.observer.bot.as_ref().is_some_and(|bot| {
                            bot.style == BotStyle::Side && Some(bot.at) == c.position
                        })
                }),
                "repetir seed reinicia o mesmo bot",
            );
            Some(Command::StopBot)
        }
        510 => {
            c.need(
                lab.active
                    .as_ref()
                    .is_some_and(|a| !a.observer.bot_running())
                    && (session.elapsed - c.frozen).abs() < 0.001
                    && c.mission == Some(session.progress().phase()),
                "sessão do laboratório não progride na campanha",
            );
            if let Some(dir) = &options.captures {
                c.need(
                    std::fs::read_to_string(dir.join("inteligencia-presenca.txt")).is_ok_and(
                        |data| data.contains("MEMÓRIA") && data.contains("Resultado PASS"),
                    ),
                    "exportação do diagnóstico",
                );
            }
            None
        }
        554 => {
            c.need(
                lab.active.is_none() && session.elapsed > c.frozen,
                "sair devolveu a sessão",
            );
            let pass = c.errors.is_empty();
            let report = format!(
                "AINDA VIVO v{VERSION}\nInteligência Windows: {}\nQuadros: {f}\nEstalos físicos: {}\nAtraso desde E: {:?}\nDesvio do bot: {:.2} m\nFalhas: {:?}\n",
                if pass { "PASS" } else { "FAIL" },
                c.pulses,
                c.relay_time
                    .zip(c.arm_time)
                    .map(|(sound, armed)| sound - armed),
                c.excursion,
                c.errors,
            );
            info!("{report}");
            if let Some(path) = std::env::args_os()
                .collect::<Vec<_>>()
                .windows(2)
                .find(|pair| pair[0] == "--smoke-report")
                .map(|pair| pair[1].clone())
                && let Err(err) = std::fs::write(path, &report)
            {
                error!("{err}");
                exit.write(AppExit::error());
                return;
            }
            exit.write(if pass {
                AppExit::Success
            } else {
                AppExit::error()
            });
            None
        }
        _ => None,
    };
    if let Some(action) = action {
        actions.write(Action::Observer(action));
    }
}
