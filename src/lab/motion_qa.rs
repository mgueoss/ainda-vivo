use super::{
    Action, Active, Knob, Lab, Options,
    library::{self, Catalog, Player, Viewer},
    motion::{APPROACH_VIEW_Z, EventKind, Mode, MotionEvent, TRACK_START},
    room::{Guide, LabCamera},
    settings::{Config, Preset, Tab},
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::path::{Path, PathBuf};

const FPS_SHOT: &str = "06-aproximacao-lanterna";
const SHOTS: &[&str] = &[
    "01-ruptura",
    "02-maos-no-chao",
    "03-passada-quadrupede",
    "04-despedida",
    "05-pista",
    FPS_SHOT,
    "07-linha-do-tempo",
    "08-banco",
];

#[derive(Default, Resource)]
pub(super) struct Check {
    frame: u32,
    ticks: u32,
    report: Option<PathBuf>,
    failures: Vec<&'static str>,
    events: Vec<EventKind>,
    track_distance: f32,
    track_yaw: f32,
    nearest: f32,
}

impl Check {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args_os().collect();
        Self {
            report: args
                .windows(2)
                .find(|p| p[0] == "--smoke-report")
                .map(|p| PathBuf::from(&p[1])),
            nearest: f32::INFINITY,
            ..default()
        }
    }

    fn require(&mut self, condition: bool, label: &'static str) {
        if !condition {
            self.failures.push(label);
            error!("Movimentos: {label}");
        }
    }
}

pub(super) fn open(check: Option<Res<Check>>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    if let Some(check) = check {
        match check.frame {
            1 | 1499 => {
                keys.press(KeyCode::ControlLeft);
                keys.press(KeyCode::KeyI);
            }
            2 | 1500 => {
                keys.release(KeyCode::KeyI);
                keys.release(KeyCode::ControlLeft);
            }
            _ => {}
        }
    }
}

fn aim(active: &mut Active, config: &mut Config, camera: Vec3, target: Vec3) {
    let direction = (target - camera).normalize();
    active.orbit = None;
    config.lens.free = true;
    config.lens.position = camera.to_array();
    config.lens.yaw = (-direction.x).atan2(-direction.z);
    config.lens.pitch = direction.y.asin();
    config.lens.fov = 48.0;
}

