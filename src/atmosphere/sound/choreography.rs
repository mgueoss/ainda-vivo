use super::presence_audio::{Category, Echo, Family, Pattern, Request};
use crate::scene::station;
use bevy::prelude::*;

const PATTERNS: [Pattern; 5] = [
    Pattern::Knock,
    Pattern::Breath,
    Pattern::Trail,
    Pattern::Rush,
    Pattern::Far,
];

#[derive(Default)]
pub(super) struct Preferences {
    last: Option<Pattern>,
    ignored: [u8; 5],
}

pub(super) struct Opportunity {
    pub speed: f32,
    pub pressure: f32,
    pub strain: f32,
    pub familiar: bool,
    pub covered: bool,
    pub separated_actor: bool,
    pub distorted: bool,
    pub armed: bool,
}

impl Preferences {
    pub(super) fn choose(&self, context: Opportunity, roll: f32) -> Pattern {
        let moving = (context.speed / 3.5).clamp(0.0, 1.0);
        let waiting = 1.0 - moving;
        let covered = u8::from(context.covered) as f32;
        let known = u8::from(context.familiar) as f32;
        let progress = u8::from(context.armed) as f32;
        let false_world = u8::from(context.distorted) as f32;
        let scores = if context.separated_actor {
            [
                0.19,
                0.50 + waiting * 0.27 + covered * 0.09,
                0.41 + moving * 0.33,
                0.42 + moving * 0.40 - context.strain * 0.52,
                0.25,
            ]
        } else {
            [
                0.39 + context.strain * 0.32 + context.pressure * 0.21,
                0.39 + waiting * 0.24 + covered * 0.13,
                0.29 + moving * 0.35 + known * 0.18,
                0.43 + moving * 0.34 + progress * 0.10 + false_world * 0.07
                    - context.pressure * 0.57
                    - context.strain * 0.38,
                0.48 + (1.0 - known) * 0.08,
            ]
        };
        PATTERNS
            .into_iter()
            .enumerate()
            .max_by(|&(a, p), &(b, q)| {
                let value = |i, pattern| {
                    let novelty = if self.last == Some(pattern) {
                        0.14
                    } else {
                        0.0
                    };
                    let jitter = ((roll + i as f32 * 0.618_034).fract() - 0.5) * 0.16;
                    scores[i] - f32::from(self.ignored[i]) * 0.16 - novelty + jitter
                };
                value(a, p).total_cmp(&value(b, q))
            })
            .unwrap()
            .1
    }

