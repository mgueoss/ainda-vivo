//! Schedules opportunities for presentation systems. It never changes mission state.
use crate::{
    app::{LocalPlayer, Screen, Session},
    atmosphere::Atmosphere,
    scene::{presence::Presence, station},
    smoke::SmokeTest,
};
use ainda_vivo_core::story::Phase;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mood {
    #[default]
    Calm,
    Suspicion,
    Tension,
    Relief,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Presence,
    Sound,
    Illusion,
}

#[derive(Default)]
pub struct Tally {
    pub cues: [u32; 3],
    pub empty: u32,
    pub deferred: u32,
    pub revisits: u32,
    pub moods: u8,
}

struct Window {
    began: f32,
    until: f32,
    first: Option<Cue>,
}

struct Context {
    elapsed: f32,
    sector: Option<u8>,
    phase: Phase,
    forward: Vec3,
    speed: f32,
    protected: bool,
}

#[derive(Resource, Default)]
pub struct Director {
    pub attempt: u64,
    seed: u32,
    pub mood: Mood,
    pub tally: Tally,
    pub hush: f32,
    pub revisit: bool,
    last_sector: Option<u8>,
    visited: u16,
    next_sample: f32,
    next_window: f32,
    last_event: Option<(f32, Cue)>,
    window: Option<Window>,
    last_forward: Vec3,
    phase: Phase,
    pub order: usize,
}

impl Director {
    fn new(attempt: u64, seed: u32) -> Self {
        Self {
            attempt,
            seed: seed ^ (attempt as u32).wrapping_mul(0x9e3779b9),
            next_window: 5.0,
            tally: Tally {
                moods: 1,
                ..default()
            },
            ..default()
        }
    }

    fn random(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.seed >> 8) as f32 / 16_777_216.0
    }

    fn set_mood(&mut self, mood: Mood) {
        self.mood = mood;
        self.tally.moods |= 1 << mood as u8;
    }

    pub fn allows(&self, cue: Cue, now: f32) -> bool {
        let Some(window) = &self.window else {
            return false;
        };
        now < window.until
            && window.first.is_some_and(|first| {
                first == cue || (now > window.began + 1.4 && cue != Cue::Sound)
            })
            && self.last_event.is_none_or(|(time, last)| {
                cue != last
                    || now - time
                        >= match cue {
                            Cue::Presence => 9.0,
                            Cue::Sound => 15.0,
                            Cue::Illusion => 13.0,
                        }
            })
    }

    pub fn claim(&mut self, cue: Cue, now: f32) {
        debug_assert!(self.allows(cue, now));
        self.tally.cues[cue as usize] += 1;
        self.last_event = Some((now, cue));
        self.window = None;
        self.next_window =
            now + match cue {
                Cue::Presence => 7.0,
                Cue::Sound => 6.0,
                Cue::Illusion => 6.5,
            } + self.random() * 3.0;
        self.set_mood(Mood::Tension);
        debug!("Diretor: {cue:?} em {now:.2}s");
    }

    fn step(&mut self, context: Context, dt: f32) {
        let now = context.elapsed;
        let target = match self.mood {
            Mood::Calm | Mood::Relief => 0.0,
            Mood::Suspicion => 0.25,
            Mood::Tension => 0.38,
        };
        self.hush += (target - self.hush) * (1.0 - (-1.9 * dt).exp());

        if context.protected && self.window.take().is_some() {
            self.next_window = self.next_window.max(now + 1.5);
            self.tally.deferred += 1;
        }
        if now < self.next_sample {
            return;
        }
        self.next_sample = now + 0.34;
        let turning =
            self.last_forward != Vec3::ZERO && self.last_forward.dot(context.forward) < 0.78;
        self.last_forward = context.forward;
        if context.sector != self.last_sector
            && let Some(id) = context.sector
        {
            let bit = 1 << id;
            self.revisit = self.visited & bit != 0;
            self.visited |= bit;
            self.tally.revisits += u32::from(self.revisit);
            self.last_sector = context.sector;
        }
        if self.phase != context.phase {
            self.phase = context.phase;
            let after_recovery = self.last_event.map_or(0.0, |(time, _)| time + 5.4);
            self.next_window = self.next_window.min((now + 2.4).max(after_recovery));
        }
        if let Some(window) = &self.window {
            if now < window.until {
                return;
            }
            self.tally.empty += 1;
            self.window = None;
            self.next_window = now + 3.0 + self.random() * 4.0;
            self.set_mood(Mood::Relief);
        } else if let Some((previous, _)) = self.last_event {
            self.set_mood(if now - previous < 1.8 {
                Mood::Tension
            } else if now - previous < 5.4 {
                Mood::Relief
            } else {
                Mood::Calm
            });
        }

        if now < self.next_window {
            return;
        }
        if context.protected || turning || context.sector.is_none() {
            self.next_window = now + 0.85;
            self.tally.deferred += 1;
            return;
        }
        let roll = self.random();
        let first = if roll < 0.12 && now > 14.0 {
            None
        } else if (context.phase == Phase::Exit || context.speed > 2.65) && roll < 0.83 {
            Some(Cue::Presence)
        } else if context.speed < 2.55 && roll > if self.revisit { 0.54 } else { 0.68 } {
            Some(Cue::Illusion)
        } else if roll < 0.48 {
            Some(Cue::Sound)
        } else {
            Some(Cue::Presence)
        };
        self.order = (self.random() * 1024.0) as usize;
        self.window = Some(Window {
            began: now,
            until: now + 3.5 + self.random() * 2.7,
            first,
        });
        debug!(
            "Diretor: janela {first:?} em {now:.2}s ({:?}/{:?})",
            context.sector, context.phase
        );
        self.set_mood(Mood::Suspicion);
    }
}

