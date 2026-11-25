use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(super) const MAX_INSTANCES: usize = 12;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Model {
    #[default]
    Presence,
    Legacy,
    Pump,
    Door,
    Structure,
    Human,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(super) enum Category {
    #[default]
    All,
    Characters,
    Equipment,
    Structure,
    Reference,
}

impl Category {
    pub const ALL: [(Self, &'static str); 5] = [
        (Self::All, "Tudo"),
        (Self::Characters, "Figuras"),
        (Self::Equipment, "Equipamentos"),
        (Self::Structure, "Estruturas"),
        (Self::Reference, "Referência"),
    ];
}

impl Model {
    pub const ALL: [Self; 6] = [
        Self::Presence,
        Self::Legacy,
        Self::Pump,
        Self::Door,
        Self::Structure,
        Self::Human,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Presence => "A Presença 3D",
            Self::Legacy => "Presença anterior",
            Self::Pump => "Bomba 02",
            Self::Door => "Comporta de saída",
            Self::Structure => "Pilar da estação",
            Self::Human => "Referência humana",
        }
    }

    pub fn category(self) -> Category {
        match self {
            Self::Presence | Self::Legacy => Category::Characters,
            Self::Pump => Category::Equipment,
            Self::Door | Self::Structure => Category::Structure,
            Self::Human => Category::Reference,
        }
    }

    pub fn matches(self, category: Category, search: &str) -> bool {
        (category == Category::All || self.category() == category)
            && fold(self.name()).contains(&fold(search.trim()))
    }
}

fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'ô' | 'õ' => 'o',
            'ú' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Surface {
    #[default]
    Original,
    Clay,
    Silhouette,
}

impl Surface {
    pub fn next(self) -> Self {
        match self {
            Self::Original => Self::Clay,
            Self::Clay => Self::Silhouette,
            Self::Silhouette => Self::Original,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Clay => "Argila neutra",
            Self::Silhouette => "Silhueta opaca",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct ModelSpec {
    pub model: Model,
    pub position: [f32; 3],
    pub angles: [f32; 3],
    pub scale: f32,
    pub visible: bool,
    pub shadow: bool,
    pub bounds: bool,
    pub surface: Surface,
    pub clip: usize,
}

impl Default for ModelSpec {
    fn default() -> Self {
        Self::new(Model::Presence, Vec3::ZERO)
    }
}

impl ModelSpec {
    pub fn new(model: Model, position: Vec3) -> Self {
        Self {
            model,
            position: position.into(),
            angles: [0.0; 3],
            scale: 1.0,
            visible: true,
            shadow: true,
            bounds: false,
            surface: Surface::Original,
            clip: 0,
        }
    }

    pub fn transform(&self) -> Transform {
        let [x, y, z] = self.angles.map(f32::to_radians);
        Transform {
            translation: self.position.into(),
            rotation: Quat::from_euler(EulerRot::YXZ, y, x, z),
            scale: Vec3::splat(self.scale),
        }
    }

    pub fn validate(&mut self) {
        self.position[0] = bounded(self.position[0], -13.0, 13.0, 0.0);
        self.position[1] = bounded(self.position[1], -3.0, 9.0, 0.0);
        self.position[2] = bounded(self.position[2], -19.0, 10.0, 0.0);
        self.angles = self.angles.map(|f| {
            if f.is_finite() {
                (f + 180.0).rem_euclid(360.0) - 180.0
            } else {
                0.0
            }
        });
        self.scale = bounded(self.scale, 0.05, 8.0, 1.0);
        // The clip list comes from the asset and can contain user imports.
        self.clip = self.clip.min(255);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Preset {
    #[default]
    Studio,
    Station,
    Corridor,
    Emergency,
    Torch,
}

impl Preset {
    pub const ALL: [Self; 5] = [
        Self::Studio,
        Self::Station,
        Self::Corridor,
        Self::Emergency,
        Self::Torch,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Studio => "Estúdio claro",
            Self::Station => "Estação 07",
            Self::Corridor => "Corredor escuro",
            Self::Emergency => "Emergência",
            Self::Torch => "Só lanterna",
        }
    }

    pub fn next(self) -> Self {
        Self::ALL[(self as usize + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Lighting {
    pub preset: Preset,
    pub key: f32,
    pub fill: f32,
    pub color: usize,
    pub azimuth: f32,
    pub elevation: f32,
    pub exposure: f32,
    pub fog: bool,
    pub shadow: bool,
    pub torch: bool,
    pub vhs: u8,
}

impl Default for Lighting {
    fn default() -> Self {
        Self::from(Preset::Studio)
    }
}

impl From<Preset> for Lighting {
    fn from(preset: Preset) -> Self {
        let (key, fill, color, azimuth, elevation, exposure, fog, torch, vhs) = match preset {
            Preset::Studio => (16_000., 800., 0, 25., 40., 10., false, false, 0),
            Preset::Station => (1_250., 54., 1, -44., 42., 8.75, true, false, 1),
            Preset::Corridor => (170., 14., 2, 72., 21., 8.75, true, false, 0),
            Preset::Emergency => (1_150., 13., 3, -56., 25., 8.75, true, false, 0),
            Preset::Torch => (0., 0., 0, 35., 53., 8.75, true, true, 0),
        };
        Self {
            preset,
            key,
            fill,
            color,
            azimuth,
            elevation,
            exposure,
            fog,
            shadow: true,
            torch,
            vhs,
        }
    }
}

impl Lighting {
    pub const COLORS: [(&'static str, Color); 5] = [
        ("Neutra", Color::srgb(1.0, 0.966, 0.913)),
        ("Fluorescente", Color::srgb(0.65, 0.86, 0.77)),
        ("Fria", Color::srgb(0.65, 0.77, 1.0)),
        ("Emergência", Color::srgb(1.0, 0.23, 0.128)),
        ("Âmbar", Color::srgb(1.0, 0.74, 0.42)),
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Tab {
    #[default]
    Object,
    Lighting,
    Animation,
    Observer,
    Help,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Lens {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub speed: f32,
    pub free: bool,
}

impl Default for Lens {
    fn default() -> Self {
        Self {
            position: [0., 1.53, 4.2],
            yaw: 0.,
            pitch: -0.085,
            fov: 51.,
            speed: 4.,
            free: false,
        }
    }
}

#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Config {
    pub lighting: Lighting,
    pub lens: Lens,
    pub instances: Vec<ModelSpec>,
    pub selected: usize,
    pub category: Category,
    pub search: String,
    pub tab: Tab,
    pub grid: bool,
    pub skeleton: bool,
    pub wireframe: bool,
    pub hidden: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lighting: Lighting::default(),
            lens: Lens::default(),
            instances: vec![ModelSpec::default()],
            selected: 0,
            category: Category::All,
            search: String::new(),
            tab: Tab::Object,
            grid: true,
            skeleton: false,
            wireframe: false,
            hidden: false,
        }
    }
}

impl Config {
    pub fn validate(&mut self) {
        self.instances.truncate(MAX_INSTANCES);
        self.selected = self.selected.min(self.instances.len().saturating_sub(1));
        self.search = self
            .search
            .chars()
            .take(48)
            .filter(|c| !c.is_control())
            .collect();
        self.instances.iter_mut().for_each(ModelSpec::validate);
        let lens = &mut self.lens;
        let defaults = Lens::default();
        for (i, value) in lens.position.iter_mut().enumerate() {
            *value = bounded(*value, -35.0, 35.0, defaults.position[i]);
        }
        lens.yaw = bounded(lens.yaw, -7.0, 7.0, 0.0);
        lens.pitch = bounded(lens.pitch, -1.52, 1.52, defaults.pitch);
        lens.fov = bounded(lens.fov, 25.0, 95.0, defaults.fov);
        lens.speed = bounded(lens.speed, 0.25, 18.0, defaults.speed);
        let light = &mut self.lighting;
        light.key = bounded(light.key, 0.0, 60_000.0, 16_000.0);
        light.fill = bounded(light.fill, 0.0, 1_000.0, 800.0);
        light.color %= Lighting::COLORS.len();
        light.elevation = bounded(light.elevation, -85.0, 85.0, 53.0);
        light.azimuth = bounded(light.azimuth, -360.0, 360.0, 35.0);
        light.exposure = bounded(light.exposure, 5.0, 14.0, 10.0);
        light.vhs %= 3;
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut settings: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        settings.validate();
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        if let Err(error) = std::fs::rename(&temporary, path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error.into());
        }
        Ok(())
    }

    pub fn selected(&self) -> Option<&ModelSpec> {
        self.instances.get(self.selected)
    }

    pub fn append(&mut self, spec: ModelSpec) -> bool {
        if self.instances.len() == MAX_INSTANCES {
            return false;
        }
        self.instances.push(spec);
        self.selected = self.instances.len() - 1;
        true
    }

    pub fn remove(&mut self) {
        if self.selected < self.instances.len() {
            self.instances.remove(self.selected);
            self.selected = self.selected.min(self.instances.len().saturating_sub(1));
        }
    }

    pub fn next_spot(&self) -> Vec3 {
        // Three columns, then another row so silhouettes remain readable.
        (0..MAX_INSTANCES)
            .map(|i| Vec3::new([0., 1.55, -1.55][i % 3], 0., -2.7 * (i / 3) as f32))
            .find(|candidate| {
                self.instances
                    .iter()
                    .all(|s| candidate.distance(Vec3::from(s.position)) >= 0.85)
            })
            .unwrap_or(Vec3::new(2.9, 0., -1.5))
    }
}

fn bounded(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_and_filter_are_combined() {
        assert!(Model::Presence.matches(Category::Characters, "  presenca "));
        assert!(!Model::Presence.matches(Category::Equipment, "presença"));
        assert!(Model::Pump.matches(Category::All, "02"));
    }

    #[test]
    fn edited_preferences_round_trip_and_clamp() {
        let mut config = Config {
            lighting: Preset::Emergency.into(),
            lens: Lens {
                fov: 35.0,
                ..Lens::default()
            },
            ..Config::default()
        };
        assert!(config.append(ModelSpec::new(Model::Human, Vec3::X * 1.55)));
        config.instances[1].angles = [11., 45., 0.];
        config.wireframe = true;
        let raw = serde_json::to_string(&config).unwrap();
        assert_eq!(serde_json::from_str::<Config>(&raw).unwrap(), config);
        config.instances[1].scale = f32::INFINITY;
        config.lens.speed = 200.;
        config.selected = 100;
        config.validate();
        assert_eq!(
            (
                config.instances[1].scale,
                config.selected,
                config.lens.speed
            ),
            (1., 1, 18.)
        );
    }

    #[test]
    fn empty_scene_and_repeated_duplication_respect_bounds() {
        let mut config = Config::default();
        config.remove();
        config.remove();
        assert!(config.selected().is_none());
        assert_eq!(config.next_spot(), Vec3::ZERO);
        for _ in 0..MAX_INSTANCES {
            let spot = config.next_spot();
            assert!(config.append(ModelSpec::new(Model::Door, spot)));
        }
        assert!(!config.append(ModelSpec::default()));
        assert_eq!(config.selected, MAX_INSTANCES - 1);
    }
}
