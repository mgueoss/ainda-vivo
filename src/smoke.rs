//! Bounded render/integration check; never enabled for a normal play session.
use crate::app::*;
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
use std::{io::Write, path::PathBuf};

#[derive(Resource, Default)]
pub struct SmokeTest {
    pub enabled: bool,
    frame: u32,
    paused_position: Vec3,
    failures: Vec<&'static str>,
    report: Option<String>,
    capture_dir: Option<PathBuf>,
    captures: u32,
}

impl SmokeTest {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args().collect();
        Self {
            enabled: args.iter().any(|a| a == "--smoke-test"),
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
        10 => restart(&mut session, &mut player, &mut comfort),
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
        210 => {
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
            keys.press(KeyCode::KeyE);
        }
        211 => {
            keys.release(KeyCode::KeyE);
            if !session.progress.has_fuse {
                smoke.failures.push("fuse_interaction_ray");
            }
        }
        220 => {
            // Try the panel from the other side of the corridor wall.
            place(
                &mut player,
                Vec3::new(2.3, 0.002, -8.6),
                std::f32::consts::FRAC_PI_2,
                -0.3,
            );
            keys.press(KeyCode::KeyE);
        }
        221 => {
            keys.release(KeyCode::KeyE);
            if session.progress.power_restored {
                smoke.failures.push("interaction_through_wall");
            }
        }
        230 => {
            place(
                &mut player,
                Vec3::new(-0.2, 0.002, -8.6),
                -std::f32::consts::FRAC_PI_2,
                -0.2,
            );
            keys.press(KeyCode::KeyE);
        }
        231 => {
            keys.release(KeyCode::KeyE);
            if !session.progress.power_restored {
                smoke.failures.push("panel_interaction_ray");
            }
        }
        240 => {
            place(&mut player, Vec3::new(0.0, 0.002, -8.3), 0.0, 0.0);
            player.command.movement = Vec2::Y;
        }
        241..=359 => {
            if session.screen == Screen::Playing {
                player.command.movement = Vec2::Y;
            }
        }
        380 => {
            if session.screen != Screen::Complete {
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
        400 => {
            if session.progress.has_fuse || session.progress.power_restored {
                smoke.failures.push("restart");
            }
            if smoke.capture_dir.is_some() && smoke.captures != 3 {
                smoke.failures.push("capture_completion");
            }
            let passed = smoke.failures.is_empty();
            let report = format!(
                "AINDA VIVO v{VERSION}\nrender/integration smoke: {}\nframes: {}\nchecks: movement, grounding, wall collision, pause, crouch, fuse ray + E, wall occlusion, panel ray + E, gate traversal, exit, restart, finite camera\nfailures: {:?}\n",
                if passed { "PASS" } else { "FAIL" },
                smoke.frame,
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
