use super::{
    Active, Lab,
    observer::{self, State},
    room,
    settings::{Config, Tab},
};
use crate::atmosphere::sound::presence_audio::{
    Category, Echo, Family, Pattern, PresenceEmitter, Request,
};
use bevy::{
    camera::visibility::{NoFrustumCulling, RenderLayers},
    prelude::*,
};

pub(super) type CampaignSinks<'w, 's> =
    Query<'w, 's, (Entity, &'static AudioSink), (Added<AudioSink>, Without<PresenceEmitter>)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Command {
    Play,
    Family(Family),
    Variation,
    Category,
    Position,
    Reverb,
    Occlusion,
    Guides,
    Door,
    Gain(i8),
    Pitch(i8),
    Distance(i8),
    Pattern(Pattern),
    Chase,
    Walk,
    Stop,
}

pub(super) struct SoundLab {
    family: Family,
    category: usize,
    variant: usize,
    gain: f32,
    pitch: f32,
    distance: f32,
    position: usize,
    pub reverb: bool,
    pub occlusion: bool,
    pub guides: bool,
    play: bool,
    pattern: Option<Pattern>,
    echo: Option<Echo>,
    source: Option<Vec3>,
    beacon: Option<Entity>,
    variation_seed: u32,
}

impl Default for SoundLab {
    fn default() -> Self {
        Self {
            family: Family::Foot,
            category: 0,
            variant: 0,
            gain: 0.85,
            pitch: 1.0,
            distance: 6.0,
            position: 0,
            reverb: true,
            occlusion: true,
            guides: false,
            play: false,
            pattern: None,
            echo: None,
            source: None,
            beacon: None,
            variation_seed: 120078,
        }
    }
}

impl SoundLab {
    pub fn apply(&self, request: &mut Request) {
        request.gain *= self.gain;
        request.pitch *= self.pitch;
        request.reverb = self.reverb;
        request.occlusion = self.occlusion;
    }

    pub fn dispose(&mut self, commands: &mut Commands) {
        if let Some(beacon) = self.beacon.take() {
            commands.entity(beacon).despawn();
        }
    }

    pub fn status(&self, eye: Vec3, gate: bool) -> String {
        let distance = self.source.map_or(self.distance, |p| p.distance(eye));
        format!(
            "{} · variação {}\n{} · {}\nVolume {:.0}% · pitch {:.2}×\nDistância {:.1} m · porta {}\n{}",
            self.family.label(),
            self.variant + 1,
            Category::ALL[self.category].label(),
            ["Frente", "Esquerda", "Direita", "Atrás da porta"][self.position],
            100.0 * self.gain,
            self.pitch,
            distance,
            if gate { "fechada" } else { "aberta" },
            self.echo
                .as_ref()
                .map_or("Amostra individual", |e| e.pattern.label())
        )
    }
}

fn corridor(active: &mut Active, config: &mut Config) {
    if !active.observer.enabled {
        observer::command(active, config, observer::Command::Enable);
        active.observer.automatic = false;
        observer::command(active, config, observer::Command::Select(State::Absent));
    }
    config.tab = Tab::Sound;
}

pub(super) fn command(active: &mut Active, config: &mut Config, action: Command) {
    match action {
        Command::Door => {
            corridor(active, config);
            active.sound.position = 3;
            observer::command(active, config, observer::Command::Door);
        }
        Command::Chase => {
            observer::hunt_command(active, config, observer::HuntCommand::Start);
            config.tab = Tab::Sound;
        }
        Command::Walk => {
            corridor(active, config);
            if active.observer.closed {
                observer::command(active, config, observer::Command::Door);
            }
            observer::command(
                active,
                config,
                observer::Command::Select(State::Investigate),
            );
        }
        _ => {
            let sound = &mut active.sound;
            match action {
                Command::Play => sound.play = true,
                Command::Family(f) => {
                    sound.family = f;
                    sound.play = true;
                }
                Command::Variation => sound.variant = (sound.variant + 1) % 3,
                Command::Category => sound.category = (sound.category + 1) % 3,
                Command::Position => sound.position = (sound.position + 1) % 4,
                Command::Reverb => sound.reverb = !sound.reverb,
                Command::Occlusion => sound.occlusion = !sound.occlusion,
                Command::Guides => sound.guides = !sound.guides,
                Command::Gain(d) => sound.gain = (sound.gain + f32::from(d) * 0.1).clamp(0.0, 1.0),
                Command::Pitch(d) => {
                    sound.pitch = (sound.pitch + f32::from(d) * 0.08).clamp(0.65, 1.45)
                }
                Command::Distance(d) => {
                    sound.distance = (sound.distance + f32::from(d) * 1.0).clamp(2.0, 22.0)
                }
                Command::Pattern(p) => sound.pattern = Some(p),
                Command::Stop => {
                    sound.echo = None;
                    sound.pattern = None;
                }
                _ => unreachable!(),
            }
        }
    }
}

