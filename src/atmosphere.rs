//! Local presentation. Reads the expedition state without changing its rules.
pub mod director;
mod episode;
pub mod sound;

use crate::app::{ExitPresentation, LocalPlayer, Screen, Session};
use bevy::prelude::*;
pub(crate) use episode::PUMP_KNOCK;

pub const WORKSHOP_HUM: Vec3 = Vec3::new(-3.3, 2.48, 4.0);
pub const WORKSHOP_ECHO: Vec3 = Vec3::new(-3.6, 0.32, 4.40);
pub const DUCT: Vec3 = Vec3::new(1.45, 2.32, -6.7);
const BREATH_BEFORE_SOUND: f32 = 0.85;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Echo {
    #[default]
    Waiting,
    Hush {
        starts_at: f32,
    },
    Heard {
        starts_at: f32,
    },
    Finished,
}

impl Echo {
    fn advance(&mut self, elapsed: f32, has_fuse: bool, power: bool, feet: Vec3) -> bool {
        if power {
            *self = Self::Finished;
            return false;
        }
        match *self {
            Self::Waiting
                if has_fuse && feet.x.abs() <= 1.45 && (2.65..=5.45).contains(&feet.z) =>
            {
                *self = Self::Hush { starts_at: elapsed };
                false
            }
            Self::Hush { starts_at } if elapsed - starts_at >= BREATH_BEFORE_SOUND => {
                *self = Self::Heard { starts_at };
                true
            }
            Self::Heard { starts_at } if elapsed - starts_at > 4.0 => {
                *self = Self::Finished;
                false
            }
            _ => false,
        }
    }

    fn silence(self, elapsed: f32) -> f32 {
        let starts_at = match self {
            Self::Hush { starts_at } | Self::Heard { starts_at } => starts_at,
            _ => return 0.0,
        };
        let age = elapsed - starts_at;
        let start = smooth(age / 0.63);
        let end = smooth((4.0 - age) / 1.75);
        start * end
    }
}

fn smooth(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

#[derive(Resource, Default)]
pub struct Atmosphere {
    attempt: u64,
    revision: u32,
    pub echo: Echo,
    pub(crate) play_echo: bool,
    pub(crate) quiet: f32,
    pub(crate) relief: f32,
    pub(crate) receipt: f32,
    pub(crate) desk: f32,
    pub(crate) refuge: f32,
    pub(crate) episode: episode::Episode,
}

#[derive(Clone, Copy)]
pub enum Circuit {
    Workshop,
    Inspection,
    Exit,
    Desk,
}

#[derive(Component)]
pub struct CircuitLight {
    pub circuit: Circuit,
    pub lumens: f32,
    pub glow: LinearRgba,
    pub lens: Handle<StandardMaterial>,
}

pub fn update(
    time: Res<Time>,
    session: Res<Session>,
    player: Res<LocalPlayer>,
    mut atmosphere: ResMut<Atmosphere>,
    mut lights: Query<(&CircuitLight, &mut SpotLight)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if atmosphere.attempt != session.attempt {
        *atmosphere = Atmosphere {
            attempt: session.attempt,
            ..default()
        };
    }
    if atmosphere.revision != session.perception_revision {
        atmosphere.revision = session.perception_revision;
        if !session.progress().exit_unlocked() {
            atmosphere.episode.reset_rail();
        }
        atmosphere.receipt = 0.0;
    }
    atmosphere.play_echo = false;
    if session.screen == Screen::Playing {
        atmosphere.play_echo = atmosphere.echo.advance(
            session.elapsed,
            session.progress().has_fuse(),
            session.progress().power_restored(),
            player.body.feet,
        );
        atmosphere.episode.update(
            session.elapsed,
            player.body.feet,
            session.progress().phase(),
        );
        let blend = 1.0 - (-3.2 * time.delta_secs()).exp();
        let lit = session.memory().desk_light;
        atmosphere.desk += (f32::from(lit) - atmosphere.desk) * blend;
        let near_desk = crate::scene::station::lamp_refuge(lit, player.body.feet);
        atmosphere.refuge += (f32::from(near_desk) - atmosphere.refuge) * blend;
        let quiet = atmosphere.echo.silence(session.elapsed);
        atmosphere.quiet += (quiet - atmosphere.quiet) * blend;
        let open = if session.progress().exit_unlocked() {
            1.0
        } else {
            0.0
        };
        atmosphere.relief += (open - atmosphere.relief) * blend;
        let receipt = if matches!(session.false_exit, Some(ExitPresentation::Receipt { .. })) {
            1.0
        } else {
            0.0
        };
        atmosphere.receipt += (receipt - atmosphere.receipt) * blend;
    }
    for (fixture, mut light) in &mut lights {
        let multiplier = match fixture.circuit {
            Circuit::Workshop => 1.0 - 0.64 * atmosphere.quiet,
            Circuit::Inspection => 1.0 - 0.36 * atmosphere.episode.pump_hush,
            Circuit::Exit => (0.06 + 0.94 * atmosphere.relief) * (1.0 + 0.12 * atmosphere.receipt),
            Circuit::Desk => 0.04 + 0.96 * atmosphere.desk,
        };
        let intensity = fixture.lumens * multiplier;
        if (light.intensity - intensity).abs() > 5.0 {
            light.intensity = intensity;
            if let Some(material) = materials.get_mut(&fixture.lens) {
                material.emissive = fixture.glow * multiplier;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_waits_for_leaving_the_workshop_and_plays_once() {
        let mut echo = Echo::default();
        let workshop = Vec3::new(-3.5, 0.0, 4.0);
        let corridor = Vec3::new(0.0, 0.0, 4.0);
        assert!(!echo.advance(2.0, true, false, workshop));
        assert!(!echo.advance(3.0, true, false, corridor));
        assert!(matches!(echo, Echo::Hush { .. }));
        assert!(!echo.advance(3.6, true, false, corridor));
        assert!(echo.silence(3.6) > 0.8);
        assert!(echo.advance(3.9, true, false, corridor));
        assert!(!echo.advance(4.8, true, false, workshop));
        assert!(!echo.advance(8.0, true, false, corridor));
        assert_eq!(echo, Echo::Finished);
        assert_eq!(echo.silence(8.0), 0.0);
    }

    #[test]
    fn restoring_power_cancels_a_pending_cue() {
        let corridor = Vec3::new(0.0, 0.0, 4.0);
        let mut echo = Echo::default();
        assert!(!echo.advance(1.0, true, false, corridor));
        assert!(!echo.advance(2.0, true, true, corridor));
        assert_eq!(echo, Echo::Finished);
    }
}
