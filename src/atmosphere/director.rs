//! Schedules opportunities for presentation systems. It never changes mission state.
use crate::{
    app::{ExitPresentation, LocalPlayer, Screen, Session},
    atmosphere::Atmosphere,
    reality::{Developer, Threat},
    scene::{illusion::Illusions, presence::Presence, station},
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
    Unease,
    Anticipation,
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
    distortion: bool,
    anomalies: u8,
    echo_ready: bool,
    relay_ready: bool,
    strain: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Beat {
    at: f32,
    cue: Cue,
    sector: Option<u8>,
}

#[derive(Resource, Default)]
pub struct Director {
    pub attempt: u64,
    seed: u32,
    initial_seed: u32,
    revision: u32,
    pub mood: Mood,
    pub tally: Tally,
    pub hush: f32,
    pub revisit: bool,
    last_sector: Option<u8>,
    visited: u16,
    recent_sectors: [Option<u8>; 4],
    next_sample: f32,
    next_window: f32,
    last_event: Option<(f32, Cue)>,
    window: Option<Window>,
    last_forward: Vec3,
    phase: Phase,
    pub order: usize,
    pressure: f32,
    last_presence: Option<f32>,
    history: [Option<Beat>; 6],
    requested: Option<Cue>,
    quiet_until: f32,
}

impl Director {
    fn new(attempt: u64, seed: u32) -> Self {
        Self {
            attempt,
            seed: seed ^ (attempt as u32).wrapping_mul(0x9e3779b9),
            initial_seed: seed,
            next_window: 5.0,
            tally: Tally {
                moods: 1,
                ..default()
            },
            ..default()
        }
    }

    pub fn initial_seed(&self) -> u32 {
        self.initial_seed
    }

    pub fn intensity(&self) -> f32 {
        self.pressure
    }

    pub fn recent(&self) -> &[Option<Beat>] {
        &self.history
    }

    pub fn request(&mut self, cue: Cue) {
        self.requested = Some(cue);
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
            && (cue != Cue::Presence || self.last_presence.is_none_or(|last| now - last >= 6.0))
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
        self.history.rotate_right(1);
        self.history[0] = Some(Beat {
            at: now,
            cue,
            sector: self.last_sector,
        });
        if cue == Cue::Presence {
            self.last_presence = Some(now);
        }
        self.pressure = (self.pressure
            + match cue {
                Cue::Presence => 0.62,
                Cue::Sound => 0.28,
                Cue::Illusion => 0.40,
            })
        .min(1.0);
        self.window = None;
        self.next_window =
            now + match cue {
                Cue::Presence => 7.0,
                Cue::Sound => 6.0,
                Cue::Illusion => 6.5,
            } + self.random() * 3.0
                + (self.pressure - 0.72).max(0.0) * 5.0;
        self.set_mood(Mood::Tension);
        debug!("Diretor: {cue:?} em {now:.2}s");
    }

    fn step(&mut self, context: Context, dt: f32) {
        let now = context.elapsed;
        let target = match self.mood {
            Mood::Calm | Mood::Relief => 0.0,
            Mood::Unease => 0.08,
            Mood::Suspicion => 0.25,
            Mood::Anticipation => 0.30,
            Mood::Tension => 0.38,
        };
        self.hush += (target - self.hush) * (1.0 - (-1.9 * dt).exp());
        self.pressure *= (-dt / 5.7).exp();

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
            self.recent_sectors.rotate_right(1);
            self.recent_sectors[0] = Some(id);
            self.last_sector = context.sector;
        }
        if self.phase != context.phase {
            self.phase = context.phase;
            let after_recovery = self.last_event.map_or(0.0, |(time, _)| time + 5.4);
            self.next_window = self.next_window.min((now + 2.4).max(after_recovery));
        }
        if let Some(window) = &self.window {
            if now < window.until {
                if now - window.began >= 1.05 {
                    self.set_mood(Mood::Anticipation);
                }
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
            } else if now + 1.6 >= self.next_window {
                Mood::Unease
            } else {
                Mood::Calm
            });
        } else if now + 1.6 >= self.next_window {
            self.set_mood(Mood::Unease);
        }

        if now < self.quiet_until || now < self.next_window && self.requested.is_none() {
            return;
        }
        if context.protected || turning || context.sector.is_none() {
            self.next_window = now + 0.85;
            self.tally.deferred += 1;
            return;
        }
        let roll = self.random();
        let recently_here = self.recent_sectors[1..].contains(&context.sector);
        let first = if let Some(requested) = self.requested.take() {
            Some(requested)
        } else if roll < 0.12 + context.strain * 0.30 && now > 14.0 {
            None
        } else if context.echo_ready {
            Some(Cue::Sound)
        } else if context.relay_ready
            || context.strain > 0.1 && roll < 0.64
            || context.distortion && roll > 0.44 + f32::from(context.anomalies.min(4)) * 0.035
        {
            Some(Cue::Illusion)
        } else if (context.phase == Phase::Exit || context.speed > 2.65) && roll < 0.83 {
            Some(Cue::Presence)
        } else if context.speed < 2.55 && roll > if self.revisit { 0.54 } else { 0.68 } {
            Some(Cue::Illusion)
        } else if roll < 0.48 {
            Some(Cue::Sound)
        } else {
            Some(Cue::Presence)
        };
        let first = first.map(|cue| {
            let repeated_here = recently_here
                && self
                    .history
                    .iter()
                    .flatten()
                    .filter(|beat| {
                        beat.sector == context.sector && beat.cue == cue && now - beat.at < 36.0
                    })
                    .count()
                    >= 2;
            if repeated_here {
                match cue {
                    Cue::Presence => Cue::Illusion,
                    Cue::Sound | Cue::Illusion => Cue::Presence,
                }
            } else {
                cue
            }
        });
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

#[allow(
    clippy::too_many_arguments,
    reason = "Independent optional physical representation."
)]
pub fn update(
    time: Res<Time>,
    mut session: ResMut<Session>,
    player: Res<LocalPlayer>,
    atmosphere: Res<Atmosphere>,
    (presence, threat, illusions): (Res<Presence>, Res<Threat>, Res<Illusions>),
    (smoke, developer): (Res<SmokeTest>, Res<Developer>),
    mut director: ResMut<Director>,
    #[cfg(feature = "dev-lab")] physical: Option<Res<crate::lab::observer::Campaign>>,
) {
    if session.attempt != director.attempt {
        let seed = if let Some(seed) = developer.seed {
            seed
        } else if smoke.enabled {
            20261118
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(20261118, |time| time.subsec_nanos())
        };
        *director = if developer.seed.is_some() {
            Director {
                attempt: session.attempt,
                ..Director::new(1, seed)
            }
        } else {
            Director::new(session.attempt, seed)
        };
        if developer.enabled {
            info!("DEV seed {seed} · tentativa {}", session.attempt);
        }
    }
    if director.revision != session.perception_revision {
        director.revision = session.perception_revision;
        director.window = None;
        director.requested = None;
        director.quiet_until = session.elapsed + 7.0;
        director.set_mood(Mood::Relief);
    }
    if session.screen != Screen::Playing || session.collapse_active() {
        return;
    }
    let sector = station::sector_id(player.body.feet);
    if let Some(id) = sector
        && session.memory().recent_route()[0] != sector
    {
        session.memory_mut().enter(id);
    }
    let distorted = session
        .expedition
        .observer()
        .distortion_ready(session.elapsed);
    let memory = session.memory();
    #[cfg(feature = "dev-lab")]
    let physical_active = physical.is_some_and(|mode| mode.active());
    #[cfg(not(feature = "dev-lab"))]
    let physical_active = false;
    let echo_ready = memory.echo_pattern(session.elapsed, distorted).is_some()
        && session.progress().phase() >= Phase::Electrical
        && crate::atmosphere::sound::service_source(player.body.feet, player.yaw).is_some();
    let relay_ready = memory.relay_due(session.elapsed, distorted)
        && player
            .body
            .feet
            .xz()
            .distance_squared(station::SERVICE_RELAY.xz())
            < 5.5 * 5.5;
    director.step(
        Context {
            elapsed: session.elapsed,
            sector,
            phase: session.progress().phase(),
            forward: Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0) * Vec3::NEG_Z,
            speed: player.body.velocity.xz().length(),
            protected: physical_active
                || presence.active()
                || atmosphere.quiet > 0.14
                || atmosphere.episode.holding_silence()
                || threat.active()
                || session.psychology.value() >= 60.0
                    && session.psychology.recovery() != ainda_vivo_core::psychology::Recovery::None
                || station::lamp_refuge(memory.desk_light, player.body.feet)
                || matches!(session.false_exit, Some(ExitPresentation::Receipt { .. })),
            distortion: distorted,
            anomalies: illusions.altered,
            echo_ready,
            relay_ready,
            strain: ((session.psychology.value() - 55.0) / 45.0).clamp(0.0, 1.0),
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
            distortion: false,
            anomalies: 0,
            echo_ready: false,
            relay_ready: false,
            strain: 0.0,
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

    #[test]
    fn same_seed_and_route_produce_the_same_windows_with_new_moods() {
        fn run() -> (Vec<(u32, Cue)>, u8) {
            let mut director = Director::new(1, 932);
            let mut cues = Vec::new();
            for frame in 1..900 {
                let now = frame as f32 / 10.0;
                director.step(
                    Context {
                        distortion: frame > 400,
                        ..at(now, 1 + (frame / 174 % 3) as u8)
                    },
                    0.1,
                );
                if frame % 16 == 0
                    && let Some(cue) = [Cue::Presence, Cue::Sound, Cue::Illusion]
                        .into_iter()
                        .find(|cue| director.allows(*cue, now))
                {
                    director.claim(cue, now);
                    cues.push((frame, cue));
                }
            }
            (cues, director.tally.moods)
        }
        let (events, moods) = run();
        assert!(!events.is_empty());
        assert_eq!((events, moods), run());
        assert_eq!(moods, 0b11_1111);
    }
}
