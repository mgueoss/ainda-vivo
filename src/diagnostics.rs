use crate::app::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use std::io::Write;

#[derive(Resource, Default)]
pub struct Performance {
    pub text: String,
    since: f32,
    initialized: bool,
    log: Option<std::io::BufWriter<std::fs::File>>,
}

pub fn update(
    time: Res<Time>,
    real: Res<Time<Real>>,
    store: Res<DiagnosticsStore>,
    player: Res<LocalPlayer>,
    mut performance: ResMut<Performance>,
    smoke: Res<crate::smoke::SmokeTest>,
    session: Res<Session>,
) {
    if !smoke.enabled {
        if !performance.initialized {
            performance.initialized = true;
            let directory = std::env::var_os("AINDA_VIVO_DATA_DIR")
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var_os("LOCALAPPDATA")
                        .map(|p| std::path::PathBuf::from(p).join("AindaVivo"))
                });
            if let Some(directory) = directory {
                let directory = directory.join("sessions");
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                if std::fs::create_dir_all(&directory).is_ok()
                    && let Ok(file) = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(directory.join(format!("{stamp}.csv")))
                {
                    let mut writer = std::io::BufWriter::new(file);
                    let _ = writeln!(writer, "elapsed_s,frame_ms,playing");
                    performance.log = Some(writer);
                }
            }
        }
        if let Some(log) = performance.log.as_mut() {
            let _ = writeln!(
                log,
                "{:.4},{:.4},{}",
                real.elapsed_secs_f64(),
                real.delta_secs_f64() * 1000.0,
                session.screen == Screen::Playing
            );
        }
    }
    performance.since += time.delta_secs();
    if performance.since < 0.25 {
        return;
    }
    performance.since = 0.0;
    if let Some(log) = performance.log.as_mut() {
        let _ = log.flush();
    }
    let fps = store
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let ms = store
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let p = player.body.feet;
    performance.text = format!(
        "{fps:.0} FPS  |  {ms:.1} ms\nFísica 60 Hz  |  contatos {}\n{:.1}, {:.2}, {:.1}  |  {}\nF3 ocultar",
        player.body.contacts,
        p.x,
        p.y,
        p.z,
        if player.body.crouched {
            "AGACHADO"
        } else if player.body.grounded {
            "NO CHÃO"
        } else {
            "NO AR"
        }
    );
}
