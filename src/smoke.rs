//! Bounded integration check for the full inspection, including the return and the exit.
pub mod continuity;
use crate::{
    app::*,
    atmosphere::{
        Atmosphere, Echo,
        director::Director,
        sound::{Emitter, EpisodeCue},
    },
    scene::{
        illusion::{self, Illusions, Impression},
        presence::{self, Presence, Silhouette},
        recording::{self, Recording},
        station::{self, EXIT_GATE, FixtureMotion, PANEL, PUMP_02_PANEL},
        water::Ripple,
    },
    vhs::VhsMode,
};
use ainda_vivo_core::story::Phase;
use bevy::{
    app::AppExit,
    audio::AudioSinkPlayback,
    ecs::system::SystemParam,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
use std::{io::Write, path::PathBuf};

const END_ROUTE: u32 = 4510;
const END_STAGE: u32 = 8860;
const CONTROL_FRAMES: u32 = 3400;

#[derive(Resource, Default)]
pub struct SmokeTest {
    pub enabled: bool,
    pub reality: bool,
    pub check_audio: bool,
    frame: u32,
    paused_position: Vec3,
    paused_fixtures: [Option<(Entity, Quat)>; 2],
    failures: Vec<&'static str>,
    report: Option<String>,
    capture_dir: Option<PathBuf>,
    captures: u32,
    waypoint: usize,
    layout: (usize, usize),
    sightlines: usize,
    sightings: usize,
    heard_incidents: u8,
    heard_episode: u8,
    breaker: Quat,
    needle: Quat,
    passage: bool,
    wake: bool,
    pause_director: (f32, f32, [u32; 3]),
    director_stats: ([u32; 3], u32, u32, u32, u8),
    framed_at: Option<u32>,
    departed: bool,
    illusion_verified: bool,
    illusion_reset: bool,
    log_read: bool,
    archive_read: bool,
    lens_verified: bool,
}

impl SmokeTest {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args().collect();
        let reality = args.iter().any(|a| a == "--smoke-reality");
        Self {
            enabled: reality || args.iter().any(|a| a == "--smoke-test"),
            reality,
            check_audio: args.iter().any(|a| a == "--check-audio"),
            report: args
                .windows(2)
                .find(|a| a[0] == "--smoke-report")
                .map(|a| a[1].clone()),
            capture_dir: args
                .windows(2)
                .find(|a| a[0] == "--capture-dir")
                .map(|a| PathBuf::from(&a[1])),
            ..default()
        }
    }

    fn capture(&self, commands: &mut Commands, label: &str) {
        let Some(directory) = &self.capture_dir else {
            return;
        };
        let path = directory.join(format!("{label}.bmp"));
        commands.spawn(Screenshot::primary_window()).observe(
            move |capture: On<ScreenshotCaptured>, mut result: ResMut<SmokeTest>| {
                match write_bitmap(&capture.image, &path) {
                    Ok(()) => {
                        result.captures += 1;
                        info!("Captura de validação: {}", path.display());
                    }
                    Err(error) => {
                        error!("Falha na captura: {error}");
                        result.failures.push("render_capture");
                    }
                }
            },
        );
    }

    fn reached(&mut self, player: &LocalPlayer, route: &[Vec2], failure: &'static str) {
        if self.waypoint != route.len() {
            error!(
                "Trajeto {failure} parou em {:?}, ponto {}",
                player.body.feet, self.waypoint
            );
            self.failures.push(failure);
        }
    }

    fn finish(&mut self, mut exit: MessageWriter<AppExit>) {
        if self.capture_dir.is_some() && self.captures != 21 {
            self.failures.push("capture_completion");
        }
        if !self.archive_read || !self.lens_verified {
            self.failures.push("archive_occurrence_or_false_indicator");
        }
        if self.check_audio && (self.heard_incidents.count_ones() < 2 || self.heard_episode != 3) {
            self.failures.push("spatial_audio_playback");
        }
        let passed = self.failures.is_empty();
        let report = format!(
            "AINDA VIVO v{VERSION}\nrender/integration smoke: {}\nframes: {}\nmap audit: {} sectors, {} bidirectional portals\npresence: {} valid sightlines, {} distinct framed sites on the main route, walkway crossing: {}, offscreen disappearance: {}\ndirector (first attempt): cues [presence, sound, illusion] {:?}, expired empty windows {}, protected or contextual deferrals {}, sector revisits {}, mood bit mask {:04b}\nchecks: workshop cue, collision and pause, fuse, first pump reading, outside-wall ray, rearm, locked gate, pressure needle, second reading, full walking route, exit, optional monitor, restart, direct-gaze hold and close flashlight\nbasin ripple: {}\nControl shift log: {}\nobserver-local prop shifted: {}, restored on restart: {}\naudio device checked: {}\ndistinct incidental sounds: {}\ninspection and basin sounds played: {} of 2\nBMP captures: {}\nfailures: {:?}\n",
            if passed { "PASS" } else { "FAIL" },
            self.frame,
            self.layout.0,
            self.layout.1,
            self.sightlines,
            self.sightings,
            self.passage,
            self.departed,
            self.director_stats.0,
            self.director_stats.1,
            self.director_stats.2,
            self.director_stats.3,
            self.director_stats.4,
            self.wake,
            self.log_read,
            self.illusion_verified,
            self.illusion_reset,
            self.check_audio,
            self.heard_incidents.count_ones(),
            self.heard_episode.count_ones(),
            self.captures,
            self.failures,
        );
        let report = format!(
            "{report}Archive occurrence read: {}\nControl false indicator applied: {}\n",
            self.archive_read, self.lens_verified
        );
        info!("{report}");
        if let Some(path) = &self.report
            && let Err(error) = std::fs::write(path, &report)
        {
            error!("Não foi possível gravar relatório: {error}");
            exit.write(AppExit::error());
            return;
        }
        exit.write(if passed {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}

#[derive(SystemParam)]
pub struct SceneViews<'w, 's> {
    cameras: Query<'w, 's, (&'static Transform, &'static DistanceFog), With<MainCamera>>,
    silhouettes: Query<'w, 's, &'static Visibility, With<Silhouette>>,
    lantern: Query<'w, 's, (&'static Transform, &'static Visibility), With<Flashlight>>,
    fixtures: Query<'w, 's, (Entity, &'static FixtureMotion, &'static Transform)>,
    waves: Query<'w, 's, &'static Visibility, With<Ripple>>,
    recording: Res<'w, Recording>,
    director: Res<'w, Director>,
    illusions: Res<'w, Illusions>,
    impressions: Query<'w, 's, (&'static Transform, &'static Impression)>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy injects independent test system parameters."
)]
pub fn drive(
    mut smoke: ResMut<SmokeTest>,
    mut session: ResMut<Session>,
    mut player: ResMut<LocalPlayer>,
    mut comfort: ResMut<Comfort>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    atmosphere: Res<Atmosphere>,
    presence: Res<Presence>,
    world: Res<CollisionWorld>,
    voices: Query<(&Emitter, &AudioSink)>,
    scene: SceneViews,
    mut commands: Commands,
    exit: MessageWriter<AppExit>,
) {
    if !smoke.enabled || smoke.reality {
        return;
    }
    smoke.frame += 1;
    if smoke.check_audio {
        for (emitter, sink) in &voices {
            if sink.is_paused() || sink.position().is_zero() {
                continue;
            }
            if let Some(kind) = emitter.incident_kind() {
                smoke.heard_incidents |= 1 << kind;
            }
            match emitter.episode_cue() {
                Some(EpisodeCue::Pump) => smoke.heard_episode |= 1,
                Some(EpisodeCue::Rail) => smoke.heard_episode |= 2,
                None => (),
            }
        }
    }
    // Additional focused checks after completing the route. Restart checks
    // below retain their own per-attempt timeline.
    match smoke.frame {
        5330..=6509 if !smoke.passage && presence.crossing(session.elapsed) => {
            smoke.capture(&mut commands, "passarela-vulto");
            smoke.passage = true;
        }
        6510 => {
            place(
                &mut player,
                illusion::LENS.with_y(0.002) + Vec3::Z * 1.60,
                0.0,
                0.0,
            );
            turn_to(&mut player, illusion::LENS);
            comfort.vhs = VhsMode::Off;
        }
        6600 => smoke.capture(&mut commands, "ramal-antes"),
        6620 => player.yaw += std::f32::consts::PI,
        8420 => {
            smoke.lens_verified = scene
                .impressions
                .iter()
                .any(|(pose, prop)| pose.translation == illusion::LENS && prop.is_applied());
            turn_to(&mut player, illusion::LENS);
        }
        8440 => smoke.capture(&mut commands, "ramal-depois"),
        8510 => {
            place(
                &mut player,
                station::ARCHIVE_LOG.with_y(0.002) + Vec3::Z * 1.16,
                0.0,
                0.0,
            );
            turn_to(&mut player, station::ARCHIVE_LOG);
        }
        8530 => keys.press(KeyCode::KeyE),
        8531 => {
            keys.release(KeyCode::KeyE);
            smoke.archive_read =
                session.message.contains("03:12") && session.progress().phase() == Phase::Exit;
        }
        8550 => smoke.capture(&mut commands, "arquivo-ocorrencia"),
        _ => (),
    }
    let stage = if smoke.frame < 5330 {
        smoke.frame
    } else if smoke.frame < 5330 + CONTROL_FRAMES {
        return;
    } else {
        smoke.frame - CONTROL_FRAMES
    };
    let label = match stage {
        8 => Some("menu"),
        90 => Some("corredor"),
        175 => Some("pausa"),
        209 => Some("oficina"),
        565 => Some("triagem-percurso"),
        1174 => Some("bomba-repouso"),
        2118 => Some("quadro-rearmado"),
        3035 => Some("bomba-retorno"),
        4540 => Some("gravacao"),
        6690 => Some("ficha-antes"),
        8250 => Some("ficha-depois"),
        8500 => Some("lanterna-ligada"),
        8540 => Some("lanterna-desligada"),
        8600 => Some("nave-alcance"),
        _ => None,
    };
    if let Some(label) = label {
        smoke.capture(&mut commands, label);
    }

    match stage {
        10 => {
            match station::audit_routes(&world) {
                Ok(layout) => smoke.layout = layout,
                Err(error) => {
                    error!("{error}");
                    smoke.failures.push("station_layout");
                }
            }
            match presence::audit_sites(&world) {
                Ok(count) => smoke.sightlines = count,
                Err(site) => {
                    error!("Ponto de aparição sem linha de visão: {site}");
                    smoke.failures.push("presence_site_sightline");
                }
            }
            if !matches!(scene.cameras.single(), Ok((_, fog))
                if fog.color == crate::scene::DISTANCE_DARK
                && matches!(fog.falloff, FogFalloff::Linear { start, end }
                    if start == crate::scene::DISTANCE_START && end == crate::scene::DISTANCE_END))
            {
                smoke.failures.push("distance_darkness_config");
            }
            smoke.paused_fixtures = background_poses(&scene.fixtures);
            restart(&mut session, &mut player, &mut comfort);
        }
        11..=99 => player.command.movement = Vec2::Y,
        100 => {
            if !player.body.grounded || player.body.feet.z >= 7.9 {
                smoke.failures.push("movement_or_grounding");
            }
            player.command.movement = Vec2::X;
        }
        101..=159 => player.command.movement = Vec2::X,
        160 => {
            if player.body.feet.x > 1.34 {
                smoke.failures.push("corridor_wall_collision");
            }
            session.screen = Screen::Paused;
            smoke.paused_position = player.body.feet;
            smoke.pause_director = (
                session.elapsed,
                scene.director.hush,
                scene.director.tally.cues,
            );
            let moving = background_poses(&scene.fixtures);
            if moving
                .iter()
                .zip(smoke.paused_fixtures)
                .any(|(current, initial)| {
                    !matches!((current, initial), (Some((a, now)), Some((b, was)))
                    if *a == b && now.angle_between(was).abs() > 0.03)
                })
            {
                smoke.failures.push("background_movement");
            }
            smoke.paused_fixtures = moving;
        }
        185 => {
            if player.body.feet != smoke.paused_position
                || background_poses(&scene.fixtures) != smoke.paused_fixtures
                || smoke.pause_director
                    != (
                        session.elapsed,
                        scene.director.hush,
                        scene.director.tally.cues,
                    )
            {
                smoke.failures.push("pause_simulation");
            }
            session.screen = Screen::Playing;
            player.command.movement = Vec2::ZERO;
            player.command.crouch = true;
        }
        208 => {
            if !player.body.crouched {
                smoke.failures.push("crouch");
            }
            player.command.crouch = false;
            place(
                &mut player,
                Vec3::new(-3.3, 0.002, 4.0),
                std::f32::consts::FRAC_PI_2,
                -0.34,
            );
        }
        210 | 1180 | 2045 | 2064 | 2145 | 3040 | 4520 | 4563 => keys.press(KeyCode::KeyE),
        211 => {
            keys.release(KeyCode::KeyE);
            if session.progress().phase() != Phase::Baseline {
                smoke.failures.push("fuse_interaction");
            }
        }
        212..=278 => {
            player.yaw = 0.0;
            player.pitch = 0.0;
            player.command.movement = Vec2::X;
        }
        279 => {
            if !matches!(atmosphere.echo, Echo::Hush { .. }) {
                smoke.failures.push("return_through_workshop_door");
            }
            player.command.movement = Vec2::ZERO;
            session.screen = Screen::Paused;
        }
        304 => {
            if !matches!(atmosphere.echo, Echo::Hush { .. })
                || (smoke.check_audio
                    && (voices.is_empty() || voices.iter().any(|(_, sink)| !sink.is_paused())))
            {
                smoke.failures.push("workshop_pause");
            }
            session.screen = Screen::Playing;
        }
        351 => {
            if !matches!(atmosphere.echo, Echo::Heard { .. })
                || (smoke.check_audio
                    && !voices.iter().any(|(emitter, sink)| {
                        emitter.is_echo() && !sink.is_paused() && !sink.position().is_zero()
                    }))
            {
                smoke.failures.push("workshop_spatial_playback");
            }
        }
        380 => {
            place(&mut player, Vec3::new(0.0, 0.002, 3.8), 0.0, 0.0);
            smoke.waypoint = 0;
        }
        381..=1159 => follow(&mut player, &mut smoke.waypoint, TO_PUMP),
        1160 => {
            smoke.reached(&player, TO_PUMP, "walk_to_first_reading");
            turn_to(&mut player, PUMP_02_PANEL);
            smoke.needle = fixture_pose(&scene.fixtures, |f| {
                matches!(f, FixtureMotion::PressureNeedle { .. })
            })
            .unwrap_or_default();
            if atmosphere.episode.pressure != 0.0 {
                smoke.failures.push("pressure_during_baseline");
            }
        }
        1181 => {
            keys.release(KeyCode::KeyE);
            if session.progress().phase() != Phase::Electrical {
                smoke.failures.push("baseline_interaction");
            }
            smoke.waypoint = 0;
        }
        1182..=2039 => follow(&mut player, &mut smoke.waypoint, TO_PANEL),
        2040 => {
            smoke.reached(&player, TO_PANEL, "walk_to_electrical_room");
            smoke.breaker = fixture_pose(&scene.fixtures, |f| matches!(f, FixtureMotion::Breaker))
                .unwrap_or_default();
            place(&mut player, Vec3::new(36.02, 0.02, PANEL.z), 0.0, 0.0);
            turn_to(&mut player, PANEL);
        }
        2046 => {
            keys.release(KeyCode::KeyE);
            if session.progress().phase() != Phase::Electrical {
                smoke.failures.push("panel_ray_through_wall");
            }
            place(&mut player, PANEL_VIEW, 0.0, 0.0);
            turn_to(&mut player, PANEL);
        }
        2065 => {
            keys.release(KeyCode::KeyE);
            if session.progress().phase() != Phase::Return {
                smoke.failures.push("rearm_interaction");
            }
        }
        2134 => {
            if !fixture_pose(&scene.fixtures, |f| matches!(f, FixtureMotion::Breaker))
                .is_some_and(|now| now.angle_between(smoke.breaker).abs() > 0.55)
                || atmosphere.relief > 0.0
                || session.progress().exit_unlocked()
            {
                smoke.failures.push("rearm_should_leave_gate_locked");
            }
            place(&mut player, Vec3::new(0.0, 0.002, -84.43), 0.0, 0.0);
            turn_to(&mut player, EXIT_GATE);
        }
        2146 => {
            keys.release(KeyCode::KeyE);
            if session.progress().phase() != Phase::Return
                || !session.message.contains("retorno")
                || !world.occludes(
                    player.body.feet + Vec3::Y * 1.62,
                    EXIT_GATE.with_z(-90.0),
                    session.progress().exit_unlocked(),
                )
            {
                smoke.failures.push("gate_blocks_before_return");
            }
        }
        2180 => {
            place(&mut player, PANEL_VIEW, 0.0, 0.0);
            smoke.waypoint = 0;
        }
        2181..=3019 => follow(&mut player, &mut smoke.waypoint, BACK_TO_PUMP),
        3020 => {
            smoke.reached(&player, BACK_TO_PUMP, "walk_to_return_reading");
            turn_to(&mut player, PUMP_02_PANEL);
            if atmosphere.episode.pressure < 0.76
                || !fixture_pose(&scene.fixtures, |f| {
                    matches!(f, FixtureMotion::PressureNeedle { .. })
                })
                .is_some_and(|now| now.angle_between(smoke.needle).abs() > 1.8)
            {
                smoke.failures.push("pressure_does_not_rise_on_return");
            }
        }
        3041 => {
            keys.release(KeyCode::KeyE);
            if !session.progress().exit_unlocked() {
                smoke.failures.push("return_interaction");
            }
            smoke.waypoint = 0;
        }
        3090..=4509 if session.screen == Screen::Playing => {
            follow(&mut player, &mut smoke.waypoint, TO_EXIT);
            let feet = player.body.feet;
            if feet.x > 6.5 && (-81.9..=-65.0).contains(&feet.z) {
                player.command.sprint = false;
            }
            if !smoke.passage && presence.crossing(session.elapsed) {
                smoke.capture(&mut commands, "passarela-vulto");
                smoke.passage = true;
            }
            if !smoke.wake
                && scene
                    .waves
                    .iter()
                    .any(|visible| *visible == Visibility::Visible)
                && let Some(trace) = atmosphere.episode.trace
                && trace.age >= 1.85
            {
                aim_at(&mut player, trace.water_position());
                smoke.capture(&mut commands, "onda-na-bacia");
                smoke.wake = true;
            }
        }
        END_ROUTE => {
            if session.screen != Screen::Complete || atmosphere.relief < 0.90 {
                error!(
                    "Trajeto da saída parou em {:?}, ponto {}",
                    player.body.feet, smoke.waypoint
                );
                smoke.failures.push("completion");
            }
            smoke.sightings = presence.observed_count();
            if smoke.sightings < 1 || !smoke.wake {
                smoke.failures.push("presence_and_basin_on_walking_route");
            }
            if scene
                .cameras
                .iter()
                .any(|(c, _)| !c.translation.is_finite() || !c.rotation.is_finite())
            {
                smoke.failures.push("camera_finite");
            }
            session.screen = Screen::Playing;
            comfort.diagnostics = false;
            place(
                &mut player,
                Vec3::new(recording::TERMINAL.x, 0.002, -81.42),
                0.0,
                0.0,
            );
            turn_to(&mut player, recording::TERMINAL);
        }
        4521 => {
            keys.release(KeyCode::KeyE);
            if !session.progress().recording_reviewed() || !scene.recording.displayed() {
                smoke.failures.push("recording_monitor_interaction");
            }
        }
        4552 => {
            place(
                &mut player,
                station::SHIFT_LOG.with_y(0.002) + Vec3::Z * 1.02,
                0.0,
                0.0,
            );
            turn_to(&mut player, station::SHIFT_LOG);
        }
        4564 => {
            keys.release(KeyCode::KeyE);
            smoke.log_read =
                session.message.contains("02:48") && session.progress().phase() == Phase::Exit;
            if !smoke.log_read {
                smoke.failures.push("control_shift_log");
            }
        }
        4580 => {
            place(
                &mut player,
                presence::WALKWAY_WITNESS.with_y(0.002),
                0.0,
                0.0,
            );
            turn_to(&mut player, presence::WALKWAY_MARK + Vec3::Y * 1.74);
        }
        4581..=5329 if !smoke.passage && presence.crossing(session.elapsed) => {
            smoke.capture(&mut commands, "passarela-vulto");
            smoke.passage = true;
        }
        5330 => {
            let tally = &scene.director.tally;
            smoke.director_stats = (
                tally.cues,
                tally.empty,
                tally.deferred,
                tally.revisits,
                tally.moods,
            );
            if !smoke.passage
                || tally.cues[0] == 0
                || tally.cues[1] == 0
                || tally.empty == 0
                || tally.revisits == 0
                || tally.moods & 0b1111 != 0b1111
            {
                smoke.failures.push("director_main_route_and_crossing");
            }
            restart(&mut session, &mut player, &mut comfort);
        }
        5350 => {
            if session.progress().phase() != Phase::Fuse
                || atmosphere.echo != Echo::Waiting
                || atmosphere.relief > 0.0
                || atmosphere.episode.pressure > 0.0
                || scene.recording.displayed()
                || scene.director.attempt != session.attempt
                || scene.director.tally.cues != [0; 3]
                || scene.illusions.altered != 0
            {
                smoke.failures.push("restart_resets_episode");
            }
            comfort.diagnostics = false;
            comfort.vhs = VhsMode::Soft;
            session.message_left = 0.0;
            place(
                &mut player,
                presence::FIRST_WITNESS.with_y(0.002),
                std::f32::consts::PI,
                0.0,
            );
        }
        5390 => {
            if presence.active() || !matches!(scene.silhouettes.single(), Ok(Visibility::Hidden)) {
                smoke.failures.push("presence_outside_view");
            }
            turn_to(&mut player, presence::FIRST_MARK + Vec3::Y * 1.59);
        }
        5391..=6619 => {
            if smoke.framed_at.is_none()
                && presence.observed_count() > 0
                && matches!(scene.silhouettes.single(), Ok(Visibility::Visible))
            {
                smoke.framed_at = Some(smoke.frame);
            }
            if let Some(frame) = smoke.framed_at {
                if smoke.frame == frame + 64 {
                    if !presence.active()
                        || !matches!(scene.silhouettes.single(), Ok(Visibility::Visible))
                    {
                        smoke.failures.push("presence_visible_under_direct_gaze");
                    }
                    smoke.capture(&mut commands, "triagem-vulto-vhs");
                } else if smoke.frame == frame + 70 {
                    player.yaw += std::f32::consts::PI;
                    player.pitch = 0.0;
                } else if smoke.frame == frame + 105 {
                    if presence.retired_unseen() != 1 || presence.active() {
                        smoke.failures.push("offscreen_disappearance");
                    }
                    player.yaw -= std::f32::consts::PI;
                    turn_to(&mut player, presence::FIRST_MARK + Vec3::Y * 1.59);
                } else if smoke.frame == frame + 114 {
                    smoke.departed = presence.retired_unseen() == 1
                        && matches!(scene.silhouettes.single(), Ok(Visibility::Hidden));
                    smoke.capture(&mut commands, "triagem-sem-vulto");
                }
            }
        }
        6670 => {
            if !smoke.departed {
                smoke.failures.push("presence_and_unseen_retirement");
            }
            comfort.vhs = VhsMode::Off;
            place(
                &mut player,
                illusion::FILE.with_y(0.002) + Vec3::Z * 1.84,
                0.0,
                0.0,
            );
            turn_to(&mut player, illusion::FILE);
        }
        6700 => {
            if scene.illusions.altered != 0 {
                smoke.failures.push("illusion_changes_under_gaze");
            }
            player.yaw += std::f32::consts::PI;
            player.pitch = 0.0;
        }
        8200 => {
            smoke.illusion_verified = scene.illusions.altered == 1
                && scene.director.tally.cues[2] == 1
                && scene.impressions.iter().any(|(pose, prop)| {
                    prop.is_applied()
                        && (0.24..0.35).contains(&pose.translation.distance(illusion::FILE))
                });
            if !smoke.illusion_verified {
                smoke
                    .failures
                    .push("illusion_does_not_shift_behind_observer");
            }
            turn_to(&mut player, illusion::FILE);
        }
        8310 => restart(&mut session, &mut player, &mut comfort),
        8330 => {
            smoke.illusion_reset = scene.illusions.altered == 0
                && scene.illusions.attempt == session.attempt
                && scene.impressions.iter().all(|(_, prop)| !prop.is_applied())
                && scene
                    .impressions
                    .iter()
                    .any(|(pose, _)| pose.translation == illusion::FILE);
            if !smoke.illusion_reset {
                smoke.failures.push("illusion_does_not_reset");
            }
        }
        8470 => {
            session.message_left = 0.0;
            place(&mut player, Vec3::new(0.20, 0.002, -8.72), 0.025, -0.135);
        }
        8500 => {
            if !matches!((scene.cameras.single(), scene.lantern.single()), (Ok((camera, _)), Ok((beam, Visibility::Visible)))
                if beam.translation == camera.translation && beam.rotation == camera.rotation)
            {
                smoke.failures.push("flashlight_center_alignment");
            }
        }
        8515 => comfort.flashlight = false,
        8540 => {
            if !matches!(scene.lantern.single(), Ok((_, Visibility::Hidden))) {
                smoke.failures.push("flashlight_toggle");
            }
        }
        8580 => {
            comfort.flashlight = true;
            place(&mut player, Vec3::new(-0.93, 0.002, -30.5), 0.0, -0.070);
        }
        END_STAGE => smoke.finish(exit),
        _ => (),
    }
}

fn background_poses(
    fixtures: &Query<(Entity, &FixtureMotion, &Transform)>,
) -> [Option<(Entity, Quat)>; 2] {
    let mut poses = [None; 2];
    for (id, fixture, pose) in fixtures {
        let index = match fixture {
            FixtureMotion::Fan { .. } => 0,
            FixtureMotion::Hanging { .. } => 1,
            FixtureMotion::Breaker | FixtureMotion::PressureNeedle { .. } => continue,
        };
        poses[index].get_or_insert((id, pose.rotation));
    }
    poses
}

fn fixture_pose(
    fixtures: &Query<(Entity, &FixtureMotion, &Transform)>,
    filter: impl Fn(&FixtureMotion) -> bool,
) -> Option<Quat> {
    fixtures
        .iter()
        .find_map(|(_, fixture, pose)| filter(fixture).then_some(pose.rotation))
}

const INSPECTION_VIEW: Vec2 = Vec2::new(PUMP_02_PANEL.x - 1.66, PUMP_02_PANEL.z);
const PANEL_VIEW: Vec3 = Vec3::new(PANEL.x - 1.61, 0.002, PANEL.z);

const TO_PUMP: &[Vec2] = &[Vec2::new(0.0, PUMP_02_PANEL.z), INSPECTION_VIEW];
const TO_PANEL: &[Vec2] = &[
    Vec2::new(3.67, -50.11),
    Vec2::new(3.67, -52.5),
    Vec2::new(29.6, -52.5),
    Vec2::new(29.6, PANEL.z),
    Vec2::new(PANEL_VIEW.x, PANEL_VIEW.z),
];
const BACK_TO_PUMP: &[Vec2] = &[
    Vec2::new(29.6, PANEL.z),
    Vec2::new(29.6, -52.5),
    Vec2::new(3.67, -52.5),
    Vec2::new(3.55, PUMP_02_PANEL.z),
    INSPECTION_VIEW,
];
const TO_EXIT: &[Vec2] = &[
    Vec2::new(3.44, -44.0),
    Vec2::new(1.12, -57.4),
    Vec2::new(0.4, -61.65),
    Vec2::new(7.85, -61.65),
    Vec2::new(7.85, -82.83),
    Vec2::new(0.0, -82.83),
    Vec2::new(0.0, EXIT_GATE.z - 10.0),
];

fn follow(player: &mut LocalPlayer, step: &mut usize, route: &[Vec2]) {
    let current = player.body.feet.xz();
    if route
        .get(*step)
        .is_some_and(|target| current.distance(*target) < 0.27)
    {
        *step += 1;
    }
    if let Some(target) = route.get(*step) {
        let direction = *target - current;
        player.yaw = (-direction.x).atan2(-direction.y);
        player.pitch = 0.0;
        player.command.yaw = player.yaw;
        player.command.movement = Vec2::Y;
        player.command.sprint = direction.length_squared() > 3.2;
    } else {
        player.command = default();
    }
}

fn aim_at(player: &mut LocalPlayer, target: Vec3) {
    let direction = target - (player.body.feet + Vec3::Y * player.eye_height);
    player.yaw = (-direction.x).atan2(-direction.z);
    player.pitch = direction.y.atan2(direction.xz().length());
}

fn turn_to(player: &mut LocalPlayer, target: Vec3) {
    aim_at(player, target);
    player.command = default();
}

// Uncompressed BMP needs no extra image codec/dependency. Only the optional
// validation command writes captures; normal play never touches these paths.
fn write_bitmap(image: &Image, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let rgb = image.clone().try_into_dynamic()?.to_rgb8();
    let (width, height) = rgb.dimensions();
    let stride = (width * 3 + 3) & !3;
    let size = stride * height;
    let mut header = vec![0_u8; 54];
    header[0..2].copy_from_slice(b"BM");
    header[2..6].copy_from_slice(&(54 + size).to_le_bytes());
    header[10..14].copy_from_slice(&54_u32.to_le_bytes());
    header[14..18].copy_from_slice(&40_u32.to_le_bytes());
    header[18..22].copy_from_slice(&width.to_le_bytes());
    header[22..26].copy_from_slice(&(-(height as i32)).to_le_bytes());
    header[26..28].copy_from_slice(&1_u16.to_le_bytes());
    header[28..30].copy_from_slice(&24_u16.to_le_bytes());
    header[34..38].copy_from_slice(&size.to_le_bytes());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    file.write_all(&header)?;
    let mut row = vec![0_u8; stride as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel = rgb.get_pixel(x, y).0;
            let offset = x as usize * 3;
            row[offset..offset + 3].copy_from_slice(&[pixel[2], pixel[1], pixel[0]]);
        }
        file.write_all(&row)?;
    }
    file.flush()?;
    Ok(())
}

fn place(player: &mut LocalPlayer, feet: Vec3, yaw: f32, pitch: f32) {
    player.body = ainda_vivo_core::movement::PlayerBody {
        feet,
        previous_feet: feet,
        grounded: true,
        ..default()
    };
    player.command = default();
    player.yaw = yaw;
    player.pitch = pitch;
    player.eye_height = 1.62;
}
