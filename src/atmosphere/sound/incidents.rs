use super::{Noise, samples, shoe, tone};
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Footsteps,
    Pipe,
    Drag,
    Drip,
}

impl Kind {
    pub const ALL: [Self; 4] = [Self::Footsteps, Self::Pipe, Self::Drag, Self::Drip];

    pub fn duration(self) -> f32 {
        match self {
            Self::Footsteps => 1.20,
            Self::Pipe => 1.45,
            Self::Drag => 1.80,
            Self::Drip => 1.55,
        }
    }

    pub fn gain(self) -> f32 {
        match self {
            Self::Footsteps => 0.84,
            Self::Pipe => 0.71,
            Self::Drag => 0.78,
            Self::Drip => 0.65,
        }
    }

    pub fn clip(self) -> Arc<[f32]> {
        samples(
            self.duration(),
            1473 + self as u32 * 97,
            |t, noise| match self {
                Self::Footsteps => {
                    shoe(t - 0.055, noise, 74.0) * 0.67
                        + shoe(t - 0.53, noise, 106.0) * 0.51
                        + shoe(t - 0.175, noise, 74.0) * 0.11
                        + shoe(t - 0.71, noise, 106.0) * 0.086
                }
                Self::Pipe => {
                    pipe(t - 0.06, noise, 1.0)
                        + pipe(t - 0.38, noise, 0.92) * 0.43
                        + pipe(t - 0.170, noise, 1.0) * 0.12
                }
                Self::Drag => drag(t - 0.09, noise) + drag((t - 0.89) * 1.51, noise) * 0.48,
                Self::Drip => {
                    drip(t - 0.08, noise, 560.0)
                        + drip(t - 0.74, noise, 439.0) * 0.69
                        + drip(t - 0.85, noise, 439.0) * 0.13
                }
            },
        )
        .into()
    }
}

use Kind::*;

// Station details such as ducts, piping and shelves give the sound a place to come from.
const POINTS: &[(Kind, Vec3)] = &[
    (Footsteps, Vec3::new(0.00, 0.15, 6.9)),
    (Pipe, Vec3::new(1.37, 2.42, -6.6)),
    (Drag, Vec3::new(-6.15, 0.38, 6.42)),
    (Drag, Vec3::new(-5.30, 1.05, -21.44)),
    (Footsteps, Vec3::new(-20.00, 0.14, -4.30)),
    (Pipe, Vec3::new(-27.90, 2.25, 6.85)),
    (Drag, Vec3::new(-16.20, 0.36, -22.65)),
    (Footsteps, Vec3::new(13.10, 0.12, 2.30)),
    (Pipe, Vec3::new(24.26, 2.65, -1.65)),
    (Drag, Vec3::new(22.10, 0.34, -23.20)),
    (Pipe, Vec3::new(-19.50, 2.38, -36.15)),
    (Pipe, Vec3::new(-7.18, 2.94, -33.00)),
    (Footsteps, Vec3::new(-2.85, 0.14, -44.10)),
    (Drag, Vec3::new(4.20, 0.25, -53.50)),
    (Drag, Vec3::new(24.00, 1.35, -39.20)),
    (Drip, Vec3::new(-16.95, 2.72, -57.40)),
    (Pipe, Vec3::new(20.95, 2.44, -53.10)),
    (Footsteps, Vec3::new(30.50, 0.13, -52.85)),
    (Drip, Vec3::new(-26.50, 2.30, -71.50)),
    (Drag, Vec3::new(27.00, 0.44, -81.00)),
    (Drip, Vec3::new(4.10, 0.08, -73.65)),
    (Footsteps, Vec3::new(-7.40, 0.18, -80.5)),
    (Pipe, Vec3::new(-3.70, 1.42, -64.25)),
    (Drip, Vec3::new(1.65, 2.56, -92.85)),
];

#[derive(Default)]
pub struct IncidentSchedule {
    attempt: u64,
    seed: u32,
    next_at: f32,
    previous: Option<Kind>,
}