fn png(dir: Option<&Path>, commands: &mut Commands, label: &str) {
    if let Some(dir) = dir {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(dir.join(format!("{label}.png"))));
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Scene<'w, 's> {
    players: Query<'w, 's, (&'static AnimationPlayer, &'static Player)>,
    viewers: Query<'w, 's, &'static Viewer>,
    actors: Query<'w, 's, (&'static Viewer, &'static GlobalTransform)>,
    guides: Query<'w, 's, (&'static Guide, &'static Visibility)>,
    camera: Query<'w, 's, &'static GlobalTransform, With<LabCamera>>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Independent app, scene and test observers."
)]
pub(super) fn verify(
    check: Option<ResMut<Check>>,
    mut lab: ResMut<Lab>,
    mut config: ResMut<Config>,
    catalog: Res<Catalog>,
    options: Res<Options>,
    scene: Scene,
    mut events: MessageReader<MotionEvent>,
    mut actions: MessageWriter<Action>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(check) = check else { return };
    let check = check.into_inner();
    check.ticks += 1;
    let dir = options.captures.as_deref();
    let video = dir
        .filter(|_| options.record_motion)
        .map(|base| base.join("video"));
    if check.ticks == 1 {
        check.require(
            dir.is_none_or(|path| std::fs::create_dir_all(path).is_ok()),
            "diretório PNG",
        );
        check.require(
            video
                .as_ref()
                .is_none_or(|path| std::fs::create_dir_all(path).is_ok()),
            "diretório de gravação",
        );
    }
    if catalog.clips().is_empty() && check.ticks < 800 {
        return;
    }
    if catalog.clips().is_empty() {
        check.require(false, "catálogo GLB carregado");
        check.frame = 1519;
    }
    check.frame += 1;
    check.events.extend(events.read().map(|event| event.kind));
    let frame = check.frame;
    let selected = scene
        .actors
        .iter()
        .find(|(view, _)| view.index == config.selected);
    let timeline = || library::timeline(config.selected, &catalog, &scene.players, &scene.viewers);
    let select = |actions: &mut MessageWriter<Action>, id, fraction| {
        if let Some(index) = catalog.number(id) {
            actions.write(Action::Clip(index));
            actions.write(Action::SeekFraction(fraction));
        }
    };
    match frame {
        8 => {
            check.require(lab.active.is_some(), "Ctrl+I abre o Dev Lab");
            check.require(
                catalog.clips().len() == 39 && (8..=39).all(|id| catalog.number(id).is_some()),
                "39 clipes e 32 metadados compatíveis com os tempos importados",
            );
            actions.write(Action::SetTab(Tab::Animation));
            actions.write(Action::Contacts);
            select(&mut actions, 28, 0.25);
        }
        15 => {
            check.require(
                timeline().is_some_and(|(_, at, total)| {
                    (at - 0.125).abs() < 0.001 && (total - 0.50).abs() < 0.001
                }),
                "busca de posição com amostra de meio segundo",
            );
            check.require(
                scene.guides.iter().any(|(guide, visible)| {
                    *guide == Guide::Planted && *visible == Visibility::Inherited
                }),
                "apoios do esqueleto importado visíveis",
            );
            actions.write(Action::Step(1));
        }
        18 => {
            check.require(
                timeline().is_some_and(|(_, at, _)| (at - 0.125 - 1.0 / 30.0).abs() < 0.001),
                "avanço pausa e percorre exatamente um quadro de origem",
            );
            check.require(
                check.events.is_empty(),
                "busca e avanço não disparam eventos",
            );
            actions.write(Action::Step(-1));
        }
        22 => {
            check.require(
                timeline().is_some_and(|(_, at, _)| (at - 0.125).abs() < 0.001),
                "retorno de um quadro",
            );
            actions.write(Action::Rewind);
            actions.write(Action::Adjust(Knob::Playback, -10.0));
            actions.write(Action::Play);
        }
        44 => {
            check.require(
                timeline().is_some_and(|(_, at, _)| (0.03..0.04).contains(&at)),
                "reprodução em 0,1×",
            );
            actions.write(Action::Adjust(Knob::Playback, 10.0));
        }
        52 => {
            check.require(
                timeline().is_some_and(|(_, at, _)| (0.41..0.44).contains(&at)),
                "reprodução em 3×",
            );
            if let Some(index) = catalog.number(27) {
                actions.write(Action::Clip(index));
            }
        }
        53 => {
            let blended = scene.players.iter().any(|(player, _)| {
                let active: Vec<_> = player
                    .playing_animations()
                    .map(|(_, a)| a.weight())
                    .collect();
                active.len() == 2 && (active.into_iter().sum::<f32>() - 1.0).abs() < 0.005
            });
            check.require(blended, "mistura de duas ações com pesos normalizados");
            if let Some(index) = catalog.number(28) {
                actions.write(Action::Clip(index));
            }
        }
        63 => {
            check.require(
                scene
                    .players
                    .iter()
                    .all(|(p, _)| p.playing_animations().count() == 1),
                "interrupção limpa os clipes anteriores",
            );
            actions.write(Action::Play);
            actions.write(Action::CompareClips);
        }
        78 => {
            check.require(
                scene.viewers.iter().count() == 2 && scene.players.iter().count() == 2,
                "comparação A/B com duas instâncias e relógios próprios",
            );
            actions.write(Action::Reset);
        }
        90 => {
            let Some(active) = &mut lab.active else {
                return;
            };
            config.hidden = true;
            config.lighting = Preset::Studio.into();
            aim(
                active,
                &mut config,
                Vec3::new(2.37, 1.32, 2.23),
                Vec3::new(0.0, 0.94, 0.10),
            );
            select(&mut actions, 22, 0.53);
        }
        104 => png(dir, &mut commands, SHOTS[0]),
        116 => {
            let Some(active) = &mut lab.active else {
                return;
            };
            actions.write(Action::Contacts);
            aim(
                active,
                &mut config,
                Vec3::new(2.13, 0.97, 1.65),
                Vec3::new(0.0, 0.54, 0.01),
            );
            select(&mut actions, 22, 0.98);
        }
        130 => png(dir, &mut commands, SHOTS[1]),
        142 => select(&mut actions, 28, 0.33),
        156 => png(dir, &mut commands, SHOTS[2]),
        170 => {
            let Some(active) = &mut lab.active else {
                return;
            };
            aim(
                active,
                &mut config,
                Vec3::new(0.76, 1.64, 2.91),
                Vec3::new(0.0, 1.13, 0.0),
            );
            select(&mut actions, 33, 0.50);
        }
        184 => png(dir, &mut commands, SHOTS[3]),
        188 => {
            let Some(active) = &mut lab.active else {
                return;
            };
            aim(
                active,
                &mut config,
                Vec3::new(2.64, 1.22, 2.20),
                Vec3::new(0.0, 0.78, 0.0),
            );
            select(&mut actions, 20, 0.92);
        }
        194 => png(dir, &mut commands, SHOTS[7]),
        196 => {
            actions.write(Action::Adjust(Knob::Playback, 1.0));
            actions.write(Action::MotionDemo(Mode::Track));
        }
        370 => {
            let Some(active) = &mut lab.active else {
                return;
            };
            let center = selected.map_or(TRACK_START, |(_, transform)| transform.translation());
            aim(
                active,
                &mut config,
                center + Vec3::new(-1.42, 1.25, 2.70),
                center + Vec3::Y * 0.59,
            );
        }
        380 => png(dir, &mut commands, SHOTS[4]),
        730 => {
            check.require(
                check.track_distance > 9.0 && check.track_yaw > 0.25,
                "deslocamento na pista e mudança de direção",
            );
            check.require(
                lab.active
                    .as_ref()
                    .and_then(|a| a.playback.run.as_ref())
                    .is_some_and(|r| r.complete),
                "pista completa desacelera e retorna à postura em pé",
            );
            check.require(
                [
                    EventKind::FootLeft,
                    EventKind::FootRight,
                    EventKind::HandLeft,
                    EventKind::HandRight,
                    EventKind::RuptureStart,
                    EventKind::SprintStart,
                    EventKind::Impact,
                    EventKind::Finished,
                ]
                .iter()
                .all(|kind| check.events.contains(kind)),
                "eventos de mãos, pés, ações e impactos recebidos no jogo",
            );
            actions.write(Action::Clip(catalog.number(35).unwrap_or(0)));
            actions.write(Action::Play);
        }
        790 => {
            check.require(
                check.events.contains(&EventKind::Contact),
                "evento de gesto de contato",
            );
            actions.write(Action::Reset);
        }
        798 => {
            config.hidden = true;
            config.lighting = Preset::Station.into();
            config.lighting.torch = true;
            actions.write(Action::MotionDemo(Mode::Approach));
        }
        1105 => png(dir, &mut commands, FPS_SHOT),
        1455 => {
            let done = lab
                .active
                .as_ref()
                .and_then(|a| a.playback.run.as_ref())
                .is_some_and(|r| r.complete);
            check.require(
                done && (1.85..3.0).contains(&check.nearest),
                "aproximação termina antes do operador",
            );
            check.require(
                scene
                    .camera
                    .single()
                    .is_ok_and(|camera| (camera.translation().z - APPROACH_VIEW_Z).abs() < 0.01),
                "demonstração mantém câmera e operador no lugar",
            );
            actions.write(Action::StopDemo);
            actions.write(Action::Reset);
        }
        1462 => {
            actions.write(Action::SetTab(Tab::Animation));
            actions.write(Action::Contacts);
            select(&mut actions, 28, 0.41);
        }
        1476 => png(dir, &mut commands, SHOTS[6]),
        1478 => {
            let Some(active) = &mut lab.active else {
                return;
            };
            aim(
                active,
                &mut config,
                Vec3::new(2.50, 1.50, 3.7),
                Vec3::Y * 1.15,
            );
            actions.write(Action::FollowGaze);
        }
        1491 => {
            check.require(
                timeline()
                    .is_some_and(|(index, at, _)| catalog.number(12) == Some(index) && at > 2.0),
                "a posição do operador conduz a amostra de vigília",
            );
            actions.write(Action::MotionDemo(Mode::Sequence));
        }
        1497 => {
            check.require(
                timeline()
                    .is_some_and(|(index, at, _)| catalog.number(9) == Some(index) && at > 0.01),
                "modo sequência escolhe e reproduz o estado de observação",
            );
        }
        1519 => check.require(lab.active.is_none(), "Ctrl+I restaura a tela anterior"),
        1520 => {
            check.require(
                dir.is_none_or(|path| {
                    SHOTS.iter().all(|name| {
                        path.join(format!("{name}.png"))
                            .metadata()
                            .is_ok_and(|m| m.len() > 1000)
                    })
                }),
                "capturas completas de quadros do Bevy",
            );
            check.require(
                video.as_ref().is_none_or(|path| {
                    (0..180).all(|i| path.join(format!("{i:04}.png")).is_file())
                }),
                "180 quadros consecutivos da gravação",
            );
            let pass = check.failures.is_empty();
            let report = format!(
                "AINDA VIVO v{}\nMovimentos no Dev Lab: {}\nQuadros: {frame}\nEventos: {}\nPista: {:.3} m, maior guinada {:.3} rad\nAproximação: {:.3} m de distância mínima entre inserções\nFalhas: {:?}\n",
                crate::app::VERSION,
                if pass { "PASS" } else { "FAIL" },
                check.events.len(),
                check.track_distance,
                check.track_yaw,
                check.nearest,
                check.failures,
            );
            info!("{report}");
            if let Some(path) = &check.report
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
    if let Some(run) = lab
        .active
        .as_ref()
        .and_then(|active| active.playback.run.as_ref())
    {
        match run.mode {
            Mode::Track => {
                check.track_distance = check.track_distance.max(run.distance);
                if let Some((_, yaw)) = run.path() {
                    check.track_yaw = check.track_yaw.max(yaw.abs());
                }
            }
            Mode::Approach => {
                if let Some((_, transform)) = selected {
                    check.nearest = check
                        .nearest
                        .min(APPROACH_VIEW_Z - transform.translation().z);
                }
                if (802..=1160).contains(&frame) && frame % 2 == 0 {
                    png(
                        video.as_deref(),
                        &mut commands,
                        &format!("{:04}", (frame - 802) / 2),
                    );
                }
            }
            Mode::Sequence => {}
        }
    }
}