pub fn update(
    time: Res<Time>,
    session: Res<Session>,
    player: Res<LocalPlayer>,
    atmosphere: Res<Atmosphere>,
    presence: Res<Presence>,
    smoke: Res<SmokeTest>,
    mut director: ResMut<Director>,
) {
    if session.attempt != director.attempt {
        let seed = if smoke.enabled {
            20261118
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(20261118, |time| time.subsec_nanos())
        };
        *director = Director::new(session.attempt, seed);
    }
    if session.screen != Screen::Playing {
        return;
    }
    director.step(
        Context {
            elapsed: session.elapsed,
            sector: station::sector_id(player.body.feet),
            phase: session.progress.phase(),
            forward: Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0) * Vec3::NEG_Z,
            speed: player.body.velocity.xz().length(),
            protected: presence.active()
                || atmosphere.quiet > 0.14
                || atmosphere.episode.holding_silence(),
        },
        time.delta_secs(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(elapsed: f32, sector: u8) -> Context {
        Context {
            elapsed,
            sector: Some(sector),
            phase: Phase::Baseline,
            forward: Vec3::NEG_Z,
            speed: 1.3,
            protected: false,
        }
    }

    #[test]
    fn recent_cue_has_exclusive_window_and_relief() {
        let mut director = Director::new(1, 49);
        director.window = Some(Window {
            began: 10.0,
            until: 15.0,
            first: Some(Cue::Presence),
        });
        assert!(!director.allows(Cue::Sound, 10.5));
        assert!(director.allows(Cue::Presence, 10.5));
        director.claim(Cue::Presence, 10.5);
        assert!(!director.allows(Cue::Illusion, 13.0));
        director.step(at(13.0, 1), 0.34);
        assert_eq!(director.mood, Mood::Relief);
    }

    #[test]
    fn revisits_are_detected_and_protected_beat_cancels_a_window() {
        let mut director = Director::new(1, 12);
        for (time, sector) in [(1.0, 1), (2.0, 2), (3.0, 1)] {
            director.step(at(time, sector), 0.34);
        }
        assert!(director.revisit);
        assert_eq!(director.tally.revisits, 1);
        director.window = Some(Window {
            began: 10.0,
            until: 15.0,
            first: None,
        });
        assert!(!director.allows(Cue::Presence, 12.0));
        director.step(
            Context {
                protected: true,
                ..at(12.0, 1)
            },
            0.34,
        );
        assert_eq!(director.tally.deferred, 1);
        assert!(director.window.is_none());
        director.window = Some(Window {
            began: 13.0,
            until: 15.0,
            first: None,
        });
        director.step(at(15.1, 1), 0.34);
        assert_eq!(director.tally.empty, 1);
        assert_eq!(director.mood, Mood::Relief);
    }
}
