use super::{
    cognition::{Knowledge, Memory, MemoryBook},
    nav::{HUMAN, Nav, point},
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum Goal {
    #[default]
    Wait,
    Observe,
    Approach,
    Cover,
    Feint,
    Investigate,
    Search,
    Withdraw,
}

impl Goal {
    pub(super) const ALL: [Self; 8] = [
        Self::Wait,
        Self::Observe,
        Self::Approach,
        Self::Cover,
        Self::Feint,
        Self::Investigate,
        Self::Search,
        Self::Withdraw,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Wait => "Vigiar a área",
            Self::Observe => "Estudar operador",
            Self::Approach => "Aproximar",
            Self::Cover => "Espiar de um abrigo",
            Self::Feint => "Intimidar",
            Self::Investigate => "Investigar ruído",
            Self::Search => "Procurar rastro",
            Self::Withdraw => "Retirar-se",
        }
    }
}

pub(super) struct Decision {
    pub knowledge: Knowledge,
    pub confidence: f32,
    pub exposure: f32,
    pub gap: f32,
    pub time: f32,
    pub watching: bool,
    pub cover: bool,
    pub viable_feint: bool,
    pub motion: f32,
}

impl Decision {
    pub(super) fn scores(&self) -> [f32; Goal::ALL.len()] {
        let visible = self.knowledge == Knowledge::Confirmed;
        let exposed = (self.exposure / 4.0).clamp(0.0, 1.0);
        let fatigue = (self.time / 65.0).clamp(0.0, 1.0);
        let cautious = 1.0 - self.motion;
        [
            0.29 + fatigue * 0.08,
            if !visible {
                0.0
            } else if self.watching {
                0.28 + cautious * 0.12
            } else {
                0.82
            },
            if visible && self.gap > 1.1 {
                0.40 + self.confidence * 0.18 + (self.gap / 15.0).min(0.14) - exposed * 0.08
            } else {
                0.0
            },
            if visible && self.cover {
                0.46 + exposed * 0.30
            } else {
                0.0
            },
            if visible && self.viable_feint && self.watching {
                0.40 + self.confidence * 0.16 + self.motion * 0.12
            } else {
                0.0
            },
            if self.knowledge == Knowledge::Heard {
                0.55 + self.confidence * 0.27
            } else {
                0.0
            },
            if matches!(self.knowledge, Knowledge::LastSeen | Knowledge::Region) {
                0.49 + self.confidence * 0.29
            } else {
                0.0
            },
            0.31 + fatigue * 0.36 + exposed * 0.23,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum SearchStep {
    Go { at: Vec2, link: Option<[usize; 2]> },
    Listen(f32),
}

impl SearchStep {
    pub(super) fn describe(self) -> String {
        match self {
            Self::Go { at, link } => link.map_or_else(
                || format!("Examinar {:.1}, {:.1}", at.x, at.y),
                |[from, to]| format!("Varrer passagem {from} → {to}"),
            ),
            Self::Listen(s) => format!("Escutar por {s:.1} s"),
        }
    }
}

#[derive(Default)]
pub(super) struct SearchPlan {
    pub steps: [Option<SearchStep>; 4],
    pub cursor: usize,
    pub since: f32,
    pub clue: Option<Memory>,
    pub revisions: u8,
}

impl SearchPlan {
    pub(super) fn build(
        nav: &Nav,
        memory: &MemoryBook,
        clue: Memory,
        now: f32,
        closed: bool,
    ) -> Self {
        let local = SearchStep::Go {
            at: clue.at,
            link: None,
        };
        let forecast = clue.at + (clue.velocity * 0.62).clamp_length_max(2.8);
        let probable = clue.sector.and_then(|room| {
            nav.exits(room, closed)
                .filter(|&(_, p)| {
                    p.distance_squared(clue.at) < 11.0 * 11.0 && nav.clear_body(p, p, HUMAN, closed)
                })
                .max_by(|&(a, ap), &(b, bp)| {
                    let score = |neighbor, p: Vec2| {
                        let delta = p - clue.at;
                        clue.velocity
                            .normalize_or_zero()
                            .dot(delta.normalize_or_zero())
                            * 0.47
                            + memory.rate(room, neighbor, now)
                            - delta.length() * 0.062
                    };
                    score(a, ap).total_cmp(&score(b, bp))
                })
                .map(|(id, at)| SearchStep::Go {
                    at,
                    link: Some([room, id]),
                })
        });
        let local_forecast = (forecast.distance_squared(clue.at) > 0.35
            && nav.sector(forecast) == clue.sector
            && nav.clear_body(clue.at, forecast, HUMAN, closed))
        .then_some(SearchStep::Go {
            at: forecast,
            link: None,
        });
        Self {
            steps: [
                Some(local),
                Some(SearchStep::Listen(0.44)),
                probable
                    .or(local_forecast)
                    .or(Some(SearchStep::Listen(0.66))),
                Some(SearchStep::Listen(0.79)),
            ],
            since: now,
            clue: Some(clue),
            ..default()
        }
    }

    pub(super) fn current(&self) -> Option<SearchStep> {
        self.steps.get(self.cursor).copied().flatten()
    }

    pub(super) fn next(&mut self, now: f32) {
        self.cursor += 1;
        self.since = now;
    }

    pub(super) fn can_revise(&self, clue: Memory, now: f32) -> bool {
        self.revisions < 2
            && self.clue.is_some_and(|previous| {
                clue.when > previous.when + 0.51
                    && clue.trust(now) > 0.40
                    && clue.at.distance_squared(previous.at) > 3.24
            })
    }
}

pub(super) fn intercept(
    nav: &Nav,
    memory: &MemoryBook,
    from: Vec2,
    clue: Memory,
    now: f32,
    closed: bool,
    speed: f32,
) -> Option<([usize; 2], Vec2)> {
    let sector = clue.sector?;
    let heading = clue.velocity.try_normalize()?;
    let pace = clue.velocity.length();
    if pace < 1.65 || clue.trust(now) < 0.58 {
        return None;
    }
    nav.exits(sector, closed)
        .filter_map(|(id, at)| {
            let delta = at - clue.at;
            let distance = delta.length();
            let alignment = delta.normalize_or_zero().dot(heading);
            let arrival = from.distance(at) / speed.max(1.0);
            let evidence = alignment + memory.rate(sector, id, now);
            ((2.5..=8.2).contains(&distance)
                && alignment >= 0.56
                && evidence >= 0.72
                && arrival + 0.45 < distance / pace
                && nav.clear_body(at, at, HUMAN, closed)
                && nav.clear(point(clue.at).with_y(1.25), point(at).with_y(1.25), closed))
            .then_some((id, at, evidence - arrival * 0.08))
        })
        .max_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(id, at, _)| ([sector, id], at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lab::observer::nav::{Portal, Zone};
    use ainda_vivo_core::collision::Aabb;

    #[test]
    fn interception_uses_a_witnessed_direction_and_avoids_closed_or_failed_passages() {
        let nav = Nav::new(
            vec![
                Zone {
                    min: Vec2::new(-6., -12.),
                    max: Vec2::new(6., 0.),
                },
                Zone {
                    min: Vec2::new(-6., 0.),
                    max: Vec2::new(6., 12.),
                },
            ],
            vec![Portal {
                rooms: [0, 1],
                at: Vec2::ZERO,
                width: 4.8,
                height: 3.,
            }],
            vec![],
            Some(Aabb::from_center_size(
                Vec3::new(0., 1.5, 0.),
                Vec3::new(4.8, 3., 0.24),
            )),
        );
        let clue = Memory {
            at: Vec2::new(0., -5.),
            when: 10.,
            expires: 19.,
            certainty: 0.94,
            error: 0.22,
            velocity: Vec2::Y * 2.8,
            sector: Some(0),
        };
        let from = Vec2::new(0.8, 1.);
        let mut book = MemoryBook::default();
        assert_eq!(
            intercept(&nav, &book, from, clue, 10., false, 4.95).map(|v| v.0),
            Some([0, 1])
        );
        assert!(intercept(&nav, &book, from, clue, 10., true, 4.95).is_none());
        assert!(intercept(&nav, &book, from, clue, 13., false, 4.95).is_none());
        book.failed([0, 1], 10.);
        book.failed([0, 1], 10.1);
        assert!(intercept(&nav, &book, from, clue, 10.2, false, 4.95).is_none());
        assert!(
            intercept(
                &nav,
                &book,
                from,
                Memory {
                    when: 100.,
                    expires: 109.,
                    ..clue
                },
                100.,
                false,
                4.95
            )
            .is_some()
        );
    }

    #[test]
    fn repeated_witnessed_routes_change_a_plan_and_failures_then_age_reduce_the_bias() {
        let nav = Nav::new(
            vec![
                Zone {
                    min: Vec2::new(-8., -8.),
                    max: Vec2::new(8., 8.),
                },
                Zone {
                    min: Vec2::new(-18., -8.),
                    max: Vec2::new(-8., 8.),
                },
                Zone {
                    min: Vec2::new(8., -8.),
                    max: Vec2::new(18., 8.),
                },
            ],
            vec![
                Portal {
                    rooms: [0, 1],
                    at: Vec2::new(-8., 0.),
                    width: 3.6,
                    height: 3.,
                },
                Portal {
                    rooms: [0, 2],
                    at: Vec2::new(8., 0.),
                    width: 3.6,
                    height: 3.,
                },
            ],
            Vec::new(),
            None,
        );
        let memory = |x, when, sector| Memory {
            at: Vec2::new(x, 0.),
            when,
            expires: when + 9.0,
            certainty: 0.94,
            error: 0.22,
            velocity: Vec2::ZERO,
            sector: Some(sector),
        };
        let guess = |book: &MemoryBook, now| match SearchPlan::build(
            &nav,
            book,
            memory(0.6, now, 0),
            now,
            false,
        )
        .steps[2]
        {
            Some(SearchStep::Go {
                link: Some([0, to]),
                ..
            }) => to,
            other => panic!("passagem ausente: {other:?}"),
        };
        let mut book = MemoryBook::default();
        assert_eq!(guess(&book, 3.0), 2);
        book.observe(
            Some(memory(-7.6, 2.0, 0)),
            memory(-8.4, 2.18, 1),
            &nav,
            false,
        );
        assert_eq!(guess(&book, 3.0), 2);
        for i in 4..7 {
            book.observe(
                Some(memory(-7.6, i as f32, 0)),
                memory(-8.4, i as f32 + 0.18, 1),
                &nav,
                false,
            );
        }
        assert_eq!(guess(&book, 7.0), 1);
        assert_eq!(guess(&book, 96.0), 2);
        book.failed([0, 1], 8.0);
        book.failed([0, 1], 9.0);
        assert_eq!(guess(&book, 9.0), 2);
    }
}
