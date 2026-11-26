//! Local, human-readable settings. No network or new dependencies.
use crate::app::Comfort;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, poll_once},
};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Resource)]
pub struct Preferences {
    path: Option<PathBuf>,
    saved: String,
    bindings: BTreeMap<String, KeyCode>,
    pending: Option<SaveTask>,
    retry_at: f64,
    delay: f64,
}

type SaveTask = Task<(String, std::io::Result<()>)>;

impl Preferences {
    pub fn load(smoke: bool) -> (Self, Comfort) {
        let path = if smoke {
            None
        } else {
            std::env::var_os("AINDA_VIVO_DATA_DIR")
                .or_else(|| {
                    std::env::var_os("LOCALAPPDATA")
                        .map(|p| PathBuf::from(p).join("AindaVivo").into_os_string())
                })
                .map(|p| PathBuf::from(p).join("settings.cfg"))
        };
        Self::load_from(path)
    }

    fn load_from(path: Option<PathBuf>) -> (Self, Comfort) {
        let mut comfort = Comfort::default();
        let mut bindings = BTreeMap::new();
        if let Some(source) = path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
            for line in source.lines() {
                let Some((name, value)) = line.split_once('=') else {
                    continue;
                };
                let (name, value) = (name.trim(), value.trim());
                match name {
                    "sensitivity" => {
                        if let Ok(v) = value.parse::<f32>()
                            && v.is_finite()
                        {
                            comfort.sensitivity = v.clamp(0.0004, 0.004);
                        }
                    }
                    "fov" => {
                        if let Ok(v) = value.parse::<f32>()
                            && v.is_finite()
                        {
                            comfort.fov_degrees = v.clamp(55.0, 80.0);
                        }
                    }
                    "bodycam" => comfort.bodycam = value == "true",
                    "vsync" => comfort.vsync = value != "false",
                    "diagnostics" => comfort.diagnostics = value != "false",
                    "ambience_volume" => {
                        if let Ok(volume) = value.parse::<f32>()
                            && volume.is_finite()
                        {
                            comfort.ambience_volume = volume.clamp(0.0, 1.0);
                        }
                    }
                    "ambience_muted" => comfort.ambience_muted = value == "true",
                    "psycho_av" => {
                        if let Ok(intensity) = value.parse::<f32>()
                            && intensity.is_finite()
                        {
                            comfort.psycho_av = intensity.clamp(0.0, 1.0);
                        }
                    }
                    "psycho_motion" => {
                        if let Ok(enabled) = value.parse::<bool>() {
                            comfort.psycho_motion = enabled;
                        }
                    }
                    "psycho_indicator" => {
                        if let Ok(enabled) = value.parse::<bool>() {
                            comfort.psycho_indicator = enabled;
                        }
                    }
                    "vhs" => {
                        if let Some(mode) = crate::vhs::VhsMode::parse(value) {
                            comfort.vhs = mode;
                        }
                    }
                    _ => {
                        if let Some(action) = name.strip_prefix("key.")
                            && let Some(key) = parse_key(value)
                        {
                            bindings.insert(action.to_owned(), key);
                        }
                    }
                }
            }
        }
        (
            Self {
                path,
                saved: String::new(),
                bindings,
                pending: None,
                retry_at: 0.0,
                delay: 1.0,
            },
            comfort,
        )
    }

    pub fn key(&self, action: &str, fallback: KeyCode) -> KeyCode {
        self.bindings.get(action).copied().unwrap_or(fallback)
    }
}

fn parse_key(value: &str) -> Option<KeyCode> {
    Some(match value {
        "W" => KeyCode::KeyW,
        "A" => KeyCode::KeyA,
        "S" => KeyCode::KeyS,
        "D" => KeyCode::KeyD,
        "E" => KeyCode::KeyE,
        "F" => KeyCode::KeyF,
        "C" => KeyCode::KeyC,
        "Q" => KeyCode::KeyQ,
        "R" => KeyCode::KeyR,
        "Z" => KeyCode::KeyZ,
        "X" => KeyCode::KeyX,
        "Up" => KeyCode::ArrowUp,
        "Down" => KeyCode::ArrowDown,
        "Left" => KeyCode::ArrowLeft,
        "Right" => KeyCode::ArrowRight,
        "Space" => KeyCode::Space,
        "Shift" => KeyCode::ShiftLeft,
        "Ctrl" => KeyCode::ControlLeft,
        _ => return None,
    })
}

