use super::{Campaign, Command as AI, Pose, STATION_ANCHORS, State};
use crate::{
    app::{Comfort, LocalPlayer, Screen, Session, VERSION, restart},
    atmosphere::director::{Cue, Director},
    lab::{Action, Lab, Options, library::Catalog},
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::path::PathBuf;

#[derive(Resource)]
pub(crate) struct Check {
    tick: u32,
    frame: u32,
    report: Option<PathBuf>,
    failed: Vec<&'static str>,
    presented: bool,
    pictured: bool,
    gate_checked: bool,
    walk_clip: bool,
    pause: f32,
}

impl Check {
    pub fn new() -> Self {
        let args: Vec<_> = std::env::args_os().collect();
        Self {
            tick: 0,
            frame: 0,
            report: args
                .windows(2)
                .find(|a| a[0] == "--smoke-report")
                .map(|a| PathBuf::from(&a[1])),
            failed: vec![],
            presented: false,
            pictured: false,
            gate_checked: false,
            walk_clip: false,
            pause: 0.,
        }
    }

    fn check(&mut self, state: bool, name: &'static str) {
        if !state {
            self.failed.push(name);
            error!("Observador: {name}");
        }
    }
}

pub(crate) fn keys(check: Option<Res<Check>>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    if let Some(check) = check {
        if matches!(check.frame, 4 | 8) {
            keys.press(KeyCode::F5);
        } else if matches!(check.frame, 5 | 9) {
            keys.release(KeyCode::F5);
        }
        if matches!(check.frame, 1900 | 2860) {
            keys.press(KeyCode::ControlLeft);
            keys.press(KeyCode::KeyI);
        } else if matches!(check.frame, 1901 | 2861) {
            keys.release(KeyCode::KeyI);
            keys.release(KeyCode::ControlLeft);
        }
    }
}

fn photo(options: &Options, commands: &mut Commands, name: &str) {
    if let Some(dir) = &options.captures {
        if let Err(e) = std::fs::create_dir_all(dir) {
            error!("{e}");
            return;
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(dir.join(name)));
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct Game<'w, 's> {
    session: ResMut<'w, Session>,
    player: ResMut<'w, LocalPlayer>,
    comfort: ResMut<'w, Comfort>,
    director: ResMut<'w, Director>,
    poses: Query<'w, 's, (&'static Pose, &'static Visibility)>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Independent renderer and test resources."
)]
pub(crate) fn verify(
    check: ResMut<Check>,
    catalog: Res<Catalog>,
    lab: Res<Lab>,
    mode: Res<Campaign>,
    options: Res<Options>,
    mut game: Game,
    (mut actions, mut exit, mut commands): (
        MessageWriter<Action>,
        MessageWriter<AppExit>,
        Commands,
    ),
) {
    let check = check.into_inner();
    check.tick += 1;
    if catalog.number(12).is_none() && check.tick < 800 {
        return;
    }
    if catalog.number(12).is_none() {
        check.check(false, "GLB indisponível");
        check.frame = 2879;
    }
    check.frame += 1;
    let f = check.frame;
    if (760..1830).contains(&f) && f.is_multiple_of(65) && mode.started == 0 {
        game.director.request(Cue::Presence);
    }
    if mode.brain.state != State::Absent && f < 1830 && !check.presented {
        let aim = mode.brain.pos - game.player.body.feet.xz();
        game.player.yaw = (-aim.x).atan2(-aim.y);
        check.presented = true;
    }
    let moving = game
        .poses
        .iter()
        .any(|(pose, shown)| *shown != Visibility::Hidden && matches!(pose.clip, 13 | 14 | 19));
    check.walk_clip |= moving;
    if f < 1830 && check.presented && !check.pictured && mode.brain.clock - mode.brain.born > 0.45 {
        check.pictured = true;
        photo(&options, &mut commands, "01-estacao-observador.png");
    }
    match f {
        6 => check.check(!mode.physical(), "F5 retorna à representação anterior"),
        11 => check.check(mode.physical(), "F5 rearma a representação física"),
        15 => {
            restart(&mut game.session, &mut game.player, &mut game.comfort);
            let spot = Vec3::new(0., 0.022, -8.8);
            game.player.body.feet = spot;
            game.player.body.previous_feet = spot;
            game.player.yaw = std::f32::consts::PI;
        }
        1850 => {
            check.check(
                mode.physical() && mode.started >= 1 && mode.max_travel > 0.1,
                "janela do diretor iniciou manifestação com deslocamento",
            );
            check.check(
                game.session.screen == Screen::Playing && !game.session.progress().has_fuse(),
                "manifestação não alterou a missão",
            );
            if let Some(nav) = &mode.nav {
                let mut route = vec![];
                check.check(
                    nav.route(Vec2::new(0., -7.5), Vec2::new(0., -27.), true, &mut route)
                        && route
                            .iter()
                            .filter_map(|p| nav.sector(*p))
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                            >= 3,
                    "rota da Estação cruza Acesso, Triagem e Bombas",
                );
                check.check(
                    STATION_ANCHORS
                        .iter()
                        .any(|&a| nav.clear_body(a, a, super::HUMAN, true)),
                    "postos iniciais fisicamente válidos",
                );
            } else {
                check.check(false, "grafo da Estação");
            }
        }
        1920 => {
            check.check(lab.active.is_some(), "atalho do laboratório");
            actions.write(Action::Observer(AI::Enable));
        }
        1980 => {
            if let Some(active) = &lab.active {
                check.check(
                    active.observer.brain.seen.is_some(),
                    "visão livre no corredor",
                );
            }
            actions.write(Action::Observer(AI::Door));
            actions.write(Action::Observer(AI::Reset));
        }
        2000 => {
            actions.write(Action::Observer(AI::Select(State::Investigate)));
            actions.write(Action::Observer(AI::Noise));
        }
        2080 => {
            if let Some(active) = &lab.active {
                let b = &active.observer.brain;
                check.gate_checked = active.observer.closed && b.seen.is_none() && b.pos.y < -9.0;
            }
            check.check(
                check.gate_checked,
                "a porta fecha a linha de visão e impede passagem direta",
            );
            photo(&options, &mut commands, "02-porta-fechada.png");
        }
        2130 => {
            actions.write(Action::Observer(AI::Door));
            actions.write(Action::Observer(AI::Reset));
            actions.write(Action::Observer(AI::Select(State::Investigate)));
            actions.write(Action::Observer(AI::Noise));
        }
        2220 => {
            check.check(
                lab.active.as_ref().is_some_and(|a| {
                    a.observer.brain.heard.is_some()
                        && a.observer.brain.goal.is_some()
                        && !a.observer.brain.path.is_empty()
                }),
                "ruído e rota de investigação no laboratório",
            );
            photo(&options, &mut commands, "03-rota-investigacao.png");
            actions.write(Action::Observer(AI::Pause));
        }
        2230 => {
            if let Some(active) = &lab.active {
                check.pause = active.observer.brain.clock;
            }
        }
        2260 => {
            check.check(
                lab.active
                    .as_ref()
                    .is_some_and(|a| a.observer.brain.clock == check.pause),
                "pausa congela o relógio e os estímulos",
            );
            actions.write(Action::Observer(AI::Pause));
        }
        2840 => {
            check.check(
                lab.active
                    .as_ref()
                    .is_some_and(|a| a.observer.brain.pos.y > -8.8),
                "atravessa a porta aberta",
            );
            check.check(check.walk_clip, "locomoção chamou clipe real");
        }
        2880 => {
            check.check(
                lab.active.is_none()
                    && game.session.screen == Screen::Playing
                    && !game.session.progress().has_fuse(),
                "retorno ao estado da campanha",
            );
            let result = format!(
                "AINDA VIVO v{VERSION}\nObservador Windows: {}\nQuadros: {f}\nManifestações: {}\nDistância percorrida em campanha: {:.2} m\nFalhas: {:?}\n",
                if check.failed.is_empty() {
                    "PASS"
                } else {
                    "FAIL"
                },
                mode.started,
                mode.max_travel,
                check.failed
            );
            info!("{result}");
            if let Some(path) = &check.report
                && let Err(error) = std::fs::write(path, result)
            {
                error!("{error}");
                exit.write(AppExit::error());
                return;
            }
            exit.write(if check.failed.is_empty() {
                AppExit::Success
            } else {
                AppExit::error()
            });
        }
        _ => {}
    }
}
