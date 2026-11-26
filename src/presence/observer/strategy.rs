//! Finite plans built from observed clues and the actor's own navigation state.
#[cfg(any(feature = "dev-lab", test))]
pub(crate) mod trials;
use super::{
    brain::State,
    cognition::{Knowledge, Memory, MemoryBook},
    nav::{HUMAN, Nav, point},
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum Kind {
    Diversion,
    Interception,
    FeignedRetreat,
    Ambush,
    FalseChase,
    Investigation,
}

impl Kind {
    pub(crate) const ALL: [Self; 6] = [
        Self::Diversion,
        Self::Interception,
        Self::FeignedRetreat,
        Self::Ambush,
        Self::FalseChase,
        Self::Investigation,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Diversion => "Desvio de atenção",
            Self::Interception => "Interceptação",
            Self::FeignedRetreat => "Falsa retirada",
            Self::Ambush => "Emboscada paciente",
            Self::FalseChase => "Perseguição enganosa",
            Self::Investigation => "Investigação",
        }
    }

    fn objective(self) -> &'static str {
        match self {
            Self::Diversion => "deslocar a atenção e ganhar cobertura",
            Self::Interception => "chegar antes à passagem provável",
            Self::FeignedRetreat => "recuar e revisar a última hipótese",
            Self::Ambush => "observar uma passagem provável",
            Self::FalseChase => "induzir reação e avaliar aproximação",
            Self::Investigation => "atualizar ou descartar o rastro",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Illusion {
    Diversion,
    Rush,
}

/// Sector hypotheses originate in perception and the map, never the camera.
#[derive(Clone, Copy, Debug)]
pub(super) struct Order {
    pub id: u32,
    pub kind: Illusion,
    pub sector: usize,
    pub actor_sector: usize,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Delivery {
    Reserved,
    Played,
    Rejected,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResultKind {
    Success,
    Ignored,
    Unexpected,
    NoEvidence,
    Blocked,
    Expired,
    Protected,
    Unavailable,
    Hunt,
}

impl ResultKind {
    fn penalty(self) -> f32 {
        match self {
            Self::Blocked | Self::Unexpected => 0.20,
            Self::Ignored | Self::NoEvidence | Self::Expired => 0.13,
            Self::Success => -0.14,
            Self::Protected | Self::Unavailable | Self::Hunt => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Action {
    Move(Vec2, State),
    Hold(State, f32),
    Bait(Illusion, usize),
    Verify,
}

#[derive(Clone, Copy, Debug)]
struct Plan {
    kind: Kind,
    hypothesis: Memory,
    crossing: Option<usize>,
    actions: [Option<Action>; 5],
    cost: f32,
    initial_trust: f32,
    started: f32,
    deadline: f32,
    since: f32,
    cursor: usize,
    entered: bool,
    order: Option<u32>,
    last_delivery: Option<Delivery>,
    played_at: Option<f32>,
    observed_reaction: Option<ResultKind>,
    blocks: u32,
}

impl Plan {
    fn new(kind: Kind, clue: Memory, now: f32, actions: [Option<Action>; 5], cost: f32) -> Self {
        Self {
            kind,
            hypothesis: clue,
            crossing: None,
            actions,
            cost,
            initial_trust: clue.trust(now),
            started: now,
            deadline: now + cost.clamp(6.0, 20.0),
            since: now,
            cursor: 0,
            entered: false,
            order: None,
            last_delivery: None,
            played_at: None,
            observed_reaction: None,
            blocks: 0,
        }
    }

    fn advance(&mut self, now: f32) {
        self.cursor += 1;
        self.since = now;
        self.entered = false;
    }
}

#[derive(Clone, Copy, Debug)]
struct Record {
    kind: Kind,
    result: ResultKind,
    at: f32,
    steps: usize,
    hypothesis: Vec2,
    trust: f32,
}

#[derive(Default)]
pub(super) struct Mind {
    pub enabled: bool,
    #[cfg(feature = "dev-lab")]
    forced: Option<Kind>,
    plan: Option<Plan>,
    record: [Option<Record>; 12],
    out: Option<Order>,
    serial: u32,
    next: f32,
    pub totals: [u32; 6],
    pub finished: u32,
}

pub(super) struct Inputs<'a> {
    pub now: f32,
    pub own: Vec2,
    pub state: State,
    pub knowledge: Knowledge,
    pub clue: Option<Memory>,
    pub observation: Option<Memory>,
    pub noticed: bool,
    pub protected: bool,
    pub blocked: u32,
    pub nav: &'a Nav,
    pub memory: &'a MemoryBook,
    pub closed: bool,
    pub visited: &'a [Option<usize>; 4],
    pub hints: &'a [Vec2],
}

pub(super) enum Directive {
    Continue,
    Begin(State, Option<Vec2>),
    Release,
}

impl Mind {
    pub fn permit_hunt(&self) -> bool {
        self.plan.is_some_and(|p| {
            matches!(
                p.kind,
                Kind::FalseChase | Kind::Interception | Kind::Ambush | Kind::Investigation
            ) && matches!(
                p.actions.get(p.cursor).copied().flatten(),
                Some(Action::Hold(State::Threaten | State::Watch, _))
            )
        })
    }

    pub fn active(&self) -> bool {
        self.plan.is_some()
    }

    pub fn request(&mut self) -> Option<Order> {
        self.out.take()
    }

    pub fn waiting_for(&self, id: u32) -> bool {
        self.plan.is_some_and(|p| p.order == Some(id))
    }

    pub fn feedback(&mut self, id: u32, delivery: Delivery, now: f32) {
        if let Some(p) = &mut self.plan
            && p.order == Some(id)
            && p.last_delivery != Some(Delivery::Played)
        {
            p.last_delivery = Some(delivery);
            if delivery == Delivery::Played {
                p.played_at = Some(now);
            }
        }
    }

    pub fn hunting(&mut self, now: f32) {
        if self.plan.is_some() {
            self.finish(now, ResultKind::Hunt);
        }
    }

    #[cfg(feature = "dev-lab")]
    pub fn force(&mut self, kind: Kind) {
        self.plan = None;
        self.out = None;
        self.enabled = true;
        self.next = 0.0;
        self.forced = Some(kind);
    }

    #[cfg(feature = "dev-lab")]
    pub fn clear(&mut self) {
        *self = Self {
            enabled: self.enabled,
            ..default()
        };
    }

    fn finish(&mut self, now: f32, result: ResultKind) -> Directive {
        if let Some(p) = self.plan.take() {
            self.record.rotate_right(1);
            self.record[0] = Some(Record {
                kind: p.kind,
                result,
                at: now,
                steps: p.cursor,
                hypothesis: p.hypothesis.at,
                trust: p.initial_trust,
            });
            self.finished = self.finished.saturating_add(1);
        }
        self.out = None;
        self.next = now
            + if result == ResultKind::Protected {
                8.0
            } else {
                5.0
            };
        Directive::Release
    }

    fn bias(&self, kind: Kind, now: f32) -> f32 {
        let mut value = 0.0;
        for e in self.record.iter().flatten().filter(|e| e.kind == kind) {
            let fade = (1.0 - (now - e.at) / 110.0).clamp(0.0, 1.0);
            value -= fade * e.result.penalty();
        }
        value.clamp(-0.55, 0.23)
    }

    fn scores(&self, i: &Inputs, clue: Memory) -> [f32; 6] {
        use Kind::*;
        let visible = i.knowledge == Knowledge::Confirmed;
        let distance = i.own.distance(clue.at);
        let watched = if visible && i.noticed { 0.20 } else { 0.0 };
        let habit = clue.sector.map_or(0.0, |sector| {
            i.nav
                .exits(sector, i.closed)
                .map(|(next, _)| i.memory.rate(sector, next, i.now))
                .fold(0.0, f32::max)
        });
        let base = [
            if visible && (4.0..13.0).contains(&distance) {
                0.53 - watched
            } else {
                0.0
            },
            if visible && (clue.velocity.length_squared() > 2.8 || habit >= 0.12) {
                0.43 + habit
            } else {
                0.0
            },
            if matches!(
                i.state,
                State::Investigate | State::Approach | State::Follow
            ) && distance < 12.0
            {
                0.55 + watched
            } else {
                0.0
            },
            if distance < 16.0 && clue.sector.is_some() {
                0.47 + habit * 0.55 - watched
            } else {
                0.0
            },
            if visible
                && (5.0..13.5).contains(&distance)
                && matches!(i.state, State::Approach | State::Watch)
            {
                0.51 + i.memory.running_tendency().unwrap_or(0.28) * 0.12
            } else {
                0.0
            },
            if !visible {
                0.56 + clue.trust(i.now) * 0.13
            } else {
                0.0
            },
        ];
        let mut value = base;
        for kind in [
            Diversion,
            Interception,
            FeignedRetreat,
            Ambush,
            FalseChase,
            Investigation,
        ] {
            let n = kind as usize;
            if value[n] > 0.0 {
                value[n] = (value[n] + self.bias(kind, i.now)).max(0.0);
            }
        }
        value
    }

    fn choose(&self, i: &Inputs, clue: Memory, forced: Option<Kind>) -> Option<Plan> {
        if let Some(kind) = forced {
            return build(kind, i, clue);
        }
        let scores = self.scores(i, clue);
        let mut kinds = Kind::ALL;
        kinds.sort_unstable_by(|a, b| scores[*b as usize].total_cmp(&scores[*a as usize]));
        kinds
            .into_iter()
            .filter(|k| scores[*k as usize] > 0.25)
            .find_map(|k| build(k, i, clue))
    }

    pub fn step(&mut self, i: Inputs) -> Option<Directive> {
        if !self.enabled {
            return None;
        }
        if i.protected {
            return self
                .plan
                .is_some()
                .then(|| self.finish(i.now, ResultKind::Protected));
        }
        if self.plan.is_none() {
            if i.now < self.next {
                return None;
            }
            self.next = i.now + 1.6;
            let clue = i.clue.filter(|m| m.trust(i.now) >= 0.17)?;
            #[cfg(feature = "dev-lab")]
            let forced = self.forced;
            #[cfg(not(feature = "dev-lab"))]
            let forced = None;
            let mut planned = self.choose(&i, clue, forced)?;
            #[cfg(feature = "dev-lab")]
            {
                self.forced = None;
            }
            planned.blocks = i.blocked;
            self.totals[planned.kind as usize] =
                self.totals[planned.kind as usize].saturating_add(1);
            self.plan = Some(planned);
        }
        let p = self.plan.as_mut()?;
        if p.deadline <= i.now {
            return Some(self.finish(i.now, ResultKind::Expired));
        }
        if p.blocks < i.blocked {
            return Some(self.finish(i.now, ResultKind::Blocked));
        }
        if p.kind == Kind::Investigation
            && i.knowledge == Knowledge::Confirmed
            && i.observation.is_some_and(|m| m.when >= p.started + 0.6)
        {
            return Some(self.finish(i.now, ResultKind::Success));
        }
        if i.noticed && p.kind == Kind::Diversion && p.played_at.is_some() {
            return Some(self.finish(i.now, ResultKind::Unexpected));
        }
        if let (Some(at), Some(witness)) = (p.played_at, i.observation)
            && witness.when > at + 0.14
            && witness.certainty >= 0.57
        {
            let sector = witness.sector;
            if sector != p.hypothesis.sector {
                p.observed_reaction = Some(if sector == p.crossing {
                    ResultKind::Success
                } else {
                    ResultKind::Unexpected
                });
            } else if p.crossing.is_some_and(|to| {
                i.nav
                    .exits(sector.unwrap_or(usize::MAX), i.closed)
                    .any(|(s, portal)| {
                        s == to
                            && witness
                                .velocity
                                .dot((portal - witness.at).normalize_or_zero())
                                > 1.20
                    })
            }) {
                p.observed_reaction = Some(ResultKind::Success);
            }
        }
        if p.observed_reaction == Some(ResultKind::Unexpected) {
            return Some(self.finish(i.now, ResultKind::Unexpected));
        }
        let action = p
            .actions
            .get(p.cursor)
            .copied()
            .flatten()
            .unwrap_or(Action::Verify);
        let first = !std::mem::replace(&mut p.entered, true);
        match action {
            Action::Move(at, state) if i.own.distance(at) < 0.34 => {
                p.advance(i.now);
                Some(Directive::Begin(state, None))
            }
            Action::Move(at, state) if first => Some(Directive::Begin(state, Some(at))),
            Action::Hold(state, seconds) => {
                if i.now - p.since >= seconds {
                    p.advance(i.now);
                }
                Some(if first {
                    Directive::Begin(state, None)
                } else {
                    Directive::Continue
                })
            }
            Action::Bait(kind, sector) => {
                if first {
                    let own = i.nav.sector(i.own);
                    if own == Some(sector) || own.is_none() {
                        return Some(self.finish(i.now, ResultKind::Unavailable));
                    }
                    self.serial = self.serial.wrapping_add(1);
                    p.order = Some(self.serial);
                    p.crossing = Some(sector);
                    self.out = Some(Order {
                        id: self.serial,
                        kind,
                        sector,
                        actor_sector: own.unwrap(),
                        intensity: if kind == Illusion::Rush { 0.60 } else { 0.48 },
                    });
                    return Some(Directive::Begin(State::Watch, None));
                }
                match p.last_delivery {
                    Some(Delivery::Played) => p.advance(i.now),
                    Some(Delivery::Rejected | Delivery::Interrupted) => {
                        return Some(self.finish(i.now, ResultKind::Unavailable));
                    }
                    _ if i.now - p.since >= 2.8 => {
                        return Some(self.finish(i.now, ResultKind::Unavailable));
                    }
                    _ => {}
                }
                Some(Directive::Continue)
            }
            Action::Verify => {
                let witnessed = i.observation.filter(|m| m.when > p.started + 0.2);
                let result = match p.kind {
                    Kind::Diversion | Kind::FalseChase => {
                        p.observed_reaction.unwrap_or(if witnessed.is_some() {
                            ResultKind::Ignored
                        } else {
                            ResultKind::NoEvidence
                        })
                    }
                    Kind::Interception => match witnessed {
                        Some(m) if m.sector == p.crossing || m.at.distance(i.own) < 3.2 => {
                            ResultKind::Success
                        }
                        Some(m) if m.sector != p.hypothesis.sector => ResultKind::Unexpected,
                        _ => ResultKind::NoEvidence,
                    },
                    _ if witnessed.is_some_and(|m| m.at.distance(i.own) < 12.0) => {
                        ResultKind::Success
                    }
                    _ => ResultKind::NoEvidence,
                };
                Some(self.finish(i.now, result))
            }
            _ => Some(Directive::Continue),
        }
    }

    pub fn report(&self, now: f32) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        #[cfg(feature = "dev-lab")]
        if let Some(kind) = self.forced {
            let _ = writeln!(s, "{} aguarda pista e passagem compatíveis", kind.label());
        }
        if let Some(p) = self.plan {
            let _ = writeln!(
                s,
                "{} · {} · passo {}/{} · confiança {:.2} · custo {:.1} s · validade {:.1} s",
                p.kind.label(),
                p.kind.objective(),
                p.cursor + 1,
                p.actions.iter().flatten().count(),
                p.initial_trust,
                p.cost,
                (p.deadline - now).max(0.0)
            );
            let _ = writeln!(
                s,
                "pista={:?} margem={:.2} m setor={:?} passagem={:?} solicitação={:?} retorno={:?}\nplano={:?}",
                p.hypothesis.at,
                p.hypothesis.radius(now),
                p.hypothesis.sector,
                p.crossing,
                p.order,
                p.last_delivery,
                p.actions
            );
        } else {
            let _ = writeln!(
                s,
                "Sem plano ativo · próxima revisão em {:.1} s",
                (self.next - now).max(0.0)
            );
        }
        for e in self.record.iter().flatten() {
            let _ = writeln!(
                s,
                "{:.2}s {} {:?} passos={} pista={:?} conf={:.2} viés={:+.2}",
                e.at,
                e.kind.label(),
                e.result,
                e.steps,
                e.hypothesis,
                e.trust,
                self.bias(e.kind, now)
            );
        }
        s
    }
}

fn build(kind: Kind, i: &Inputs, clue: Memory) -> Option<Plan> {
    use Action::{Bait, Hold, Move, Verify};
    let origin = clue.sector.or_else(|| i.nav.sector(clue.at))?;
    let ours = i.nav.sector(i.own)?;
    let exits = || i.nav.exits(origin, i.closed);
    let clear = |p| i.nav.clear_body(p, p, HUMAN, i.closed);
    let heading = clue.velocity.normalize_or_zero();
    let rate = |sector, at: Vec2| {
        (at - clue.at).normalize_or_zero().dot(heading) * 0.42
            + i.memory.rate(origin, sector, i.now)
            - clue.at.distance(at) * 0.05
            - if i.visited[..2].contains(&Some(sector)) {
                0.16
            } else {
                0.0
            }
    };
    let probable = || {
        exits()
            .filter(|&(_, p)| clear(p) && p.distance(clue.at) < 12.0)
            .max_by(|&(a, p), &(b, q)| rate(a, p).total_cmp(&rate(b, q)))
    };
    let covered = |p| {
        !i.nav
            .clear(point(clue.at).with_y(1.56), point(p).with_y(1.75), i.closed)
    };
    let nearby = |p: &Vec2| (0.8..8.0).contains(&p.distance(i.own)) && clear(*p);
    let refuge = |behind: bool| {
        i.hints
            .iter()
            .copied()
            .filter(nearby)
            .filter(|&p| {
                covered(p) && (!behind || p.distance(clue.at) > i.own.distance(clue.at) + 0.8)
            })
            .min_by(|a, b| {
                a.distance_squared(i.own)
                    .total_cmp(&b.distance_squared(i.own))
            })
    };
    let aside = i.own + (i.own - clue.at).normalize_or_zero() * 1.9;
    let (actions, crossing, cost) = match kind {
        Kind::Diversion => {
            let (sector, _) = exits()
                .filter(|(s, p)| *s != ours && p.distance(clue.at) < 11.0)
                .max_by(|&(a, p), &(b, q)| rate(a, p).total_cmp(&rate(b, q)))?;
            let post =
                refuge(false).or_else(|| (clear(aside) && covered(aside)).then_some(aside))?;
            (
                [
                    Some(Bait(Illusion::Diversion, sector)),
                    Some(Move(post, State::Follow)),
                    Some(Hold(State::Watch, 2.4)),
                    Some(Verify),
                    None,
                ],
                Some(sector),
                14.0,
            )
        }
        Kind::Interception => {
            let (sector, post) = probable()?;
            let mine = i.own.distance(post);
            if mine > 6.6 || mine > clue.at.distance(post) * 0.75 + 0.6 {
                return None;
            }
            (
                [
                    Some(Move(post, State::Follow)),
                    Some(Hold(State::Watch, 2.4)),
                    Some(Verify),
                    None,
                    None,
                ],
                Some(sector),
                mine / 0.65 + 4.2,
            )
        }
        Kind::FeignedRetreat => {
            let post = refuge(true).or_else(|| clear(aside).then_some(aside))?;
            let inspect = clue.at + (post - clue.at).normalize_or_zero() * 3.4;
            if !clear(inspect) {
                return None;
            }
            (
                [
                    Some(Move(post, State::Retreat)),
                    Some(Hold(State::Rest, 1.7)),
                    Some(Move(inspect, State::Investigate)),
                    Some(Hold(State::Watch, 1.8)),
                    Some(Verify),
                ],
                None,
                17.0,
            )
        }
        Kind::Ambush => {
            let (sector, portal) = probable()?;
            let post = i
                .hints
                .iter()
                .copied()
                .filter(nearby)
                .filter(|p| {
                    covered(*p)
                        && (1.2..6.4).contains(&p.distance(portal))
                        && i.nav
                            .clear(point(*p).with_y(1.75), point(portal).with_y(1.5), i.closed)
                })
                .min_by(|a, b| {
                    a.distance_squared(i.own)
                        .total_cmp(&b.distance_squared(i.own))
                })?;
            (
                [
                    Some(Move(post, State::Follow)),
                    Some(Hold(State::Watch, 6.5)),
                    Some(Verify),
                    None,
                    None,
                ],
                Some(sector),
                i.own.distance(post) / 0.65 + 8.1,
            )
        }
        Kind::FalseChase => {
            let sector = exits()
                .filter(|(s, p)| *s != ours && p.distance(clue.at) < 10.0)
                .max_by(|&(a, p), &(b, q)| rate(a, p).total_cmp(&rate(b, q)))?
                .0;
            (
                [
                    Some(Bait(Illusion::Rush, sector)),
                    Some(Hold(State::Threaten, 3.2)),
                    Some(Hold(State::Watch, 3.2)),
                    Some(Verify),
                    None,
                ],
                Some(sector),
                11.5,
            )
        }
        Kind::Investigation => {
            let remaining = i.own.distance(clue.at);
            let max_leg = 7.6;
            let (p, next) = if remaining <= max_leg && clear(clue.at) {
                (
                    clue.at,
                    probable().filter(|(_, at)| at.distance(clue.at) < 3.2),
                )
            } else {
                // Pursue the nearest reachable portion of a vague lead. The
                // walking motor still validates the path against live obstacles.
                let portal = i
                    .nav
                    .exits(ours, i.closed)
                    .filter(|&(_, at)| {
                        clear(at)
                            && at.distance(i.own) <= max_leg
                            && at.distance(clue.at) + 0.8 < remaining
                            && i.nav.acoustic(at, clue.at, i.closed) > 0.0
                    })
                    .min_by(|&(_, a), &(_, b)| {
                        a.distance_squared(clue.at)
                            .total_cmp(&b.distance_squared(clue.at))
                    });
                let p = portal.map(|(_, at)| at).or_else(|| {
                    let toward = i.own + (clue.at - i.own).clamp_length_max(max_leg);
                    (origin == ours && i.nav.clear_body(i.own, toward, HUMAN, i.closed))
                        .then_some(toward)
                })?;
                (p, None)
            };
            let estimated_metres = i.own.distance(p) + next.map_or(0.0, |(_, at)| p.distance(at));
            (
                [
                    Some(Move(p, State::Investigate)),
                    Some(Hold(State::Investigate, 1.1)),
                    next.map(|(_, at)| Move(at, State::Follow)),
                    Some(Hold(State::Watch, 2.2)),
                    Some(Verify),
                ],
                next.map(|(sector, _)| sector),
                estimated_metres / 0.78 + 6.5,
            )
        }
    };
    let mut compacted = [None; 5];
    for (k, action) in actions.into_iter().flatten().enumerate() {
        compacted[k] = Some(action);
    }
    let mut plan = Plan::new(kind, clue, i.now, compacted, cost);
    plan.crossing = crossing;
    Some(plan)
}