impl IncidentSchedule {
    pub fn reset(&mut self, attempt: u64, seed: u32) {
        if self.attempt != attempt {
            *self = Self {
                attempt,
                seed: seed ^ attempt as u32,
                ..default()
            };
            self.next_at = 7.5 + self.random() * 4.5;
        }
    }

    pub fn defer_until(&mut self, time: f32) {
        self.next_at = self.next_at.max(time);
    }

    pub(super) fn next(&mut self, elapsed: f32, camera: &Transform) -> Option<(Kind, Vec3)> {
        if elapsed < self.next_at {
            return None;
        }
        let start = (self.random() * POINTS.len() as f32) as usize;
        let forward = camera.forward().with_y(0.0).normalize_or_zero();
        let mut candidates = POINTS
            .iter()
            .cycle()
            .skip(start)
            .take(POINTS.len())
            .copied()
            .filter(|(kind, point)| {
                Some(*kind) != self.previous
                    && (4.0..=15.0).contains(&point.distance(camera.translation))
            });
        let outside_view = candidates.clone().find(|(_, point)| {
            (*point - camera.translation)
                .with_y(0.0)
                .normalize_or_zero()
                .dot(forward)
                < 0.44
        });
        let incident = outside_view.or_else(|| candidates.next());
        if let Some((kind, _)) = incident {
            self.previous = Some(kind);
            self.next_at = elapsed + 14.0 + self.random() * 12.0;
        } else {
            self.next_at = elapsed + 1.4;
        }
        incident
    }

    fn random(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.seed >> 8) as f32 / 16_777_216.0
    }
}

fn pipe(age: f32, noise: Noise, scale: f32) -> f32 {
    if !(0.0..0.70).contains(&age) {
        return 0.0;
    }
    let attack = (age / 0.022).min(1.0);
    let ring = (tone(age, 346.0 * scale) * 0.16
        + tone(age, 572.0 * scale) * 0.12
        + tone(age, 867.0 * scale) * 0.052)
        * (-11.0 * age).exp();
    let transient = noise.band * 0.46 * (-24.0 * age).exp();
    attack * (ring + transient)
}

fn drag(age: f32, noise: Noise) -> f32 {
    if !(0.0..0.74).contains(&age) {
        return 0.0;
    }
    let motion = (age / 0.15).min(1.0) * ((0.74 - age) / 0.39).min(1.0);
    let resonance = tone(age, 146.0 - 45.0 * age) * 0.044 + tone(age, 212.0 + 31.0 * age) * 0.032;
    let drag = (noise.band * 1.12 + noise.low * 0.54 + resonance) * (0.72 + 0.28 * tone(age, 19.0));
    motion * drag
}

fn drip(age: f32, noise: Noise, hz: f32) -> f32 {
    if !(0.0..0.57).contains(&age) {
        return 0.0;
    }
    let attack = (age / 0.017).min(1.0);
    let splash = noise.band * 0.54 * (-31.0 * age).exp();
    let ring =
        (tone(age, hz - 94.0 * age) * 0.19 + tone(age, hz * 0.68) * 0.083) * (-13.0 * age).exp();
    attack * (splash + ring)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cues_leave_space_between_sounds_and_do_not_repeat_immediately() {
        let mut schedule = IncidentSchedule::default();
        let camera = Transform::from_xyz(0.0, 1.62, -4.0);
        schedule.reset(1, 931);
        assert!(schedule.next(6.0, &camera).is_none());
        let (first, position) = schedule.next(12.0, &camera).unwrap();
        assert!(position.z > camera.translation.z);
        assert!(schedule.next(25.0, &camera).is_none());
        let (second, _) = schedule.next(38.0, &camera).unwrap();
        assert_ne!(first, second);
        assert!(
            schedule
                .next(60.0, &Transform::from_xyz(0.0, 1.62, 80.0))
                .is_none()
        );
        schedule.reset(2, 981);
        assert!(schedule.next(4.0, &camera).is_none());
    }
}
