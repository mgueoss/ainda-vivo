use super::{
    Campaign, HUMAN, HuntCommand, Pose, State,
    hunt::{self, Contact, Cue, End, Hunt, Phase, Signal},
    point,
};
use crate::{
    app::{Comfort, LocalPlayer, Screen, Session, VERSION, awaken, restart},
    lab::{Action, Lab, Options, library::Catalog, observer::Command},
};
use ainda_vivo_core::{
    reality::Reality,
    story::{Action as Mission, Phase as Progress},
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::path::PathBuf;

#[derive(Resource)]
pub(in crate::lab) struct Check {
    frames: u32,
    ticks: u32,
    report: Option<PathBuf>,
    errors: Vec<&'static str>,
    captured: bool,
    captured_pose: Option<(Vec3, f32, f32, bool)>,
    clip: bool,
    labpeak: f32,
    feet: u32,
    hands: u32,
    contact_cue: u32,
    photo: bool,
    labphoto: bool,
}

impl Check {
    pub fn new() -> Self {
        let a: Vec<_> = std::env::args_os().collect();
        Self {
            frames: 0,
            ticks: 0,
            report: a
                .windows(2)
                .find(|p| p[0] == "--smoke-report")
                .map(|p| PathBuf::from(&p[1])),
            errors: vec![],
            captured: false,
            captured_pose: None,
            clip: false,
            labpeak: 0.,
            feet: 0,
            hands: 0,
            contact_cue: 0,
            photo: false,
            labphoto: false,
        }
    }
    fn need(&mut self, ok: bool, label: &'static str) {
        if !ok {
            self.errors.push(label);
            error!("Caçada: {label}");
        }
    }
}

pub(in crate::lab) fn keys(check: Option<Res<Check>>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    if let Some(check) = check {
        if matches!(check.frames, 518 | 1370) {
            keys.press(KeyCode::ControlLeft);
            keys.press(KeyCode::KeyI);
        } else if matches!(check.frames, 519 | 1371) {
            keys.release(KeyCode::ControlLeft);
            keys.release(KeyCode::KeyI);
        }
    }
}

fn capture(options: &Options, commands: &mut Commands, name: &str) {
    if let Some(dir) = &options.captures
        && std::fs::create_dir_all(dir).is_ok()
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(dir.join(name)));
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(in crate::lab) struct Scene<'w, 's> {
    session: ResMut<'w, Session>,
    player: ResMut<'w, LocalPlayer>,
    comfort: ResMut<'w, Comfort>,
    poses: Query<'w, 's, (&'static Pose, &'static Visibility)>,
    signals: MessageReader<'w, 's, Signal>,
}

pub(in crate::lab) fn verify(
    check: ResMut<Check>,
    options: Res<Options>,
    catalog: Res<Catalog>,
    lab: Res<Lab>,
    mut campaign: ResMut<Campaign>,
    mut scene: Scene,
    (mut commands, mut actions, mut exit): (
        Commands,
        MessageWriter<Action>,
        MessageWriter<AppExit>,
    ),
) {
    let c = check.into_inner();
    c.ticks += 1;
    if catalog.number(28).is_none() && c.ticks < 900 {
        return;
    }
    if catalog.number(28).is_none() {
        c.need(false, "importação dos clipes");
        c.frames = 1389;
    }
    c.frames += 1;
    let f = c.frames;
    for signal in scene.signals.read() {
        match signal.cue {
            Cue::Hand => c.hands += 1,
            Cue::Foot => c.feet += 1,
            Cue::Contact => c.contact_cue += 1,
            _ => {}
        }
    }
    c.clip |= scene
        .poses
        .iter()
        .any(|(pose, shown)| pose.clip == 28 && *shown != Visibility::Hidden);
    if f == 10 {
        restart(&mut scene.session, &mut scene.player, &mut scene.comfort);
        for action in [
            Mission::TakeFuse,
            Mission::InspectPump,
            Mission::RestorePower,
        ] {
            scene.session.act(action);
        }
    }
    if f == 42 {
        if let Some(nav) = &campaign.nav {
            let area = [0., 2.55, -2.55, 5.]
                .into_iter()
                .map(|x| (Vec2::new(x, -23.5), Vec2::new(x, -15.3)))
                .find(|&(a, b)| {
                    nav.clear_body(a, b, hunt::half_at(0.), true)
                        && nav.clear_body(a, b, HUMAN, true)
                });
            if let Some((from, to)) = area {
                let start = point(to).with_y(0.022);
                scene.player.body.feet = start;
                scene.player.body.previous_feet = start;
                scene.player.yaw = 0.;
                scene.player.pitch = -0.10;
                campaign.brain.spawn(from, 0.);
                campaign
                    .brain
                    .change(State::Threaten, "ensaio de navegação", 2.0, None);
                campaign.brain.hunt = Hunt::start(hunt::Tuning::default(), to, false);
                campaign.brain.hunt.guided = true;
            } else {
                c.need(false, "corredor real para contato");
            }
        } else {
            c.need(false, "mapa da Estação");
        }
    }
    if (48..485).contains(&f) {
        let body = &scene.player;
        let current = (
            body.body.feet,
            body.yaw,
            body.pitch,
            scene.comfort.flashlight,
        );
        if scene.session.expedition.observer().reality() == Reality::False && !c.captured {
            c.need(
                c.captured_pose.is_some_and(|old| {
                    old.0.distance(current.0) < 0.004
                        && old.1 == current.1
                        && old.2 == current.2
                        && old.3 == current.3
                }),
                "captura mantém posição, câmera e lanterna",
            );
            c.need(
                scene.session.expedition.truth().phase() == Progress::Return
                    && scene.session.expedition.view().phase() == Progress::Return
                    && scene.session.screen == Screen::Playing
                    && scene.session.hunt_effect.tape(scene.session.elapsed) == 0.0,
                "captura em jogo mantém missão e apresentação",
            );
            c.captured = true;
        }
        c.captured_pose = Some(current);
        if campaign.brain.hunt.phase == Phase::Run && campaign.brain.hunt.age > 0.85 && !c.photo {
            capture(&options, &mut commands, "01-cacada-na-estacao.png");
            c.photo = true;
        }
    }
    if let Some(active) = &lab.active {
        let b = &active.observer.brain;
        if (540..900).contains(&f) {
            c.labpeak = c.labpeak.max(b.hunt.peak);
            if b.hunt.phase == Phase::Run && b.hunt.age > 1.05 && !c.labphoto {
                capture(&options, &mut commands, "02-cacada-livre.png");
                c.labphoto = true;
            }
        }
    }
    match f {
        485 => {
            c.need(
                c.captured && c.feet > 0 && c.hands > 0 && c.contact_cue == 0 && c.clip,
                "contato silencioso e marcadores da corrida real",
            );
            scene.session.screen = Screen::Paused;
            scene.session.wake_confirmation = true;
            awaken(&mut scene.session, &mut scene.player);
            c.need(
                scene.session.screen == Screen::Playing
                    && scene.session.expedition.observer().reality() == Reality::Real
                    && !scene.session.expedition.can_capture()
                    && scene.session.expedition.truth().phase() == Progress::Return,
                "Despertar retorna ao ponto real e impede nova captura",
            );
        }
        540 => {
            c.need(lab.active.is_some(), "Ctrl+I");
            actions.write(Action::Hunt(HuntCommand::Result(Some(
                Contact::Disorientation,
            ))));
            actions.write(Action::Hunt(HuntCommand::Start));
        }
        885 => {
            c.need(
                lab.active
                    .as_ref()
                    .is_some_and(|a| a.observer.hunt.last == Some(Contact::Disorientation)),
                "Caçada livre chega ao contato configurado",
            );
            c.need(
                c.labpeak > 4.5,
                "passada intensa supera corrida humana no trecho reto",
            );
            actions.write(Action::Hunt(HuntCommand::Bluff));
        }
        1020 => {
            c.need(
                lab.active
                    .as_ref()
                    .is_some_and(|a| a.observer.brain.hunt.end == Some(End::Bluff)),
                "ameaça desiste após a preparação",
            );
            actions.write(Action::Hunt(HuntCommand::DoorTrial));
        }
        1172 => {
            actions.write(Action::Observer(Command::Door));
        }
        1324 => {
            c.need(
                lab.active.as_ref().is_some_and(|a| {
                    a.observer.closed
                        && a.observer.brain.pos.y < -9.75
                        && matches!(a.observer.brain.hunt.end, Some(End::Lost | End::Route))
                }),
                "porta fecha durante a corrida e interrompe a perseguição",
            );
            capture(&options, &mut commands, "03-porta-e-recuo.png");
            actions.write(Action::Hunt(HuntCommand::Result(Some(Contact::Capture))));
            actions.write(Action::Hunt(HuntCommand::Rehearse));
        }
        1358 => {
            c.need(
                lab.active.as_ref().is_some_and(|a| {
                    a.observer.hunt.session.expedition.observer().reality() == Reality::False
                }) && scene.session.expedition.observer().reality() == Reality::Real,
                "captura do Lab isolada",
            );
        }
        1390 => {
            c.need(
                lab.active.is_none()
                    && scene.session.expedition.truth().phase() == Progress::Return,
                "retorno do Lab preservou missão",
            );
            let pass = c.errors.is_empty();
            let report = format!(
                "AINDA VIVO v{VERSION}\nCaçada Windows: {}\nQuadros: {f}\nPico no Lab: {:.3} m/s\nMarcadores em campanha: {} mãos, {} pés\nFalhas: {:?}\n",
                if pass { "PASS" } else { "FAIL" },
                c.labpeak,
                c.hands,
                c.feet,
                c.errors
            );
            info!("{report}");
            if let Some(path) = &c.report
                && let Err(error) = std::fs::write(path, report)
            {
                error!("{error}");
                exit.write(AppExit::error());
                return;
            }
            exit.write(if pass {
                AppExit::Success
            } else {
                AppExit::error()
            });
        }
        _ => {}
    }
}