pub(super) fn update(
    mut commands: Commands,
    mut lab: ResMut<Lab>,
    config: Res<Config>,
    time: Res<Time>,
    mut requests: MessageWriter<Request>,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    mut markers: Query<&mut Transform, With<Beacon>>,
) {
    let Some(active) = &mut lab.active else {
        return;
    };
    let camera = active.camera_transform(&config);
    let heading = camera.forward().with_y(0.0).normalize_or_zero();
    let side = camera.right().with_y(0.0).normalize_or_zero();
    let sound = &mut active.sound;
    let direction = match sound.position {
        1 => (heading * 0.55 - side).normalize(),
        2 => (heading * 0.55 + side).normalize(),
        _ => heading,
    };
    let point = if sound.position == 3 {
        Vec3::new(4.60, 0.64, -9.0 - sound.distance * 0.50)
    } else {
        (camera.translation + direction * sound.distance).with_y(0.64)
    };
    let category = Category::ALL[sound.category];
    if let Some(pattern) = sound.pattern.take() {
        let at = if sound.position == 3 {
            point + Vec3::NEG_Z * 2.1
        } else {
            (camera.translation + direction * (sound.distance + 4.0)).with_y(0.64)
        };
        sound.variation_seed = sound.variation_seed.wrapping_add(289);
        sound.echo = Some(Echo::new(
            pattern,
            if category == Category::Physical {
                Category::Illusion
            } else {
                category
            },
            [at, point],
            sound.variation_seed,
        ));
    }
    if std::mem::take(&mut sound.play) {
        sound.source = Some(point);
        let mut request = Request::new(sound.family, category, point, true);
        request.variant = Some(sound.variant);
        sound.apply(&mut request);
        requests.write(request);
    }
    if let Some(echo) = &mut sound.echo {
        let (ended, pulse) = echo.tick(time.delta_secs(), camera, 0.0, true);
        if ended {
            sound.echo = None;
        }
        if let Some(mut request) = pulse {
            sound.source = Some(request.at);
            sound.apply(&mut request);
            requests.write(request);
        }
    }
    if !sound.guides {
        sound.dispose(&mut commands);
        return;
    }
    let origin = sound.source.unwrap_or(point);
    if let Some(id) = sound.beacon {
        if let Ok(mut marker) = markers.get_mut(id) {
            marker.translation = origin;
        }
    } else {
        let mut lines = vec![
            [0., 0., 0.],
            [0., 1.3, 0.],
            [-0.28, 0., 0.],
            [0.28, 0., 0.],
            [0., 0., -0.28],
            [0., 0., 0.28],
        ];
        for radius in [2.5, 5.8] {
            for i in 0..32 {
                lines.extend([i, i + 1].map(|j| {
                    let a = j as f32 * std::f32::consts::TAU / 32.;
                    [radius * a.cos(), -0.60, radius * a.sin()]
                }));
            }
        }
        sound.beacon = Some(
            commands
                .spawn((
                    Beacon,
                    ChildOf(active.root),
                    Mesh3d(meshes.add(room::line_mesh(lines))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.32, 0.95, 0.74),
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(origin),
                    RenderLayers::layer(room::LAYER),
                    NoFrustumCulling,
                ))
                .id(),
        );
    }
}

#[derive(Component)]
pub(super) struct Beacon;