    pub(super) fn observe(&mut self, pattern: Pattern, heard: bool, reaction: Reaction) {
        self.last = Some(pattern);
        if heard {
            let score = &mut self.ignored[pattern as usize];
            *score = if reaction == Reaction::None {
                (*score + 1).min(3)
            } else {
                score.saturating_sub(1)
            };
        }
        for (idx, other) in self.ignored.iter_mut().enumerate() {
            if idx != pattern as usize && *other > 0 {
                *other -= 1;
                break;
            }
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) enum Reaction {
    #[default]
    None,
    Look,
    Approach,
    ChangeSector,
    Accelerate,
}

impl Reaction {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::None => "sem mudança observável",
            Self::Look => "olhou para a fonte",
            Self::Approach => "aproximou-se da fonte",
            Self::ChangeSector => "mudou de setor",
            Self::Accelerate => "acelerou o passo",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Stage {
    #[default]
    Prepare,
    Anticipate,
    Develop,
}

pub(super) struct Chain {
    echo: Echo,
    pub stage: Stage,
    pub reaction: Reaction,
    pub audible: bool,
    pub source_sector: u8,
    pub actor_sector: Option<u8>,
    pub purpose: &'static str,
    pub origin: Vec3,
    clock: f32,
    baseline: f32,
    initial_heading: f32,
    initial_sector: Option<u8>,
    initial_speed: f32,
}

pub(super) struct Start {
    pub pattern: Pattern,
    pub category: Category,
    pub path: [Vec3; 2],
    pub source_sector: u8,
    pub actor_sector: Option<u8>,
    pub seed: u32,
}

impl Chain {
    pub(super) fn new(start: Start) -> Self {
        Self {
            echo: Echo::new(start.pattern, start.category, start.path, start.seed),
            stage: Stage::Prepare,
            reaction: Reaction::None,
            audible: false,
            source_sector: start.source_sector,
            actor_sector: start.actor_sector,
            purpose: if start.actor_sector.is_some() {
                "Desviar atenção"
            } else {
                match start.pattern {
                    Pattern::Knock => "Criar antecipação",
                    Pattern::Breath => "Provocar investigação",
                    Pattern::Trail => "Repetir um passo familiar",
                    Pattern::Rush => "Simular ameaça",
                    Pattern::Far => "Induzir dúvida",
                }
            },
            origin: start.path[0],
            clock: 0.0,
            baseline: 0.0,
            initial_heading: 0.0,
            initial_sector: None,
            initial_speed: 0.0,
        }
    }

    pub(super) fn pattern(&self) -> Pattern {
        self.echo.pattern
    }

    pub(super) fn tick(
        &mut self,
        dt: f32,
        view: Transform,
        speed: f32,
        audible: bool,
    ) -> (bool, Option<Request>) {
        self.clock += dt;
        let delta = self.origin - view.translation;
        let distance = delta.length();
        let toward = view.forward().dot(delta.normalize_or_zero());
        let sector = station::sector_id(view.translation);
        if distance < 3.0 || self.clock > 1.15 && sector == Some(self.source_sector) {
            return (true, None);
        }
        if self.stage == Stage::Prepare {
            if self.clock < 0.26 {
                return (false, None);
            }
            self.baseline = distance;
            self.initial_heading = toward;
            self.initial_sector = sector;
            self.initial_speed = speed;
            self.audible = audible;
            self.stage = Stage::Anticipate;
            let family = match self.pattern() {
                Pattern::Rush | Pattern::Trail => Family::Walk,
                Pattern::Knock => Family::Stop,
                Pattern::Breath => Family::Throat,
                Pattern::Far => Family::Moan,
            };
            let mut sound = Request::new(family, self.echo.category, self.origin, false);
            sound.gain = 0.73;
            return (false, Some(sound));
        }
        if self.audible && self.reaction == Reaction::None {
            self.reaction = if self.baseline - distance > 1.05 {
                Reaction::Approach
            } else if sector != self.initial_sector {
                Reaction::ChangeSector
            } else if self.initial_heading < 0.42 && toward > 0.72 {
                Reaction::Look
            } else if self.initial_speed < 2.10 && speed > 3.32 {
                Reaction::Accelerate
            } else {
                Reaction::None
            };
        }
        if self.stage == Stage::Anticipate {
            if self.clock < 1.06 {
                return (false, None);
            }
            if matches!(self.pattern(), Pattern::Knock | Pattern::Far)
                || self.reaction == Reaction::Approach && self.pattern() == Pattern::Breath
            {
                return (true, None);
            }
            self.stage = Stage::Develop;
        }
        if matches!(self.reaction, Reaction::Approach | Reaction::ChangeSector)
            && self.clock > 2.35
            && self.initial_sector != sector
        {
            return (true, None);
        }
        let (end, mut pulse) = self.echo.tick(dt, view, speed, false);
        if self.actor_sector.is_some()
            && let Some(sound) = &mut pulse
        {
            sound.gain *= 0.82;
        }
        (end, pulse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_and_repeated_nonresponse_change_the_selected_cue() {
        let context = || Opportunity {
            speed: 3.6,
            pressure: 0.02,
            strain: 0.04,
            familiar: false,
            covered: false,
            separated_actor: false,
            distorted: false,
            armed: true,
        };
        let mut preferences = Preferences::default();
        assert_eq!(preferences.choose(context(), 0.5), Pattern::Rush);
        for _ in 0..3 {
            preferences.observe(Pattern::Rush, true, Reaction::None);
        }
        assert_ne!(preferences.choose(context(), 0.5), Pattern::Rush);
        assert_ne!(
            Preferences::default().choose(
                Opportunity {
                    strain: 0.9,
                    pressure: 0.5,
                    ..context()
                },
                0.5
            ),
            Pattern::Rush
        );
    }

    #[test]
    fn close_investigation_can_end_a_breath_before_the_next_sound() {
        let mut chain = Chain::new(Start {
            pattern: Pattern::Breath,
            category: Category::Illusion,
            path: [Vec3::new(0., 0., -7.), Vec3::new(0., 0., -5.)],
            source_sector: u8::MAX,
            actor_sector: None,
            seed: 11,
        });
        assert!(chain.tick(0.29, Transform::IDENTITY, 0., true).1.is_some());
        assert_eq!(chain.stage, Stage::Anticipate);
        let closer = Transform::from_translation(Vec3::new(0., 0., -2.));
        assert!(chain.tick(0.81, closer, 1.8, true).0);
        assert_eq!(chain.reaction, Reaction::Approach);
    }
}
