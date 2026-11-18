//! Bounded integration check for the full inspection, including the return and the exit.
use crate::{
    app::*,
    atmosphere::{
        Atmosphere, Echo,
        sound::{Emitter, EpisodeCue},
    },
    scene::{
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
const END_TEST: u32 = 5160;

#[derive(Resource, Default)]
pub struct SmokeTest {
    pub enabled: bool,
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
}

impl SmokeTest {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args().collect();
        Self {
            enabled: args.iter().any(|a| a == "--smoke-test"),
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
        if self.capture_dir.is_some() && self.captures != 16 {
            self.failures.push("capture_completion");
        }
        if self.check_audio && (self.heard_incidents.count_ones() < 2 || self.heard_episode != 3) {
            self.failures.push("spatial_audio_playback");
        }
        let passed = self.failures.is_empty();
        let report = format!(
            "AINDA VIVO v{VERSION}\nrender/integration smoke: {}\nframes: {}\nmap audit: {} sectors, {} bidirectional portals\npresence: {} valid sightlines, {} framed sightings on the main route, walkway crossing: {}\nchecks: workshop cue, collision and pause, fuse, first pump reading, outside-wall ray, rearm, locked gate, pressure needle, second reading, full walking route, exit, optional monitor, restart, direct-gaze hold and close flashlight\nbasin ripple: {}\naudio device checked: {}\ndistinct incidental sounds: {}\ninspection and basin sounds played: {} of 2\nfailures: {:?}\n",
            if passed { "PASS" } else { "FAIL" },
            self.frame,
            self.layout.0,
            self.layout.1,
            self.sightlines,
            self.sightings,
            self.passage,
            self.wake,
            self.check_audio,
            self.heard_incidents.count_ones(),
            self.heard_episode.count_ones(),
            self.failures,
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
    if !smoke.enabled {
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
    let label = match smoke.frame {
        8 => Some("menu"),
        90 => Some("corredor"),
        175 => Some("pausa"),
        209 => Some("oficina"),
        565 => Some("triagem-percurso"),
        1174 => Some("bomba-repouso"),
        2118 => Some("quadro-rearmado"),
        3035 => Some("bomba-retorno"),
        4540 => Some("gravacao"),
        4660 => Some("triagem-vulto-vhs"),
        4965 => Some("triagem-sem-vulto"),
        4995 => Some("lanterna-ligada"),
        5020 => Some("lanterna-desligada"),
        5060 => Some("nave-alcance"),
        _ => None,
    };
    if let Some(label) = label {
        smoke.capture(&mut commands, label);
    }

    match smoke.frame {
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
        210 | 1180 | 2045 | 2064 | 2145 | 3040 | 4520 => keys.press(KeyCode::KeyE),
        211 => {
            keys.release(KeyCode::KeyE);
            if session.progress.phase() != Phase::Baseline {
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
            if session.progress.phase() != Phase::Electrical {
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
            if session.progress.phase() != Phase::Electrical {
                smoke.failures.push("panel_ray_through_wall");
            }
            place(&mut player, PANEL_VIEW, 0.0, 0.0);
            turn_to(&mut player, PANEL);
        }
        2065 => {
            keys.release(KeyCode::KeyE);
            if session.progress.phase() != Phase::Return {
                smoke.failures.push("rearm_interaction");
            }
        }
        2134 => {
            if !fixture_pose(&scene.fixtures, |f| matches!(f, FixtureMotion::Breaker))
                .is_some_and(|now| now.angle_between(smoke.breaker).abs() > 0.55)
                || atmosphere.relief > 0.0
                || session.progress.exit_unlocked()
            {
                smoke.failures.push("rearm_should_leave_gate_locked");
            }
            place(&mut player, Vec3::new(0.0, 0.002, -84.43), 0.0, 0.0);
            turn_to(&mut player, EXIT_GATE);
        }
        2146 => {
            keys.release(KeyCode::KeyE);
            if session.progress.phase() != Phase::Return
                || !session.message.contains("retorno")
                || !world.occludes(
                    player.body.feet + Vec3::Y * 1.62,
                    EXIT_GATE.with_z(-90.0),
                    session.progress.exit_unlocked(),
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
            if !session.progress.exit_unlocked() {
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
                && smoke.passage
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
            if smoke.sightings < 3 || !smoke.passage || !smoke.wake {
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
            if !session.progress.recording_reviewed() || !scene.recording.displayed() {
                smoke.failures.push("recording_monitor_interaction");
            }
        }
        4560 => restart(&mut session, &mut player, &mut comfort),
        4580 => {
            if session.progress.phase() != Phase::Fuse
                || atmosphere.echo != Echo::Waiting
                || atmosphere.relief > 0.0
                || atmosphere.episode.pressure > 0.0
                || scene.recording.displayed()
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
        4595 => {
            if presence.active() || !matches!(scene.silhouettes.single(), Ok(Visibility::Hidden)) {
                smoke.failures.push("presence_outside_view");
            }
            turn_to(&mut player, presence::FIRST_MARK + Vec3::Y * 1.59);
        }
        4660 | 4800 => {
            if !presence.active() || !matches!(scene.silhouettes.single(), Ok(Visibility::Visible))
            {
                smoke.failures.push("presence_visible_under_direct_gaze");
            }
        }
        4965 => {
            if presence.active() || !matches!(scene.silhouettes.single(), Ok(Visibility::Hidden)) {
                smoke.failures.push("presence_finishes_visible_hold");
            }
        }
        4980 => {
            comfort.vhs = VhsMode::Off;
            place(&mut player, Vec3::new(0.20, 0.002, -8.72), 0.025, -0.135);
        }
        4995 => {
            if !matches!((scene.cameras.single(), scene.lantern.single()), (Ok((camera, _)), Ok((beam, Visibility::Visible)))
                if beam.translation == camera.translation && beam.rotation == camera.rotation)
            {
                smoke.failures.push("flashlight_center_alignment");
            }
        }
        5005 => comfort.flashlight = false,
        5020 => {
            if !matches!(scene.lantern.single(), Ok((_, Visibility::Hidden))) {
                smoke.failures.push("flashlight_toggle");
            }
        }
        5050 => {
            comfort.flashlight = true;
            place(&mut player, Vec3::new(-0.93, 0.002, -30.5), 0.0, -0.070);
        }
        END_TEST => smoke.finish(exit),
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
