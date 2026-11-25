use super::nav::Nav;
use bevy::prelude::*;

pub(super) const EPISODES: usize = 12;
const ROUTES: usize = 12;

#[derive(Clone, Copy, Debug)]
pub struct Memory {
    pub at: Vec2,
    pub when: f32,
    pub expires: f32,
    pub certainty: f32,
    pub error: f32,
    pub velocity: Vec2,
    pub sector: Option<usize>,
}

impl Memory {
    pub(super) fn age(self, now: f32) -> f32 {
        (now - self.when).max(0.0)
    }

    pub(super) fn trust(self, now: f32) -> f32 {
        let remaining =
            ((self.expires - now) / (self.expires - self.when).max(0.01)).clamp(0.0, 1.0);
        self.certainty * remaining * remaining
    }

    pub(super) fn radius(self, now: f32) -> f32 {
        (self.error + self.age(now) * (0.9 + self.velocity.length() * 0.29)).min(13.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Knowledge {
    Confirmed,
    Heard,
    LastSeen,
    Region,
    Unknown,
}

impl Knowledge {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmado pela visão",
            Self::Heard => "ruído localizado",
            Self::LastSeen => "última posição conhecida",
            Self::Region => "região provável",
            Self::Unknown => "localização desconhecida",
        }
    }
}

pub(super) fn best_clue(
    now: f32,
    visible: bool,
    seen: Option<Memory>,
    heard: Option<Memory>,
) -> (Knowledge, Option<Memory>) {
    if visible && seen.is_some_and(|clue| clue.age(now) < 0.26) {
        return (Knowledge::Confirmed, seen);
    }
    let clue = match (seen, heard) {
        (Some(visual), Some(noise)) if noise.trust(now) > visual.trust(now) * 1.18 => {
            Some((Knowledge::Heard, noise))
        }
        (Some(visual), _) => Some((Knowledge::LastSeen, visual)),
        (None, Some(noise)) => Some((Knowledge::Heard, noise)),
        _ => None,
    };
    match clue {
        Some((kind, m)) if m.trust(now) > 0.31 => (kind, Some(m)),
        Some((_, m)) if m.trust(now) > 0.09 => (Knowledge::Region, Some(m)),
        _ => (Knowledge::Unknown, None),
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Episode {
    pub when: f32,
    pub reason: &'static str,
    pub sector: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Habit {
    pub link: [usize; 2],
    pub crossings: u8,
    pub failures: u8,
    pub when: f32,
}

#[derive(Default)]
pub(super) struct MemoryBook {
    pub episodes: [Option<Episode>; EPISODES],
    pub routes: [Option<Habit>; ROUTES],
    pub sightings: u32,
    pub losses: u32,
    pub noises: u32,
    pub(crate) replans: u32,
    pub(crate) blocked: u32,
    pub(crate) recovered: u32,
    pub(crate) reacquired: u32,
    pub(crate) seconds_at_pace: f32,
    pub(crate) seconds_in_sight: f32,
    last_loss: f32,
}

impl MemoryBook {
    pub(super) fn record(&mut self, when: f32, reason: &'static str, sector: Option<usize>) {
        self.episodes.rotate_right(1);
        self.episodes[0] = Some(Episode {
            when,
            reason,
            sector,
        });
    }

    fn track(&mut self, link: [usize; 2], now: f32) -> &mut Habit {
        let index = self
            .routes
            .iter()
            .position(|entry| entry.is_some_and(|h| h.link == link))
            .or_else(|| self.routes.iter().position(Option::is_none))
            .unwrap_or_else(|| {
                self.routes
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.unwrap().when.total_cmp(&b.unwrap().when))
                    .unwrap()
                    .0
            });
        let habit = self.routes[index].get_or_insert_default();
        if habit.link != link || now - habit.when > 88.0 {
            *habit = Habit { link, ..default() };
        }
        habit.when = now;
        habit
    }

    pub(super) fn observe(
        &mut self,
        previous: Option<Memory>,
        current: Memory,
        nav: &Nav,
        closed: bool,
    ) {
        if current.velocity.length_squared() > 7.8 {
            self.seconds_at_pace += previous.map_or(0.0, |p| (current.when - p.when).min(0.24));
        }
        if let Some(p) = previous {
            self.seconds_in_sight += (current.when - p.when).min(0.24);
            if let (Some(from), Some(to)) = (p.sector, current.sector)
                && from != to
                && current.when - p.when <= 0.42
                && p.certainty.min(current.certainty) >= 0.58
                && p.at.distance_squared(current.at) < 12.0
                && nav.exits(from, closed).any(|(neighbor, _)| neighbor == to)
            {
                let h = self.track([from, to], current.when);
                h.crossings = (h.crossings + 1).min(5);
                self.record(current.when, "passagem do operador observada", Some(to));
            }
        }
    }

    pub(super) fn missed(&mut self, now: f32, sector: Option<usize>) {
        if self.losses == 0 || now - self.last_loss >= 0.95 {
            self.losses += 1;
            self.last_loss = now;
            self.record(now, "perdeu contato visual", sector);
        }
    }

    pub(super) fn rate(&self, from: usize, to: usize, now: f32) -> f32 {
        self.routes
            .iter()
            .flatten()
            .find(|h| h.link == [from, to])
            .map_or(0.0, |h| {
                let decay = (1.0 - (now - h.when) / 88.0).clamp(0.0, 1.0);
                let habit = f32::from(h.crossings.saturating_sub(1).min(3)) * 0.12;
                let failures = f32::from(h.failures.min(3)) * 0.23;
                (habit - failures) * decay
            })
    }

    pub(super) fn failed(&mut self, link: [usize; 2], now: f32) {
        let habit = self.track(link, now);
        habit.failures = (habit.failures + 1).min(4);
        self.record(now, "trajeto descartado após bloqueio", Some(link[1]));
    }

    pub(super) fn running_tendency(&self) -> Option<f32> {
        (self.seconds_in_sight >= 2.0)
            .then(|| (self.seconds_at_pace / self.seconds_in_sight).clamp(0.0, 1.0))
    }
}
