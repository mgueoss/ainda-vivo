use crate::app::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use std::{io::Write, path::Path};

const SESSION_FILES: usize = 8;
const SESSION_ROWS: usize = 180_000;

#[derive(Resource, Default)]
pub struct Performance {
    pub text: String,
    since: f32,
    initialized: bool,
    log: Option<std::io::BufWriter<std::fs::File>>,
    rows: usize,
}

fn retain_sessions(directory: &Path, current: u128) {
    let Ok(files) = std::fs::read_dir(directory) else {
        return;
    };
    let mut previous = files
        .filter_map(Result::ok)
        .filter_map(|entry| {
            if !entry.file_type().ok()?.is_file() {
                return None;
            }
            let path = entry.path();
            if path.extension()?.to_str()? != "csv" {
                return None;
            }
            let name = path.file_stem()?.to_str()?;
            if !(18..=20).contains(&name.len()) {
                return None;
            }
            let stamp = name.parse::<u128>().ok()?;
            (stamp < current).then_some((stamp, path))
        })
        .collect::<Vec<_>>();
    previous.sort_unstable_by_key(|(stamp, _)| std::cmp::Reverse(*stamp));
    for (_, path) in previous.into_iter().skip(SESSION_FILES - 1) {
        let _ = std::fs::remove_file(path);
    }
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
                    retain_sessions(&directory, stamp);
                }
            }
        }
        if let Some(log) = performance.log.as_mut() {
            let stored = writeln!(
                log,
                "{:.4},{:.4},{}",
                real.elapsed_secs_f64(),
                real.delta_secs_f64() * 1000.0,
                session.screen == Screen::Playing
            )
            .is_ok();
            performance.rows += usize::from(stored);
            if !stored || performance.rows >= SESSION_ROWS {
                performance.log = None;
            }
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
