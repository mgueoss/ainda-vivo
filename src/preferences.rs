//! Local, human-readable settings. No network or new dependencies.
use crate::app::Comfort;
use bevy::prelude::*;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Resource)]
pub struct Preferences {
    path: Option<PathBuf>,
    saved: String,
    bindings: BTreeMap<String, KeyCode>,
}

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

pub fn save(comfort: Res<Comfort>, mut preferences: ResMut<Preferences>) {
    let Some(path) = preferences.path.clone() else {
        return;
    };
    let mut content = format!(
        "# AINDA VIVO - preferencias locais\nsensitivity={}\nfov={}\nbodycam={}\nvsync={}\ndiagnostics={}\nambience_volume={:.2}\nambience_muted={}\n",
        comfort.sensitivity,
        comfort.fov_degrees,
        comfort.bodycam,
        comfort.vsync,
        comfort.diagnostics,
        comfort.ambience_volume,
        comfort.ambience_muted,
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
    let result = (|| -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &content)
    })();
    if let Err(error) = result {
        warn!("Preferências não salvas: {error}");
    }
    preferences.saved = content;
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "ainda-vivo-settings-{}-{}.cfg",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn settings_round_trip_preserves_preferences_and_bindings() {
        let path = path();
        std::fs::write(&path, "fov=75\nsensitivity=0.0024\nbodycam=true\nvsync=false\nambience_volume=0.42\nambience_muted=true\nkey.forward=Up\nkey.interact=Q\n").unwrap();
        let (preferences, comfort) = Preferences::load_from(Some(path.clone()));
        let mut app = App::new();
        app.insert_resource(preferences)
            .insert_resource(comfort)
            .add_systems(Update, save);
        app.update();
        let (loaded, comfort) = Preferences::load_from(Some(path.clone()));
        assert_eq!(comfort.fov_degrees, 75.0);
        assert_eq!(comfort.sensitivity, 0.0024);
        assert!(comfort.bodycam && !comfort.vsync);
        assert!(comfort.ambience_muted);
        assert_eq!(comfort.ambience_volume, 0.42);
        assert_eq!(loaded.key("forward", KeyCode::KeyW), KeyCode::ArrowUp);
        assert_eq!(loaded.key("interact", KeyCode::KeyE), KeyCode::KeyQ);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_settings_keep_safe_defaults_and_clamp_ranges() {
        let path = path();
        std::fs::write(
            &path,
            "fov=999\nsensitivity=NaN\nkey.forward=Escape\nbroken line\n",
        )
        .unwrap();
        let (loaded, comfort) = Preferences::load_from(Some(path.clone()));
        assert_eq!(comfort.fov_degrees, 80.0);
        assert_eq!(comfort.sensitivity, Comfort::default().sensitivity);
        assert_eq!(comfort.ambience_volume, Comfort::default().ambience_volume);
        assert_eq!(loaded.key("forward", KeyCode::KeyW), KeyCode::KeyW);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn simulation_test_never_persists_user_settings() {
        let (preferences, _) = Preferences::load(true);
        assert!(preferences.path.is_none());
    }
}
