use super::hunt::Hunt;
use super::nav::{HUMAN, LOW, Nav, point};
use bevy::prelude::*;

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

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct Memory {
    pub at: Vec2,
    pub expires: f32,
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
    rng: u32,
    sample: f32,
    exposure: f32,
    pub(super) visible: bool,
    noticed: f32,
    limit: f32,
    retry_at: f32,
    stuck: f32,
    old_door: bool,
    failed_target: Option<Vec2>,
    rest_at: Vec2,
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
    }

    fn sample(&mut self, f: &Frame, config: Tuning, elapsed: f32) {
        let eye = self.eye();
        let target = f.view.eye - Vec3::Y * 0.10;
        let delta = target - eye;
        let heading = forward(self.yaw + self.head);
        let cone = (config.fov.to_radians() * 0.5).cos();
        let frontal = heading.dot(delta.xz().normalize_or_zero()) >= cone;
        let in_range = delta.length_squared() <= config.range * config.range && delta.y.abs() < 4.0;
        let clear = frontal && in_range && f.nav.clear(eye, target, f.door_closed);
        self.ray = (frontal || in_range).then_some((eye, target, clear));
        self.exposure = if clear {
            (self.exposure + elapsed).min(0.5)
        } else {
            0.0
        };
        self.visible = clear && self.exposure >= 0.25;
        if self.visible {
            self.seen = Some(Memory {
                at: target.xz(),
                expires: self.clock + 9.0,
            });
            self.hit_sector = f.nav.sector(target.xz());
        }
        let in_player_view = player_sees(f, eye);
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
                if self.heard.is_some_and(|old| {
                    old.at.distance_squared(at) < 1.0 && old.expires - self.clock > 5.0
                }) {
                    continue;
                }
                loudest = power;
                self.heard = Some(Memory {
                    at,
                    expires: self.clock + 8.0,
                });
            }
        }
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
    }

    fn decide(&mut self, f: &Frame, config: Tuning) {
        let ended = self.clock >= self.limit;
        let known = self.seen.or(self.heard).map_or(self.rest_at, |m| m.at);
        let arrived = self.goal.is_none_or(|p| p.distance(self.pos) < 0.23);
        if self.state == State::Retreat {
            if (ended || arrived) && !player_sees(f, self.eye()) {
                let delay = 24.0 + 25.0 * self.random();
                self.next_manifest = self.clock + delay;
                self.change(State::Absent, "recuou fora de vista", 0.0, None);
            } else if (ended || arrived) && self.clock >= self.retry_at {
                self.retreat(f, "busca outro abrigo", known);
                self.retry_at = self.clock + 2.0;
            }
            return;
        }
        if self.state == State::Threaten {
            if ended {
                self.retreat(f, "ameaça encerrada", known);
            }
            return;
        }
        if self.clock - self.born > 33.0 || self.seen.is_some_and(|m| m.at.distance(self.pos) < 2.8)
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
                }
            }
            return;
        }
        if self.state == State::Peek {
            if self.noticed > 0.40 || ended {
                self.retreat(f, "encerra a espiada", known);
            }
            return;
        }
        if self.visible {
            if self.state == State::Approach && (arrived || ended) {
                if known.distance(self.pos) < config.approach + 2.0 && self.can_threaten(f) {
                    self.change(State::Threaten, "ameaça sem avanço", 3.4, None);
                } else {
                    self.retreat(f, "mantém distância", known);
                }
            } else if self.state == State::Watch && (ended || self.noticed > 2.5) {
                let roll = self.random();
                if let Some(target) = self
                    .peek(f, known)
                    .filter(|_| roll < 0.28 && !self.recent.contains(&State::Peek))
                {
                    self.change(
                        State::Peek,
                        "achou uma borda com cobertura",
                        5.5,
                        Some(target),
                    );
                } else if roll > 0.66
                    && known.distance(self.pos) > config.approach + 1.3
                    && !self.recent.contains(&State::Approach)
                {
                    let target = self.stance_target(known, config.approach);
                    self.change(State::Approach, "aproximação medida", 7.0, Some(target));
                } else {
                    self.retreat(f, "observação encerrada", known);
                }
            } else if !matches!(self.state, State::Watch | State::Approach) {
                let duration = config.watching * (0.75 + self.random() * 0.40);
                self.change(State::Watch, "contato visual confirmado", duration, None);
            }
            return;
        }
        if matches!(self.state, State::Watch | State::Approach) && self.clock - self.entered > 0.7 {
            if let Some(m) = self.seen
                && m.at.distance(self.pos) > 4.0
                && !self.recent.contains(&State::Follow)
                && self.random() < 0.72
            {
                self.change(
                    State::Follow,
                    "perdeu a visão, segue a última referência",
                    7.0,
                    Some(self.stance_target(m.at, 3.6)),
                );
            } else {
                self.retreat(f, "perdeu a visão", known);
            }
        } else if let Some(noise) = self.heard
            && matches!(self.state, State::Rest | State::Explore | State::Follow)
            && !self.recent.contains(&State::Investigate)
        {
            self.change(
                State::Investigate,
                "ruído localizado aproximadamente",
                config.investigation,
                Some(self.stance_target(noise.at, 0.9)),
            );
        } else if ended || (arrived && self.goal.is_some()) {
            if matches!(
                self.state,
                State::Follow | State::Investigate | State::Explore
            ) {
                self.change(State::Rest, "escuta no local alcançado", 2.2, None);
            } else if self.recent.contains(&State::Explore) {
                self.retreat(f, "fim da ronda", known);
            } else if let Some(target) = self.pocket(f, known) {
                self.change(State::Explore, "ronda exploratória", 13.0, Some(target));
            } else {
                self.retreat(f, "sem ponto acessível", known);
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
        if self.sample >= 0.14 {
            let elapsed = std::mem::take(&mut self.sample);
            self.sample(f, config, elapsed);
            if automatic && !self.hunt.active() {
                self.decide(f, config);
            }
        }
        if self.hunt.active() {
            let mut hunt = std::mem::take(&mut self.hunt);
            self.head = turn(self.head, 0.0, dt * 2.0);
            hunt.advance(self, dt, f);
            self.hunt = hunt;
            return;
        }
        let target = self.seen.or(self.heard).map(|m| m.at);
        if matches!(self.state, State::Watch | State::Peek | State::Threaten)
            && let Some(at) = target
        {
            let yaw = turn(self.yaw, angle(at - self.pos), 0.49);
            self.head = turn(self.head, yaw - self.yaw, dt * 1.4);
            if self.state != State::Watch && self.goal.is_none() {
                let next = turn(self.yaw, angle(at - self.pos), dt * 1.2);
                if f.nav.clear_body(self.pos, self.pos, HUMAN, f.door_closed) {
                    self.yaw = next;
                }
            }
        } else {
            self.head = turn(self.head, 0.0, dt);
        }
        self.walk(dt, f, config, automatic);
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
            if !f.nav.route(self.pos, goal, f.door_closed, &mut self.path) {
                self.reason = "rota bloqueada";
                self.stuck += 0.8;
                if automatic && self.stuck >= 2.3 {
                    self.failed_target = Some(goal);
                    let from = self.seen.map_or(self.pos, |m| m.at);
                    self.retreat(f, "desistiu da rota bloqueada", from);
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
            self.stuck = 0.0;
        } else {
            self.path.clear();
            self.stuck += 0.8;
            self.reason = "replanejando após bloqueio";
            if automatic && self.stuck >= 2.3 {
                self.failed_target = Some(goal);
                let from = self.seen.map_or(self.pos, |m| m.at);
                self.retreat(f, "desistiu de manobrar", from);
            }
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
    use crate::lab::observer::nav::{Portal, Zone};
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