pub fn save(comfort: Res<Comfort>, time: Res<Time<Real>>, mut preferences: ResMut<Preferences>) {
    let now = time.elapsed_secs_f64();
    if let Some(job) = &mut preferences.pending {
        let Some((written, result)) = block_on(poll_once(job)) else {
            return;
        };
        preferences.pending = None;
        match result {
            Ok(()) => {
                preferences.saved = written;
                preferences.delay = 1.0;
            }
            Err(error) => {
                warn!(
                    "Preferências não salvas. Nova tentativa em {:.0} s: {error}",
                    preferences.delay
                );
                preferences.retry_at = now + preferences.delay;
                preferences.delay = (preferences.delay * 2.0).min(8.0);
                return;
            }
        }
    }
    if now < preferences.retry_at {
        return;
    }
    let Some(path) = preferences.path.clone() else {
        return;
    };
    let mut content = format!(
        "# AINDA VIVO - preferencias locais\nsensitivity={}\nfov={}\nbodycam={}\nvsync={}\ndiagnostics={}\nambience_volume={:.2}\nambience_muted={}\nvhs={}\npsycho_av={:.2}\npsycho_motion={}\npsycho_indicator={}\n",
        comfort.sensitivity,
        comfort.fov_degrees,
        comfort.bodycam,
        comfort.vsync,
        comfort.diagnostics,
        comfort.ambience_volume,
        comfort.ambience_muted,
        comfort.vhs.setting(),
        comfort.psycho_av,
        comfort.psycho_motion,
        comfort.psycho_indicator,
    );
    for (action, fallback, label) in [
        ("forward", KeyCode::KeyW, "W"),
        ("back", KeyCode::KeyS, "S"),
        ("left", KeyCode::KeyA, "A"),
        ("right", KeyCode::KeyD, "D"),
        ("interact", KeyCode::KeyE, "E"),
        ("flashlight", KeyCode::KeyF, "F"),
        ("jump", KeyCode::Space, "Space"),
        ("sprint", KeyCode::ShiftLeft, "Shift"),
        ("crouch", KeyCode::ControlLeft, "Ctrl"),
    ] {
        let key = preferences.key(action, fallback);
        let encoded = match key {
            KeyCode::ArrowUp => "Up".to_owned(),
            KeyCode::ArrowDown => "Down".to_owned(),
            KeyCode::ArrowLeft => "Left".to_owned(),
            KeyCode::ArrowRight => "Right".to_owned(),
            KeyCode::ShiftLeft => "Shift".to_owned(),
            KeyCode::ControlLeft => "Ctrl".to_owned(),
            KeyCode::Space => "Space".to_owned(),
            _ => format!("{key:?}")
                .strip_prefix("Key")
                .unwrap_or(label)
                .to_owned(),
        };
        content.push_str(&format!("key.{action}={encoded}\n"));
    }
    if content == preferences.saved {
        return;
    }
    preferences.pending = Some(IoTaskPool::get().spawn(async move {
        let result = write_settings(&path, &content);
        (content, result)
    }));
}

fn write_settings(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let result = (|| {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&temporary, content)?;
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "ainda-vivo-settings-{}-{}-{}.cfg",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn settings_round_trip_preserves_preferences_and_bindings() {
        let path = path();
        std::fs::write(&path, "fov=75\nsensitivity=0.0024\nbodycam=true\nvsync=false\nambience_volume=0.42\nambience_muted=true\nvhs=off\npsycho_av=0.50\npsycho_motion=false\npsycho_indicator=true\nkey.forward=Up\nkey.interact=Q\n").unwrap();
        let (preferences, comfort) = Preferences::load_from(Some(path.clone()));
        let mut app = App::new();
        app.add_plugins((bevy::app::TaskPoolPlugin::default(), bevy::time::TimePlugin))
            .insert_resource(preferences)
            .insert_resource(comfort)
            .add_systems(Update, save);
        for _ in 0..150 {
            app.update();
            if !app.world().resource::<Preferences>().saved.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .starts_with("# AINDA VIVO")
        );
        let (loaded, comfort) = Preferences::load_from(Some(path.clone()));
        assert_eq!(comfort.fov_degrees, 75.0);
        assert_eq!(comfort.sensitivity, 0.0024);
        assert!(comfort.bodycam && !comfort.vsync);
        assert!(comfort.ambience_muted);
        assert_eq!(comfort.ambience_volume, 0.42);
        assert_eq!(comfort.vhs, crate::vhs::VhsMode::Off);
        assert_eq!(comfort.psycho_av, 0.50);
        assert!(!comfort.psycho_motion && comfort.psycho_indicator);
        assert_eq!(loaded.key("forward", KeyCode::KeyW), KeyCode::ArrowUp);
        assert_eq!(loaded.key("interact", KeyCode::KeyE), KeyCode::KeyQ);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_settings_keep_safe_defaults_and_clamp_ranges() {
        let path = path();
        std::fs::write(
            &path,
            "fov=999\nsensitivity=NaN\npsycho_av=NaN\npsycho_motion=unexpected\npsycho_indicator=unexpected\nkey.forward=Escape\nbroken line\n",
        )
        .unwrap();
        let (loaded, comfort) = Preferences::load_from(Some(path.clone()));
        assert_eq!(comfort.fov_degrees, 80.0);
        assert_eq!(comfort.sensitivity, Comfort::default().sensitivity);
        assert_eq!(comfort.ambience_volume, Comfort::default().ambience_volume);
        assert_eq!(comfort.psycho_av, Comfort::default().psycho_av);
        assert!(comfort.psycho_motion);
        assert!(!comfort.psycho_indicator);
        assert_eq!(loaded.key("forward", KeyCode::KeyW), KeyCode::KeyW);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn simulation_test_never_persists_user_settings() {
        let (preferences, _) = Preferences::load(true);
        assert!(preferences.path.is_none());
    }
}
