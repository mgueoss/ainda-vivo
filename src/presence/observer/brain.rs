pub(super) use super::cognition::Memory;
use super::cognition::{Knowledge, MemoryBook, best_clue};
use super::hunt::Hunt;
use super::nav::{HUMAN, LOW, Nav, RouteScratch, point};
use super::strategy::{Directive, Inputs, Mind};
use super::tactics::{Decision, Goal, SearchPlan, SearchStep};
use bevy::prelude::*;

pub(super) const STEP: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Absent,
    Rest,
    Explore,
    Watch,
    Investigate,
    Peek,
    Follow,
    Approach,
    Threaten,
    Retreat,
}

impl State {
    #[cfg(feature = "dev-lab")]
    pub const ACTIVE: [Self; 9] = [
        Self::Rest,
        Self::Explore,
        Self::Watch,
        Self::Investigate,
        Self::Peek,
        Self::Follow,
        Self::Approach,
        Self::Threaten,
        Self::Retreat,
    ];

    #[cfg(feature = "dev-lab")]
    pub fn label(self) -> &'static str {
        match self {
            Self::Absent => "Ausente",
            Self::Rest => "Repouso",
            Self::Explore => "Exploração",
            Self::Watch => "Observação",
            Self::Investigate => "Investigação",
            Self::Peek => "Espionagem",
            Self::Follow => "Acompanhamento",
            Self::Approach => "Aproximação",
            Self::Threaten => "Intimidação",
            Self::Retreat => "Retirada",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tuning {
    pub fov: f32,
    pub range: f32,
    pub hearing: f32,
    pub speed: f32,
    pub watching: f32,
    pub investigation: f32,
    pub approach: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            fov: 100.0,
            range: 20.0,
            hearing: 1.0,
            speed: 1.0,
            watching: 7.0,
            investigation: 9.0,
            approach: 5.2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Noise {
    pub at: Vec2,
    pub power: f32,
}

#[derive(Clone, Copy)]
pub struct View {
    pub eye: Vec3,
    pub facing: Vec3,
    pub protected: bool,
}

pub struct Frame<'a> {
    pub nav: &'a Nav,
    pub door_closed: bool,
    pub view: View,
    pub noises: &'a [Noise],
    pub points: &'a [Vec2],
}

#[derive(Default)]
pub struct Brain {
    pub state: State,
    pub reason: &'static str,
    pub pos: Vec2,
    pub yaw: f32,
    pub head: f32,
    pub clock: f32,
    pub born: f32,
    pub entered: f32,
    pub next_manifest: f32,
    pub seen: Option<Memory>,
    pub heard: Option<Memory>,
    pub hit_sector: Option<usize>,
    pub visited: [Option<usize>; 4],
    pub recent: [State; 4],
    pub ray: Option<(Vec3, Vec3, bool)>,
    pub goal: Option<Vec2>,
    pub path: Vec<Vec2>,
    pub waypoint: usize,
    pub moving: f32,
    pub hunt: Hunt,
    pub(super) mind: Mind,
    pub(super) memory: MemoryBook,
    pub(super) route: RouteScratch,
    pub(super) intended: Goal,
    pub(super) scores: [f32; Goal::ALL.len()],
    pub(super) plan: SearchPlan,
    rng: u32,
    sample: f32,
    exposure: f32,
    glimpse: Option<Vec2>,
    pub(super) visible: bool,
    noticed: f32,
    limit: f32,
    retry_at: f32,
    stuck: f32,
    old_door: bool,
    failed_target: Option<Vec2>,
    rest_at: Vec2,
    review_at: f32,
    used_clue: Option<f32>,
    remainder: f32,
    pending_noise: [Noise; 6],
    noise_count: usize,
    previous_eye: Option<Vec3>,
    foot_route_until: f32,
}

pub fn forward(yaw: f32) -> Vec2 {
    Vec2::new(yaw.sin(), yaw.cos())
}
pub(super) fn angle(d: Vec2) -> f32 {
    d.x.atan2(d.y)
}
pub(super) fn turn(current: f32, desired: f32, amount: f32) -> f32 {
    let diff = (desired - current + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    current + diff.clamp(-amount, amount)
}

impl Brain {
    pub(super) fn resume_on_foot(&mut self, at: Vec2) {
        self.change(State::Investigate, "retomou pista andando", 9.0, Some(at));
        self.foot_route_until = self.clock + 9.0;
    }

    pub(super) fn walking_portal(&self) -> bool {
        self.goal.is_some() && self.clock < self.foot_route_until
    }

    pub(super) fn campaign(seed: u32) -> Self {
        let mut brain = Self::seeded(seed);
        brain.mind.enabled = true;
        brain.next_manifest = 12.0;
        brain
    }

    pub fn seeded(seed: u32) -> Self {
        Self {
            rng: seed.max(1),
            ..default()
        }
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    pub fn eye(&self) -> Vec3 {
        point(self.pos).with_y(if self.low() { 0.87 } else { 1.96 })
    }

    pub fn low(&self) -> bool {
        if self.hunt.active() {
            return self.hunt.low();
        }
        self.state == State::Threaten && (1.05..2.15).contains(&(self.clock - self.entered))
    }

    pub fn spawn(&mut self, at: Vec2, yaw: f32) {
        if self.state != State::Absent {
            return;
        }
        self.pos = at;
        self.rest_at = at;
        self.yaw = yaw;
        self.head = 0.0;
        self.born = self.clock;
        self.noticed = 0.0;
        self.exposure = 0.0;
        self.glimpse = None;
        self.visible = false;
        self.ray = None;
        self.failed_target = None;
        self.change(State::Rest, "janela de manifestação", 1.4, None);
    }

    pub(super) fn change(
        &mut self,
        state: State,
        reason: &'static str,
        duration: f32,
        goal: Option<Vec2>,
    ) {
        self.recent.rotate_right(1);
        self.recent[0] = self.state;
        self.state = state;
        self.reason = reason;
        self.entered = self.clock;
        self.limit = self.clock + duration;
        self.goal = goal;
        self.path.clear();
        self.waypoint = 0;
        self.stuck = 0.0;
        self.retry_at = self.clock;
        self.plan = SearchPlan::default();
        self.memory.record(self.clock, reason, self.hit_sector);
    }

    fn sample(&mut self, f: &Frame, config: Tuning, elapsed: f32) {
        let eye = self.eye();
        let target = f.view.eye - Vec3::Y * 0.10;
        let delta = target - eye;
        let heading = forward(self.yaw + self.head);
        let cone = (config.fov.to_radians() * 0.5).cos();
        let edge = (config.fov.to_radians() * 0.5 + 0.45).min(1.61).cos();
        let dot = heading.dot(delta.xz().normalize_or_zero());
        let frontal = dot >= cone || delta.length_squared() < 1.2 * 1.2;
        let peripheral = dot >= edge && delta.length_squared() <= (config.range * 0.68).powi(2);
        let in_range = delta.length_squared() <= config.range * config.range && delta.y.abs() < 4.0;
        let clear = (frontal || peripheral) && in_range && f.nav.clear(eye, target, f.door_closed);
        let moving = self
            .glimpse
            .is_some_and(|old| target.xz().distance_squared(old) > (elapsed * 0.8).powi(2));
        self.glimpse = clear.then_some(target.xz());
        self.ray = in_range.then_some((eye, target, clear));
        let track = clear && (frontal || moving || self.visible || delta.length_squared() < 6.25);
        let weight = if frontal { 1.0 } else { 0.69 };
        self.exposure = if track {
            (self.exposure + elapsed * weight).min(0.5)
        } else {
            (self.exposure - elapsed * 1.75).max(0.0)
        };
        let was = self.visible;
        self.visible = track && self.exposure >= if was { 0.13 } else { 0.25 };
        if self.visible {
            let sector = f.nav.sector(target.xz());
            let measured = self
                .seen
                .filter(|p| (0.025..=0.42).contains(&p.age(self.clock)))
                .map(|p| {
                    let v = ((target.xz() - p.at) / p.age(self.clock)).clamp_length_max(6.4);
                    if v.length_squared() < 0.04 || v.dot(p.velocity) < 0.0 {
                        v
                    } else {
                        p.velocity.lerp(v, 0.73)
                    }
                })
                .unwrap_or(Vec2::ZERO);
            let current = Memory {
                at: target.xz(),
                when: self.clock,
                expires: self.clock + 9.0,
                certainty: if frontal { 0.94 } else { 0.57 },
                error: if frontal { 0.22 } else { 0.74 },
                velocity: measured,
                sector,
            };
            self.memory
                .observe(self.seen, current, f.nav, f.door_closed);
            self.seen = Some(current);
            self.hit_sector = sector;
            if !was {
                self.memory.sightings += 1;
                self.memory
                    .record(self.clock, "confirmou o operador pela visão", sector);
            }
        } else if was {
            self.memory.missed(self.clock, self.hit_sector);
        }
        let in_player_view = self.visible && player_sees(f, eye);
        self.noticed = if in_player_view {
            self.noticed + elapsed
        } else {
            (self.noticed - elapsed).max(0.0)
        };
    }

    fn listen(&mut self, f: &Frame, config: Tuning) {
        let mut loudest = 0.21;
        for &noise in f.noises {
            let distance = noise.at.distance(self.pos);
            if distance > 27.0 {
                continue;
            }
            let connected = f.nav.acoustic(self.pos, noise.at, f.door_closed);
            let unobstructed = f
                .nav
                .clear(self.eye(), point(noise.at).with_y(0.8), f.door_closed);
            let power =
                noise.power * config.hearing * connected * if unobstructed { 1.0 } else { 0.55 }
                    / (1.0 + distance * 0.12);
            if power > loudest {
                let rounded = (noise.at / 1.6).round() * 1.6;
                let at = if f.nav.sector(rounded) == f.nav.sector(noise.at) {
                    rounded
                } else {
                    noise.at
                };
                let confidence = (0.26 + power * 0.35).clamp(0.27, 0.79);
                if self.heard.is_some_and(|old| {
                    old.at.distance_squared(at) < 1.0
                        && old.age(self.clock) < 0.32
                        && old.certainty > confidence - 0.09
                }) {
                    continue;
                }
                loudest = power;
                self.heard = Some(Memory {
                    at,
                    when: self.clock,
                    expires: self.clock + 8.0,
                    certainty: confidence,
                    error: 0.9 + distance * 0.06 + if unobstructed { 0.0 } else { 1.4 },
                    velocity: Vec2::ZERO,
                    sector: f.nav.sector(at),
                });
            }
        }
        if loudest > 0.21
            && let Some(noise) = self.heard
        {
            self.memory.noises += 1;
            self.memory
                .record(self.clock, "ouviu um ruído físico", noise.sector);
        }
    }

    pub(super) fn knowledge(&self) -> (Knowledge, Option<Memory>) {
        best_clue(self.clock, self.visible, self.seen, self.heard)
    }

    #[cfg(feature = "dev-lab")]
    pub(super) fn forget(&mut self) {
        self.mind.clear();
        self.memory = MemoryBook::default();
        self.seen = None;
        self.heard = None;
        self.exposure = 0.0;
        self.glimpse = None;
        self.visible = false;
        self.hit_sector = None;
        self.visited = [None; 4];
        self.recent = [State::Absent; 4];
        self.used_clue = None;
        self.plan = SearchPlan::default();
        self.hunt.forget_path();
        self.path.clear();
        self.goal = None;
        self.waypoint = 0;
        self.review_at = self.clock;
        self.memory
            .record(self.clock, "memória limpa no laboratório", None);
    }

    #[cfg(feature = "dev-lab")]
    pub(super) fn diagnosis(&self) -> String {
        use std::fmt::Write;

        let (knowledge, clue) = self.knowledge();
        let (objective, plan) = if self.hunt.active() {
            (self.hunt.strategy.label(), &self.hunt.search)
        } else {
            (self.intended.label(), &self.plan)
        };
        let mut out = format!(
            "{objective} · {}\n{} · {}",
            self.state.label(),
            knowledge.label(),
            self.reason
        );
        if let Some(m) = clue {
            let _ = write!(
                out,
                "\nIndício {:.1}, {:.1} · {:.0}% · raio {:.1} m · há {:.1} s",
                m.at.x,
                m.at.y,
                100.0 * m.trust(self.clock),
                m.radius(self.clock),
                m.age(self.clock)
            );
        }
        for (i, step) in plan.steps.iter().flatten().enumerate().skip(plan.cursor) {
            let mark = if i == plan.cursor { "›" } else { "·" };
            let _ = write!(out, "\n{mark} {}", step.describe());
        }
        let _ = write!(out, "\n\nUTILIDADES");
        for goal in Goal::ALL {
            let _ = write!(out, "\n{} {:.2}", goal.label(), self.scores[goal as usize]);
        }
        let _ = write!(out, "\n\nMEMÓRIA");
        for habit in self
            .memory
            .routes
            .iter()
            .flatten()
            .filter(|h| self.clock - h.when < 88.0)
        {
            let [a, b] = habit.link;
            let _ = write!(
                out,
                "\n{a} → {b} · {} vistos · {} falhas · peso {:.2}",
                habit.crossings,
                habit.failures,
                self.memory.rate(a, b, self.clock)
            );
        }
        let running = self
            .memory
            .running_tendency()
            .map_or("—".into(), |v| format!("{:.0}%", v * 100.0));
        let _ = write!(
            out,
            "\nCorrida observada {running}\nPlanos {} · bloqueios {} · retomadas {} · manobras {}\nGrade {} nós / {} limites · {} KiB de busca",
            self.memory.replans,
            self.memory.blocked,
            self.memory.reacquired,
            self.memory.recovered,
            self.route.expanded,
            self.route.budget_hits,
            self.route.bytes() / 1024
        );
        let _ = write!(out, "\n\nEPISÓDIOS");
        for e in self.memory.episodes.iter().flatten() {
            let _ = write!(out, "\n{:.1}s · {:?} · {}", e.when, e.sector, e.reason);
        }
        out
    }

    fn pocket(&self, f: &Frame, avoid: Vec2) -> Option<Vec2> {
        f.points
            .iter()
            .copied()
            .filter(|&p| {
                let distance = p.distance(self.pos);
                (2.0..19.0).contains(&distance)
                    && p.distance(avoid) > 4.0
                    && self
                        .failed_target
                        .is_none_or(|old| p.distance_squared(old) > 1.0)
                    && f.nav.clear_body(p, p, HUMAN, f.door_closed)
            })
            .min_by(|a, b| {
                let score = |&p: &Vec2| {
                    p.distance(self.pos)
                        + if self.visited.contains(&f.nav.sector(p)) {
                            3.0
                        } else {
                            0.0
                        }
                        + if self.seen.or(self.heard).is_some()
                            && f.nav.clear(
                                point(avoid).with_y(1.55),
                                point(p).with_y(1.9),
                                f.door_closed,
                            )
                        {
                            9.0
                        } else {
                            0.0
                        }
                };
                score(a).total_cmp(&score(b))
            })
    }

    fn peek(&self, f: &Frame, remembered: Vec2) -> Option<Vec2> {
        let facing = (remembered - self.pos).normalize_or_zero();
        let side = Vec2::new(facing.y, -facing.x);
        [1.0, 2.2, 3.4]
            .into_iter()
            .flat_map(|d| [-1.0, 1.0].map(move |s| self.pos + side * (d * s)))
            .find(|&p| {
                let observer = point(remembered).with_y(1.55);
                let center = point(p).with_y(1.94);
                f.nav
                    .clear_body(p, p, Vec3::new(0.66, 1.12, 0.52), f.door_closed)
                    && !f.nav.clear(observer, center, f.door_closed)
                    && f.nav
                        .clear(observer, center + point(side) * 0.34, f.door_closed)
                    && f.nav.clear_body(self.pos, p, HUMAN, f.door_closed)
            })
    }

    fn stance_target(&self, at: Vec2, distance: f32) -> Vec2 {
        at + (self.pos - at).normalize_or_zero() * distance.min(at.distance(self.pos))
    }

    #[cfg(any(feature = "dev-lab", test))]
    pub fn direct(&mut self, state: State, target: Vec2, config: Tuning) {
        let duration = if state == State::Investigate {
            config.investigation
        } else {
            config.watching
        };
        let goal = matches!(
            state,
            State::Explore
                | State::Investigate
                | State::Peek
                | State::Follow
                | State::Approach
                | State::Retreat
        )
        .then_some(target);
        self.change(state, "controle manual", duration, goal);
    }

    pub(super) fn retreat(&mut self, f: &Frame, why: &'static str, from: Vec2) {
        let target = self
            .pocket(f, from)
            .or_else(|| {
                [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
                    .into_iter()
                    .map(|d| self.pos + d * 2.4)
                    .find(|&p| {
                        p.distance(from) >= self.pos.distance(from)
                            && self
                                .failed_target
                                .is_none_or(|old| old.distance_squared(p) > 1.0)
                            && f.nav.clear_body(self.pos, p, HUMAN, f.door_closed)
                            && self.seen.or(self.heard).is_none_or(|memory| {
                                !f.nav.clear(
                                    point(memory.at).with_y(1.55),
                                    point(p).with_y(1.96),
                                    f.door_closed,
                                )
                            })
                    })
            })
            .unwrap_or(self.rest_at);
        self.change(State::Retreat, why, 14.0, Some(target));
        self.intended = Goal::Withdraw;
    }

    fn decide(&mut self, f: &Frame, config: Tuning) {
        let ended = self.clock >= self.limit;
        let (knowledge, clue) = self.knowledge();
        let known = clue.map_or(self.rest_at, |m| m.at);
        let arrived = self.goal.is_none_or(|p| p.distance(self.pos) < 0.23);
        if self.state == State::Retreat {
            if (ended || arrived) && !player_sees(f, self.eye()) {
                self.next_manifest = self.clock + 24.0 + 25.0 * self.random();
                self.change(State::Absent, "recuou fora de vista", 0.0, None);
            } else if (ended || arrived) && self.clock >= self.retry_at {
                self.retreat(f, "busca outro abrigo", known);
                self.retry_at = self.clock + 2.0;
            }
            return;
        }
        if self.state == State::Threaten {
            if ended && self.intended == Goal::Feint && self.visible && self.noticed < 3.0 {
                self.change(State::Watch, "permanece de vigia após a ameaça", 3.6, None);
                self.intended = Goal::Observe;
            } else if ended {
                self.retreat(f, "ameaça encerrada", known);
            }
            return;
        }
        let linger = if matches!(self.state, State::Watch | State::Peek) && self.noticed < 1.4 {
            85.0
        } else {
            49.0
        };
        if self.clock - self.born > linger
            || self.visible && self.seen.is_some_and(|m| m.at.distance(self.pos) < 2.65)
        {
            self.retreat(f, "encerra a exposição", known);
            return;
        }
        if f.view.protected {
            if self.visible || matches!(self.state, State::Watch | State::Peek | State::Approach) {
                self.retreat(f, "protege o momento de atenção", known);
            } else if matches!(self.state, State::Rest | State::Explore)
                && let Some(noise) = self.heard
            {
                let goal = self.stance_target(noise.at, config.approach.max(4.8));
                if goal.distance(self.pos) > 0.23 {
                    self.change(
                        State::Investigate,
                        "ouve de longe durante a interação",
                        config.investigation,
                        Some(goal),
                    );
                    self.intended = Goal::Investigate;
                }
            }
            return;
        }
        if self.state == State::Peek {
            if ended || self.goal.is_none() && self.noticed > 1.55 {
                self.retreat(f, "encerra a espiada", known);
            }
            return;
        }

        let sight = knowledge == Knowledge::Confirmed;
        let watching = self.state == State::Watch;
        if sight && self.state == State::Approach {
            if known.distance(self.pos) <= config.approach + 0.55 && self.can_threaten(f) {
                self.change(State::Threaten, "ameaça sem avanço", 3.4, None);
                self.intended = Goal::Feint;
            } else if ended {
                self.change(State::Watch, "reavalia a aproximação", 2.8, None);
                self.intended = Goal::Observe;
            } else {
                let goal = self.stance_target(known, config.approach);
                if self
                    .goal
                    .is_none_or(|old| old.distance_squared(goal) > 0.30)
                {
                    self.set_goal(Some(goal));
                }
            }
            return;
        }
        if watching && sight && !ended && self.noticed < 3.1
            || !sight
                && matches!(self.state, State::Watch | State::Approach)
                && self
                    .seen
                    .is_some_and(|visual| visual.age(self.clock) < 0.60)
        {
            return;
        }
        if !sight && self.plan.clue.is_some() && !ended {
            self.investigate(f, clue);
            return;
        }
        if !sight && !ended && !arrived && knowledge == Knowledge::Unknown {
            return;
        }
        let new_evidence = sight && !watching
            || knowledge == Knowledge::Heard
                && matches!(self.state, State::Rest | State::Explore)
                && clue.is_some_and(|m| m.age(self.clock) < 0.22);
        if self.clock < self.review_at && !new_evidence {
            return;
        }
        self.review_at = self.clock + 0.31;
        self.choose(f, config, knowledge, clue);
    }

    fn choose(&mut self, f: &Frame, config: Tuning, knowledge: Knowledge, clue: Option<Memory>) {
        let known = clue.map_or(self.rest_at, |m| m.at);
        let sight = knowledge == Knowledge::Confirmed;
        let watching = self.state == State::Watch;
        let cover = if sight && watching && !self.recent.contains(&State::Peek) {
            self.peek(f, known)
        } else {
            None
        };
        let gap = self.pos.distance(known) - config.approach;
        let decision = Decision {
            knowledge,
            confidence: clue.map_or(0.0, |m| m.trust(self.clock)),
            exposure: self.noticed,
            gap,
            time: self.clock - self.born,
            watching,
            cover: cover.is_some(),
            viable_feint: sight && (0.1..4.3).contains(&gap) && self.can_threaten(f),
            motion: self.memory.running_tendency().unwrap_or(0.45),
        };
        let mut scores = decision.scores();
        if !sight && clue.is_some_and(|m| self.used_clue.is_some_and(|used| m.when <= used + 0.2)) {
            scores[Goal::Search as usize] = 0.0;
            scores[Goal::Investigate as usize] = 0.0;
        }
        if self.recent.contains(&State::Approach) {
            scores[Goal::Approach as usize] *= 0.76;
        }
        if self.recent.contains(&State::Threaten) {
            scores[Goal::Feint as usize] *= 0.56;
        }
        if !sight && self.noticed < 1.0 && !self.recent.contains(&State::Explore) {
            scores[Goal::Withdraw as usize] *= 0.74;
        }
        for goal in Goal::ALL {
            if scores[goal as usize] > 0.0 {
                scores[goal as usize] +=
                    self.random() * 0.032 + if self.intended == goal { 0.035 } else { 0.0 };
            }
        }
        let goal = Goal::ALL
            .into_iter()
            .max_by(|a, b| scores[*a as usize].total_cmp(&scores[*b as usize]))
            .unwrap();
        self.scores = scores;
        self.intended = goal;

        match goal {
            Goal::Observe => {
                let duration = config.watching * (0.83 + self.random() * 0.60);
                self.change(
                    State::Watch,
                    "observa a partir de contato visual",
                    duration,
                    None,
                );
            }
            Goal::Approach => self.change(
                State::Approach,
                "aproximação medida",
                8.2,
                Some(self.stance_target(known, config.approach)),
            ),
            Goal::Cover => self.change(State::Peek, "borda ao alcance com cobertura", 6.6, cover),
            Goal::Feint => self.change(State::Threaten, "ameaça de ruptura", 3.4, None),
            Goal::Investigate | Goal::Search => {
                if let Some(clue) = clue {
                    let (state, reason) = if goal == Goal::Investigate {
                        (State::Investigate, "ruído localizado aproximadamente")
                    } else {
                        (State::Follow, "procura após perder o rastro")
                    };
                    self.change(state, reason, config.investigation, Some(clue.at));
                    self.used_clue = Some(clue.when);
                    self.plan =
                        SearchPlan::build(f.nav, &self.memory, clue, self.clock, f.door_closed);
                    self.investigate(f, None);
                }
            }
            Goal::Wait if self.clock < self.limit && self.state == State::Rest => {}
            Goal::Wait => {
                if let Some(target) = self.pocket(f, known) {
                    self.change(
                        State::Explore,
                        "varre um setor pouco observado",
                        14.0,
                        Some(target),
                    );
                } else {
                    self.change(State::Rest, "escuta sem uma rota útil", 2.1, None);
                }
            }
            Goal::Withdraw => self.retreat(f, "terminou de observar a área", known),
        }
    }

    fn set_goal(&mut self, goal: Option<Vec2>) {
        if self.goal != goal {
            self.goal = goal;
            self.path.clear();
            self.waypoint = 0;
            self.retry_at = self.clock;
        }
    }

    fn investigate(&mut self, f: &Frame, latest: Option<Memory>) {
        if let Some(clue) = latest
            && self.plan.can_revise(clue, self.clock)
        {
            let revisions = self.plan.revisions + 1;
            self.plan = SearchPlan::build(f.nav, &self.memory, clue, self.clock, f.door_closed);
            self.plan.revisions = revisions;
            self.used_clue = Some(clue.when);
            self.memory
                .record(self.clock, "ajustou a busca a um novo indício", clue.sector);
        }
        match self.plan.current() {
            Some(SearchStep::Go { at, .. }) if self.pos.distance_squared(at) > 0.72 * 0.72 => {
                self.set_goal(Some(at));
            }
            Some(SearchStep::Listen(duration)) if self.clock - self.plan.since < duration => {
                self.set_goal(None);
            }
            Some(_) => {
                self.plan.next(self.clock);
                self.set_goal(None);
            }
            None => {
                self.change(State::Rest, "examinou a área sem confirmação", 2.8, None);
                self.intended = Goal::Wait;
            }
        }
    }

    fn can_threaten(&self, f: &Frame) -> bool {
        let reach = LOW.x.max(LOW.z);
        f.nav.clear_body(
            self.pos,
            self.pos,
            Vec3::new(reach, 1.12, reach),
            f.door_closed,
        )
    }

    pub fn step(&mut self, dt: f32, f: &Frame, config: Tuning, automatic: bool) {
        let dt = dt.clamp(0.0, 0.25);
        let old_eye = self
            .previous_eye
            .replace(f.view.eye)
            .filter(|old| old.distance_squared(f.view.eye) < 1.7 * 1.7)
            .unwrap_or(f.view.eye);
        if self.state == State::Absent {
            self.clock += dt;
            self.moving = 0.0;
            self.seen = self.seen.filter(|m| m.expires > self.clock);
            self.heard = self.heard.filter(|m| m.expires > self.clock);
            self.remainder = 0.0;
            self.noise_count = 0;
            return;
        }
        for &noise in f.noises {
            if self.noise_count < self.pending_noise.len() {
                self.pending_noise[self.noise_count] = noise;
                self.noise_count += 1;
            }
        }
        self.remainder += dt;
        let ticks = ((self.remainder + 1.0e-6) / STEP) as usize;
        if ticks == 0 {
            return;
        }
        self.remainder = (self.remainder - ticks as f32 * STEP).max(0.0);
        let noises = self.pending_noise;
        let count = std::mem::take(&mut self.noise_count);
        for tick in 0..ticks {
            let frame = Frame {
                nav: f.nav,
                door_closed: f.door_closed,
                noises: if tick == 0 { &noises[..count] } else { &[] },
                points: f.points,
                view: View {
                    eye: old_eye.lerp(f.view.eye, (tick + 1) as f32 / ticks as f32),
                    ..f.view
                },
            };
            self.tick(&frame, config, automatic);
        }
    }

    fn tick(&mut self, f: &Frame, config: Tuning, automatic: bool) {
        let dt = STEP;
        self.clock += dt;
        self.moving = 0.0;
        self.seen = self.seen.filter(|m| m.expires > self.clock);
        self.heard = self.heard.filter(|m| m.expires > self.clock);
        if self.state == State::Absent {
            return;
        }
        if !f.noises.is_empty() {
            self.listen(f, config);
        }
        self.sample += dt;
        if self.sample >= if self.hunt.active() { 0.095 } else { 0.14 } {
            let elapsed = std::mem::take(&mut self.sample);
            self.sample(f, config, elapsed);
            if self.walking_portal() && f.view.protected {
                self.foot_route_until = 0.0;
                self.set_goal(None);
            }
            if automatic && !self.hunt.active() && !self.walking_portal() && !self.strategic(f) {
                self.decide(f, config);
            }
        }
        if self.hunt.active() {
            let mut hunt = std::mem::take(&mut self.hunt);
            self.aim_head(self.seen.or(self.heard).map(|m| m.at), dt);
            hunt.advance(self, dt, f);
            self.hunt = hunt;
            return;
        }
        let target = self.seen.or(self.heard).map(|m| m.at);
        if matches!(
            self.state,
            State::Watch
                | State::Peek
                | State::Threaten
                | State::Approach
                | State::Follow
                | State::Investigate
        ) && let Some(at) = target
        {
            self.aim_head(Some(at), dt);
            if self.state != State::Watch && self.goal.is_none() {
                let next = turn(self.yaw, angle(at - self.pos), dt * 1.2);
                if f.nav.clear_body(self.pos, self.pos, HUMAN, f.door_closed) {
                    self.yaw = next;
                }
            }
        } else {
            self.head = turn(self.head, 0.0, dt);
        }
        if matches!(self.plan.current(), Some(SearchStep::Listen(_))) {
            self.head = ((self.clock - self.plan.since) * 4.1).sin() * 0.45;
        }
        self.walk(dt, f, config, automatic);
    }

    fn strategic(&mut self, f: &Frame) -> bool {
        if !self.mind.enabled || self.state == State::Rest && self.clock - self.born < 1.35 {
            return false;
        }
        let (knowledge, clue) = self.knowledge();
        let directive = self.mind.step(Inputs {
            now: self.clock,
            own: self.pos,
            state: self.state,
            knowledge,
            clue,
            observation: self.seen.filter(|_| self.visible),
            noticed: self.noticed >= 0.75,
            protected: f.view.protected,
            blocked: self.memory.blocked,
            nav: f.nav,
            memory: &self.memory,
            closed: f.door_closed,
            visited: &self.visited,
            hints: f.points,
        });
        match directive {
            Some(Directive::Begin(state, destination)) => {
                self.change(state, "executa plano estratégico", 2.5, destination);
                true
            }
            Some(Directive::Continue) => true,
            Some(Directive::Release) => {
                self.set_goal(None);
                false
            }
            None => false,
        }
    }

    fn aim_head(&mut self, target: Option<Vec2>, dt: f32) {
        let local = target.map_or(0.0, |at| {
            turn(self.yaw, angle(at - self.pos), 0.49) - self.yaw
        });
        self.head = turn(self.head, local, dt * 2.8);
    }

    fn walk(&mut self, dt: f32, f: &Frame, config: Tuning, automatic: bool) {
        let Some(goal) = self.goal else { return };
        if self.pos.distance(goal) < 0.11 {
            let sector = f.nav.sector(self.pos);
            if self.visited[0] != sector {
                self.visited.rotate_right(1);
                self.visited[0] = sector;
            }
            self.goal = None;
            self.path.clear();
            if self.state == State::Peek {
                self.entered = self.clock;
                self.limit = self.clock + 2.7;
            }
            return;
        }
        if self.old_door != f.door_closed {
            self.old_door = f.door_closed;
            self.failed_target = None;
            self.path.clear();
            self.retry_at = self.clock;
        }
        if self.path.is_empty() && self.clock >= self.retry_at {
            self.retry_at = self.clock + 0.8;
            self.waypoint = 0;
            self.memory.replans += 1;
            if f.nav.sector(self.pos) == f.nav.sector(goal)
                && f.nav.clear_body(self.pos, goal, HUMAN, f.door_closed)
            {
                self.path.push(goal);
            } else if !f.nav.plan(
                self.pos,
                goal,
                f.door_closed,
                false,
                &mut self.path,
                &mut self.route,
            ) {
                self.reason = "rota bloqueada";
                self.stuck += 0.8;
                if automatic && self.stuck >= 2.3 {
                    self.walk_blocked(f, goal, "desistiu da rota bloqueada");
                }
            }
        }
        while self
            .path
            .get(self.waypoint)
            .is_some_and(|p| p.distance(self.pos) < 0.095)
        {
            self.waypoint += 1;
        }
        let Some(&waypoint) = self.path.get(self.waypoint) else {
            return;
        };
        let delta = waypoint - self.pos;
        let wish = angle(delta);
        self.yaw = turn(self.yaw, wish, dt * 2.8);
        if forward(self.yaw).dot(delta.normalize()) < 0.75 {
            return;
        }
        let speed = match self.state {
            State::Approach => 0.36,
            State::Retreat => 0.88,
            State::Investigate | State::Follow => 0.78,
            _ => 0.55,
        } * config.speed;
        let next = self.pos + delta.clamp_length_max(speed * dt);
        if f.nav.clear_body(self.pos, next, HUMAN, f.door_closed) {
            self.moving = self.pos.distance(next) / dt.max(0.001);
            self.pos = next;
            if self.stuck > 0.0 {
                self.memory.recovered += 1;
            }
            self.stuck = 0.0;
        } else {
            self.path.clear();
            self.stuck += 0.8;
            self.reason = "replanejando após bloqueio";
            if automatic && self.stuck >= 2.3 {
                self.walk_blocked(f, goal, "desistiu de manobrar");
            }
        }
    }

    fn walk_blocked(&mut self, f: &Frame, goal: Vec2, reason: &'static str) {
        self.foot_route_until = 0.0;
        self.memory.blocked += 1;
        self.failed_target = Some(goal);
        if let Some(SearchStep::Go { link, .. }) = self.plan.current() {
            if let Some(link) = link {
                self.memory.failed(link, self.clock);
            }
            self.plan.next(self.clock);
            self.set_goal(None);
            self.stuck = 0.0;
        } else {
            let from = self.knowledge().1.map_or(self.pos, |m| m.at);
            self.retreat(f, reason, from);
        }
    }
}

pub fn player_sees(f: &Frame, point: Vec3) -> bool {
    let delta = point - f.view.eye;
    delta.length_squared() < 48.0 * 48.0
        && f.view.facing.dot(delta.normalize_or_zero()) > 0.55
        && f.nav.clear(f.view.eye, point, f.door_closed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presence::observer::nav::{Portal, Zone};
    use ainda_vivo_core::collision::Aabb;

    fn box_at(x: f32, z: f32, width: f32, depth: f32) -> Aabb {
        Aabb::from_center_size(Vec3::new(x, 1.4, z), Vec3::new(width, 2.8, depth))
    }

    fn corridor() -> Nav {
        Nav::new(
            vec![
                Zone {
                    min: Vec2::new(-4., -8.),
                    max: Vec2::new(4., 0.),
                },
                Zone {
                    min: Vec2::new(-4., 0.),
                    max: Vec2::new(4., 8.),
                },
            ],
            vec![Portal {
                rooms: [0, 1],
                at: Vec2::ZERO,
                width: 2.6,
                height: 2.7,
            }],
            vec![
                box_at(-2.65, 0., 2.7, 0.22),
                box_at(2.65, 0., 2.7, 0.22),
                box_at(0., -3.3, 1.25, 0.27),
            ],
            Some(box_at(0., 0., 2.60, 0.22)),
        )
    }

    fn frame(nav: &Nav, door_closed: bool, view: Vec2) -> Frame<'_> {
        Frame {
            nav,
            door_closed,
            view: View {
                eye: point(view).with_y(1.62),
                facing: Vec3::Z,
                protected: false,
            },
            noises: &[],
            points: &[],
        }
    }

    #[test]
    fn alternate_plan_avoids_obstacle_and_never_crosses_closed_gate() {
        let n = corridor();
        let a = Vec2::new(0., -6.);
        let b = Vec2::new(0., 6.);
        let mut path = Vec::new();
        assert!(!n.route(a, b, true, &mut path));
        assert!(n.route(a, b, false, &mut path));
        assert!(path.iter().any(|p| p.x.abs() > 1.12));
        assert!(a.distance(path[0]) < 0.9 && path.last() == Some(&b));
        assert!(
            path.windows(2)
                .all(|adj| n.clear_body(adj[0], adj[1], HUMAN, false))
        );
    }

    #[test]
    fn measured_approach_updates_its_destination_when_target_sidesteps() {
        let n = Nav::new(
            vec![Zone {
                min: Vec2::splat(-12.),
                max: Vec2::splat(12.),
            }],
            Vec::new(),
            Vec::new(),
            None,
        );
        let tuning = Tuning::default();
        let mut b = Brain::seeded(32);
        b.spawn(Vec2::new(0., -8.), 0.);
        let mut f = frame(&n, false, Vec2::new(0., 2.));
        for _ in 0..24 {
            b.step(1. / 60., &f, tuning, false);
        }
        let start = b.stance_target(b.seen.unwrap().at, tuning.approach);
        b.change(State::Approach, "teste de desvio", 7., Some(start));
        for tick in 0..120 {
            f.view.eye.x = (tick as f32 / 24.).min(4.);
            b.step(1. / 60., &f, tuning, true);
        }
        assert_eq!(b.state, State::Approach);
        assert!(
            b.goal.is_some_and(|at| at.x > start.x + 1.2),
            "meta após o desvio: {:?}",
            b.goal
        );
    }

    #[test]
    fn vision_requires_exposure_and_closed_door_preserves_stale_location() {
        let n = corridor();
        let settings = Tuning::default();
        let mut b = Brain::seeded(563);
        b.spawn(Vec2::new(0., -1.4), 0.);
        let f = frame(&n, false, Vec2::new(0., 4.));
        b.step(0.15, &f, settings, true);
        assert!(b.seen.is_none());
        b.step(0.15, &f, settings, true);
        assert_eq!(b.state, State::Watch);
        let last = b.seen.unwrap();
        let f = frame(&n, true, Vec2::new(0.5, 2.0));
        for _ in 0..8 {
            b.step(0.15, &f, settings, true);
        }
        assert!(!b.visible);
        assert_eq!(b.seen.unwrap().at, last.at);
        assert!(
            matches!(b.state, State::Follow | State::Retreat)
                || b.state == State::Absent && b.recent.contains(&State::Retreat)
        );
        for _ in 0..70 {
            b.step(0.15, &f, settings, false);
        }
        assert!(b.seen.is_none());
    }

    #[test]
    fn peripheral_motion_creates_an_uncertain_clue_and_occlusion_never_refreshes_it() {
        let nav = Nav::new(
            vec![Zone {
                min: Vec2::splat(-15.),
                max: Vec2::splat(15.),
            }],
            Vec::new(),
            Vec::new(),
            Some(box_at(3., 1.5, 0.3, 6.0)),
        );
        let config = Tuning::default();
        let mut b = Brain::seeded(42);
        b.spawn(Vec2::ZERO, 0.);
        let mut f = frame(&nav, false, Vec2::new(7., 3.));
        for _ in 0..5 {
            b.step(0.15, &f, config, false);
        }
        assert!(b.seen.is_none());
        for tick in 1..=5 {
            f.view.eye.x = 7. + tick as f32 * 0.18;
            b.step(0.15, &f, config, false);
        }
        let clue = b.seen.expect("movimento sustentado na periferia");
        assert!(clue.certainty < 0.7 && clue.velocity.x > 0.0);
        f.door_closed = true;
        f.view.eye.x = 10.0;
        for _ in 0..8 {
            b.step(0.15, &f, config, false);
        }
        assert_ne!(b.knowledge().0, Knowledge::Confirmed);
        let still = b.seen.unwrap();
        assert_eq!(still.at, clue.at);
        assert_eq!(still.when, clue.when);
        assert!(still.trust(b.clock) < clue.certainty && still.radius(b.clock) > clue.error);
    }

    #[test]
    fn sound_investigation_respects_connections_and_uses_an_approximation() {
        let n = corridor();
        let tuning = Tuning::default();
        let mut b = Brain::seeded(39);
        b.spawn(Vec2::new(0., -1.6), std::f32::consts::PI);
        let noise = [Noise {
            at: Vec2::new(0.72, 3.6),
            power: 3.0,
        }];
        let mut f = frame(&n, true, Vec2::new(0., 7.));
        f.noises = &noise;
        b.step(0.15, &f, tuning, true);
        assert!(b.heard.is_none());
        f.door_closed = false;
        b.step(0.15, &f, tuning, true);
        assert_eq!(b.state, State::Investigate);
        assert_ne!(b.heard.unwrap().at, noise[0].at);
        assert!(b.goal.is_some());
    }

    #[test]
    fn blocked_actor_gives_up_and_threat_is_stationary() {
        let n = corridor();
        let tuning = Tuning::default();
        let mut b = Brain::seeded(39);
        b.spawn(Vec2::new(0., -1.6), 0.0);
        b.direct(State::Investigate, Vec2::new(0., 4.0), tuning);
        let f = frame(&n, true, Vec2::new(0., 7.));
        for _ in 0..144 {
            b.step(1. / 60., &f, tuning, true);
        }
        assert!(
            b.state == State::Retreat
                || b.state == State::Absent && b.recent.contains(&State::Retreat)
        );
        let start = b.pos;
        let mut f = frame(&n, false, Vec2::new(0., 5.));
        f.view.facing = Vec3::NEG_Z;
        b.change(State::Threaten, "teste", 3.4, None);
        for _ in 0..84 {
            b.step(1. / 60., &f, tuning, true);
        }
        assert!(b.low());
        assert_eq!(b.pos, start);
        for _ in 0..140 {
            b.step(1. / 60., &f, tuning, true);
        }
        assert_eq!(b.state, State::Retreat);
    }

    #[test]
    fn peek_needs_a_real_edge_and_space_outside_the_wall() {
        let make = |solids| {
            Nav::new(
                vec![Zone {
                    min: Vec2::splat(-9.),
                    max: Vec2::splat(9.),
                }],
                vec![],
                solids,
                None,
            )
        };
        let empty = make(vec![]);
        let edge = make(vec![box_at(-1.21, 4.0, 1.57, 0.12)]);
        let mut b = Brain::seeded(1);
        b.spawn(Vec2::new(-1., 0.), 0.);
        let at = Vec2::new(-1., 8.);
        assert!(b.peek(&frame(&empty, false, at), at).is_none());
        assert_eq!(b.peek(&frame(&edge, false, at), at), Some(Vec2::ZERO));
    }
}
