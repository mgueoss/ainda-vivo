//! Bounded render/integration check; never enabled for a normal play session.
use crate::{
    app::*,
    atmosphere::{Atmosphere, Echo, sound::Emitter},
    scene::station::{self, EXIT_GATE, PANEL},
    vhs::VhsMode,
};
use bevy::{
    app::AppExit,
    audio::AudioSinkPlayback,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
use std::{io::Write, path::PathBuf};

#[derive(Resource, Default)]
pub struct SmokeTest {
    pub enabled: bool,
    pub check_audio: bool,
    frame: u32,
    paused_position: Vec3,
    failures: Vec<&'static str>,
    report: Option<String>,
    capture_dir: Option<PathBuf>,
    captures: u32,
    waypoint: usize,
    layout: (usize, usize),
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
    cameras: Query<&Transform, With<MainCamera>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    atmosphere: Res<Atmosphere>,
    world: Res<CollisionWorld>,
    voices: Query<(&Emitter, &AudioSink)>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if !smoke.enabled {
        return;
    }
    smoke.frame += 1;
    if let Some(directory) = &smoke.capture_dir {
        let label = match smoke.frame {
            8 => Some("menu"),
            90 => Some("corredor"),
            175 => Some("pausa"),
            209 => Some("oficina"),
            1860 => Some("eletrica"),
            3160 => Some("nave-vhs"),
            3173 => Some("nave-limpa"),
            3187 => Some("nave-vhs-forte"),
            3206 => Some("decantacao"),
            3224 => Some("arquivo"),
            3242 => Some("carga"),
            _ => None,
        };
        if let Some(label) = label {
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
            restart(&mut session, &mut player, &mut comfort);
        }
        11..=99 => {
            player.command.movement = Vec2::Y;
        }
        100 => {
            if !player.body.grounded || player.body.feet.z >= 7.9 {
                smoke.failures.push("movement_or_grounding");
            }
            player.command.movement = Vec2::X;
        }
        101..=159 => {
            player.command.movement = Vec2::X;
        }
        160 => {
            if player.body.feet.x > 1.34 {
                smoke.failures.push("corridor_wall_collision");
            }
            session.screen = Screen::Paused;
            smoke.paused_position = player.body.feet;
        }
        185 => {
            if player.body.feet != smoke.paused_position {
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
        210 => {
            keys.press(KeyCode::KeyE);
        }
        211 => {
            keys.release(KeyCode::KeyE);
            if !session.progress.has_fuse {
                smoke.failures.push("fuse_interaction_ray");
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
            if !matches!(atmosphere.echo, Echo::Hush { .. }) {
                smoke.failures.push("ambience_cue_advanced_during_pause");
            }
            if smoke.check_audio
                && (voices.is_empty() || voices.iter().any(|(_, sink)| !sink.is_paused()))
            {
                smoke.failures.push("ambience_audio_pause");
            }
            session.screen = Screen::Playing;
        }
        351 => {
            if !matches!(atmosphere.echo, Echo::Heard { .. }) {
                smoke.failures.push("workshop_echo_trigger");
            }
            if smoke.check_audio
                && !voices.iter().any(|(emitter, sink)| {
                    emitter.is_echo() && !sink.is_paused() && !sink.position().is_zero()
                })
            {
                smoke.failures.push("workshop_spatial_playback");
            }
        }
        368 => {
            // A ray aimed through the external wall must not actuate the panel.
            place(
                &mut player,
                Vec3::new(36.02, 0.02, PANEL.z),
                std::f32::consts::FRAC_PI_2,
                -0.20,
            );
            keys.press(KeyCode::KeyE);
        }
        369 => {
            keys.release(KeyCode::KeyE);
            if session.progress.power_restored {
                smoke.failures.push("interaction_through_wall");
            }
        }
        380 => {
            place(&mut player, Vec3::new(0.0, 0.002, 3.8), 0.0, 0.0);
            smoke.waypoint = 0;
        }
        381..=1849 => follow(&mut player, &mut smoke.waypoint, TO_PANEL),
        1850 => {
            if smoke.waypoint != TO_PANEL.len() || player.body.feet.xz().distance(PANEL.xz()) > 2.1
            {
                error!(
                    "Trajeto da sala elétrica parou em {:?}, ponto {}",
                    player.body.feet, smoke.waypoint
                );
                smoke.failures.push("walk_to_electrical_room");
            }
            turn_to(&mut player, PANEL);
        }
        1900 => {
            turn_to(&mut player, PANEL);
            keys.press(KeyCode::KeyE);
        }
        1901 => {
            keys.release(KeyCode::KeyE);
            if !session.progress.power_restored {
                smoke.failures.push("panel_interaction_ray");
            }
            smoke.waypoint = 0;
        }
        1960 => {
            if atmosphere.relief < 0.90 || atmosphere.echo != Echo::Finished {
                smoke.failures.push("exit_light_recovery");
            }
        }
        1961..=3100 => {
            if session.screen == Screen::Playing {
                follow(&mut player, &mut smoke.waypoint, TO_EXIT);
            }
        }
        3110 => {
            if session.screen != Screen::Complete {
                error!(
                    "Trajeto da saída parou em {:?}, ponto {}",
                    player.body.feet, smoke.waypoint
                );
                smoke.failures.push("completion");
            }
            if cameras
                .iter()
                .any(|c| !c.translation.is_finite() || !c.rotation.is_finite())
            {
                smoke.failures.push("camera_finite");
            }
            restart(&mut session, &mut player, &mut comfort);
        }
        3150 => {
            if session.progress.has_fuse || session.progress.power_restored {
                smoke.failures.push("restart");
            }
            if atmosphere.echo != Echo::Waiting || atmosphere.relief > 0.0 {
                smoke.failures.push("ambience_restart");
            }
            comfort.diagnostics = false;
            comfort.vhs = VhsMode::Soft;
            session.message_left = 0.0;
            place(&mut player, Vec3::new(-2.9, 0.002, -28.9), 0.40, -0.02);
        }
        3164 => comfort.vhs = VhsMode::Off,
        3177 => comfort.vhs = VhsMode::Strong,
        3194 => {
            comfort.vhs = VhsMode::Soft;
            place(&mut player, Vec3::new(5.66, 0.002, -66.15), 0.63, -0.17);
        }
        3216 => place(&mut player, Vec3::new(18.15, 0.002, -0.85), -1.30, -0.06),
        3234 => place(&mut player, Vec3::new(-16.6, 0.002, 2.45), 1.12, -0.03),
        3280 => {
            if smoke.capture_dir.is_some() && smoke.captures != 11 {
                smoke.failures.push("capture_completion");
            }
            let passed = smoke.failures.is_empty();
            let report = format!(
                "AINDA VIVO v{VERSION}\nrender/integration smoke: {}\nframes: {}\nmap audit: {} sectors, {} bidirectional portals (closed gate isolated)\nchecks: full walking route, movement, grounding, wall collision, pause, crouch, fuse ray + E, wall occlusion, panel ray + E, gate traversal, exit, restart, finite camera, workshop cue, paused cue, lighting recovery, ambience restart, VHS mode capture\nspatial audio playback/pause check requested: {}\nfailures: {:?}\n",
                if passed { "PASS" } else { "FAIL" },
                smoke.frame,
                smoke.layout.0,
                smoke.layout.1,
                smoke.check_audio,
                smoke.failures
            );
            info!("{report}");
            if let Some(path) = &smoke.report
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
        _ => (),
    }
}

const TO_PANEL: &[Vec2] = &[
    Vec2::new(0.0, -18.8),
    Vec2::new(0.0, -52.5),
    Vec2::new(29.6, -52.5),
    Vec2::new(29.6, PANEL.z),
    Vec2::new(PANEL.x - 1.61, PANEL.z),
];

const TO_EXIT: &[Vec2] = &[
    Vec2::new(24.0, PANEL.z),
    Vec2::new(24.0, -75.0),
    Vec2::new(8.0, -75.0),
    Vec2::new(8.0, -82.83),
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

fn turn_to(player: &mut LocalPlayer, target: Vec3) {
    let direction = target - (player.body.feet + Vec3::Y * player.eye_height);
    player.yaw = (-direction.x).atan2(-direction.z);
    player.pitch = direction.y.atan2(direction.xz().length());
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
