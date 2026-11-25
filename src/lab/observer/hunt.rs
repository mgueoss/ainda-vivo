use super::{
    brain::{Brain, Frame, State, angle, forward, turn},
    nav::{HUMAN, LOW, point},
};
use crate::app::Session;
use ainda_vivo_core::psychology::{Origin, Stimulus};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Contact {
    Illusion,
    Intimidation,
    Disorientation,
    Capture,
    Quiet,
}

impl Contact {
    pub const ALL: [Self; 5] = [
        Self::Illusion,
        Self::Intimidation,
        Self::Disorientation,
        Self::Capture,
        Self::Quiet,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Illusion => "Ilusão",
            Self::Intimidation => "Intimidação",
            Self::Disorientation => "Desorientação",
            Self::Capture => "Captura silenciosa",
            Self::Quiet => "Sem efeito aparente",
        }
    }
}

pub(crate) fn select_contact(roll: f32, eligible: bool, strain: f32) -> Contact {
    match roll {
        r if eligible && strain < 76.0 && r < 0.28 => Contact::Capture,
        r if r < 0.23 => Contact::Illusion,
        r if r < 0.49 && strain < 82.0 => Contact::Disorientation,
        r if r < 0.78 && strain < 87.0 => Contact::Intimidation,
        _ => Contact::Quiet,
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Effect {
    result: Option<Contact>,
    until: f32,
}

impl Effect {
    pub fn new(result: Contact, now: f32) -> Self {
        Self {
            result: Some(result),
            until: now
                + if result == Contact::Disorientation {
                    0.62
                } else {
                    0.32
                },
        }
    }

    pub fn motion(self, now: f32) -> f32 {
        if self.result == Some(Contact::Disorientation) {
            0.15 * ((self.until - now) / 0.62).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn tape(self, now: f32) -> f32 {
        let strength = match self.result {
            Some(Contact::Illusion) => 0.17,
            Some(Contact::Disorientation) => 0.095,
            Some(Contact::Intimidation) => 0.05,
            _ => 0.0,
        };
        strength * ((self.until - now) / 0.32).clamp(0.0, 1.0)
    }
}

pub fn apply(session: &mut Session, result: Contact, psychological: bool) -> Contact {
    let result = if result == Contact::Capture && !session.expedition.capture(session.elapsed) {
        Contact::Quiet
    } else {
        result
    };
    if matches!(result, Contact::Capture | Contact::Quiet) {
        return result;
    }
    session.hunt_effect = Effect::new(result, session.elapsed);
    if psychological {
        match result {
            Contact::Intimidation => session.stimulate(Stimulus::Sighting, Origin::Presence, 0.82),
            Contact::Disorientation => session.stimulate(Stimulus::Gaze, Origin::Presence, 0.31),
            _ => {}
        }
    }
    result
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    pub speed: f32,
    pub acceleration: f32,
    pub start: f32,
    pub duration: f32,
    pub prepare: f32,
    pub reach: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            speed: 4.95,
            acceleration: 4.2,
            start: 9.0,
            duration: 7.5,
            prepare: 2.2,
            reach: 1.40,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum End {
    Contact,
    Lost,
    Route,
    Safe,
    Time,
    Bluff,
    Interrupted,
}

impl End {
    pub fn label(self) -> &'static str {
        match self {
            Self::Contact => "contato confirmado",
            Self::Lost => "perdeu o rastro",
            Self::Route => "sem rota livre",
            Self::Safe => "posto seguro",
            Self::Time => "limite da perseguição",
            Self::Bluff => "ameaça interrompida",
            Self::Interrupted => "interrompida pelo operador",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    Prepare,
    Run,
    Brake,
    Rise,
    Flinch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cue {
    Walk,
    Hurry,
    Approach,
    Brake,
    Stop,
    Rupture,
    Ready,
    Acceleration,
    Breath,
    Vocal,
    Hand,
    Foot,
    Contact,
    Retreat,
}

#[derive(Message)]
pub(crate) struct Signal {
    pub cue: Cue,
    pub at: Vec3,
    pub lab: bool,
}

#[derive(Default)]
pub struct Hunt {
    pub phase: Phase,
    pub age: f32,
    pub speed: f32,
    pub distance: f32,
    pub peak: f32,
    pub end: Option<End>,
    pub result: Option<Contact>,
    pub pending: bool,
    pub tuning: Tuning,
    pub finished: bool,
    pub guided: bool,
    pub cue: Option<Cue>,
    start_target: Vec2,
    last_signal: f32,
    next_plan: f32,
    planned: Option<Vec2>,
    blocked: f32,
    old_door: bool,
    bluff: bool,
    folding: f32,
    brake_speed: f32,
    progress: f32,
}

impl Hunt {
    pub fn start(tuning: Tuning, target: Vec2, bluff: bool) -> Self {
        Self {
            phase: Phase::Prepare,
            tuning,
            start_target: target,
            bluff,
            ..default()
        }
    }

    pub fn active(&self) -> bool {
        self.phase != Phase::Idle
    }

    pub fn low(&self) -> bool {
        matches!(self.phase, Phase::Run | Phase::Brake) || self.fold() > 0.62
    }

    fn fold(&self) -> f32 {
        match self.phase {
            Phase::Prepare => ((self.age - self.tuning.prepare * 0.33)
                / (self.tuning.prepare * 0.47))
                .clamp(0.0, 1.0),
            Phase::Rise => self.folding * (1.0 - self.age / 1.18).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    pub fn hidden(&self) -> bool {
        self.active() && self.result == Some(Contact::Illusion)
    }

    pub fn preview(result: Contact) -> Self {
        Self {
            phase: Phase::Brake,
            end: Some(End::Contact),
            result: Some(result),
            folding: 1.0,
            ..default()
        }
    }

    pub fn animation(&self, moving: f32) -> (u8, Option<f32>, f32) {
        match self.phase {
            Phase::Idle => (8, None, 1.0),
            Phase::Prepare if self.fold() == 0.0 => {
                (10, Some((self.age / self.tuning.prepare).min(0.3)), 1.0)
            }
            Phase::Prepare if self.fold() < 1.0 => (23, Some(self.fold()), 1.0),
            Phase::Prepare => (25, None, 1.0),
            Phase::Run if self.progress < 0.48 => (26, Some((self.progress / 0.88).sqrt()), 1.0),
            Phase::Run if self.speed < 1.40 => (27, None, moving / 0.99),
            Phase::Run => (28, None, moving / 2.20),
            Phase::Brake => (31, Some((self.age / 0.43).min(1.0)), 1.0),
            Phase::Rise if self.folding < 0.99 => (23, Some(self.fold()), 1.0),
            Phase::Rise => (32, Some((self.age / 1.18).min(1.0)), 1.0),
            Phase::Flinch => (34, Some((self.age / 0.49).min(1.0)), 1.0),
        }
    }

    pub fn stop(&mut self, why: End) {
        if !matches!(self.phase, Phase::Run | Phase::Prepare) {
            return;
        }
        self.end = Some(why);
        self.folding = if self.phase == Phase::Run {
            1.0
        } else {
            self.fold()
        };
        self.phase = if self.phase == Phase::Run {
            Phase::Brake
        } else {
            Phase::Rise
        };
        self.brake_speed = self.speed;
        if self.phase == Phase::Brake {
            self.cue = Some(Cue::Brake);
        }
        self.age = 0.0;
        if why == End::Contact {
            self.pending = true;
            self.brake_speed = 0.0;
            self.cue = Some(Cue::Contact);
        }
    }

    pub fn advance(&mut self, b: &mut Brain, dt: f32, f: &Frame) {
        self.age += dt;
        if f.view.protected {
            self.stop(End::Safe);
        }
        match self.phase {
            Phase::Idle => {}
            Phase::Prepare => self.prepare(b, dt, f),
            Phase::Run => self.run(b, dt, f),
            Phase::Brake => {
                let wanted = self.brake_speed * (1.0 - self.age / 0.43).clamp(0.0, 1.0);
                let next = b.pos + forward(b.yaw) * wanted * dt;
                if wanted > 0.0
                    && next.distance(f.view.eye.xz()) > self.tuning.reach + 0.35
                    && motor_clear(f, b.pos, next, b.yaw, b.yaw)
                {
                    b.moving = wanted;
                    b.pos = next;
                }
                self.speed = b.moving;
                if self.age >= 0.44 {
                    self.phase = Phase::Rise;
                    self.age = 0.0;
                    self.cue = Some(if self.end == Some(End::Contact) {
                        Cue::Retreat
                    } else {
                        Cue::Stop
                    });
                }
            }
            Phase::Rise if self.age >= 1.18 && self.result == Some(Contact::Intimidation) => {
                self.phase = Phase::Flinch;
                self.age = 0.0;
                self.cue = Some(Cue::Contact);
            }
            Phase::Rise | Phase::Flinch
                if self.age
                    >= if self.phase == Phase::Rise {
                        1.18
                    } else {
                        0.49
                    } =>
            {
                self.phase = Phase::Idle;
                self.finished = true;
                self.speed = 0.0;
                b.path.clear();
                if self.result == Some(Contact::Illusion) {
                    b.next_manifest = b.clock + 45.0;
                    b.change(State::Absent, "eco dissipado", 0.0, None);
                } else if self.end == Some(End::Lost) && b.visible {
                    b.change(State::Watch, "vigia depois de parar", 2.8, None);
                } else {
                    b.retreat(f, "encerrou a caçada", b.seen.map_or(b.pos, |m| m.at));
                }
            }
            Phase::Rise | Phase::Flinch => {}
        }
    }

    fn prepare(&mut self, b: &mut Brain, dt: f32, f: &Frame) {
        let target = b.seen.map_or(self.start_target, |m| m.at);
        let wish = angle(target - b.pos);
        let next = turn(b.yaw, wish, 1.65 * dt);
        if motor_clear(f, b.pos, b.pos, b.yaw, next) {
            b.yaw = next;
        }
        let fold = self.fold();
        if self.age - dt <= self.tuning.prepare * 0.33 && fold > 0.0 {
            self.cue = Some(Cue::Rupture);
        }
        if self.age - dt < self.tuning.prepare * 0.80 && fold >= 1.0 {
            self.cue = Some(Cue::Ready);
        }
        if self.age < self.tuning.prepare {
            return;
        }
        if self.bluff {
            self.stop(End::Bluff);
            return;
        }
        let fresh = b.seen.is_some_and(|m| m.expires - b.clock >= 7.65);
        if !fresh && !self.guided {
            self.stop(End::Lost);
            return;
        }
        self.phase = Phase::Run;
        self.age = 0.0;
        self.last_signal = b.clock;
        self.cue = Some(Cue::Acceleration);
        self.old_door = f.door_closed;
    }

    fn run(&mut self, b: &mut Brain, dt: f32, f: &Frame) {
        if self.age >= self.tuning.duration {
            self.stop(End::Time);
            return;
        }
        let seen = b.seen.filter(|m| m.expires - b.clock > 6.74);
        let heard = b.heard.filter(|m| m.expires - b.clock > 5.8);
        let fresh = seen.filter(|m| m.expires - b.clock > 8.76);
        if fresh.is_some() || heard.is_some() {
            self.last_signal = b.clock;
        }
        if b.clock - self.last_signal > 2.15
            || (seen.is_none() && heard.is_none() && self.age > 2.15)
        {
            self.stop(End::Lost);
            return;
        }
        let target = fresh
            .or(heard)
            .or(seen)
            .map(|m| m.at)
            .unwrap_or(self.start_target);
        if body_contact(f, b.pos, b.pos, b.yaw, self.tuning.reach) {
            self.stop(End::Contact);
            return;
        }
        if self.old_door != f.door_closed {
            b.path.clear();
            self.planned = None;
            self.next_plan = b.clock;
            self.old_door = f.door_closed;
        }
        if b.clock >= self.next_plan
            && (b.path.get(b.waypoint).is_none()
                || self
                    .planned
                    .is_none_or(|p| p.distance_squared(target) > 0.20))
        {
            self.next_plan = b.clock + 0.34;
            self.planned = Some(target);
            b.waypoint = 0;
            b.goal = pursuit_route(f, b, target, self.tuning.reach);
        }
        if let Some(&ahead) = b.path.get(b.waypoint)
            && ahead.distance(b.pos) < 0.15
        {
            b.waypoint += 1;
        }
        if b.path.get(b.waypoint).is_none() {
            let yaw = turn(b.yaw, angle(target - b.pos), dt * 3.8);
            if motor_clear(f, b.pos, b.pos, b.yaw, yaw) {
                b.yaw = yaw;
            }
            self.blocked += dt;
            self.speed = (self.speed - dt * 7.0).max(0.0);
            if self.blocked > 0.75 {
                self.stop(if b.pos.distance(target) < 1.1 {
                    End::Lost
                } else {
                    End::Route
                });
            }
            return;
        }
        // Salta pontos próximos apenas quando o corredor comporta a postura orientada.
        for (i, &far) in b.path.iter().enumerate().skip(b.waypoint + 1).take(4) {
            let bearing = angle(far - b.pos);
            if f.nav
                .clear_body(b.pos, far, half_at(bearing), f.door_closed)
                && f.nav.clear_body(b.pos, far, HUMAN, f.door_closed)
            {
                b.waypoint = i;
            }
        }
        let next_point = b.path[b.waypoint];
        let direction = next_point - b.pos;
        let turning = angle(direction);
        let yaw = turn(b.yaw, turning, dt * 3.8);
        let alignment = forward(b.yaw)
            .dot(direction.normalize_or_zero())
            .clamp(0.0, 1.0);
        let curve = b.path.get(b.waypoint + 1).is_some_and(|p| {
            direction.length() < 1.5
                && direction
                    .normalize_or_zero()
                    .dot((*p - next_point).normalize_or_zero())
                    < 0.64
        });
        let goal_speed = self.tuning.speed * alignment.powi(4) * if curve { 0.53 } else { 1.0 };
        self.speed += (goal_speed - self.speed).clamp(-8.0 * dt, self.tuning.acceleration * dt);
        let step = direction.clamp_length_max(self.speed * dt * alignment.powi(3));
        let next = b.pos + step;
        if motor_clear(f, b.pos, next, b.yaw, yaw) {
            b.yaw = yaw;
            b.moving = step.length() / dt.max(0.001);
            b.pos = next;
            self.progress += step.length();
            self.distance += step.length();
            self.peak = self.peak.max(b.moving);
            self.blocked = 0.0;
            if body_contact(f, next - step, next, yaw, self.tuning.reach) {
                self.stop(End::Contact);
                return;
            }
        } else {
            if motor_clear(f, b.pos, b.pos, b.yaw, yaw) {
                b.yaw = yaw;
            }
            self.blocked += dt;
            self.speed = (self.speed - dt * 8.0).max(0.0);
            if self.blocked > 0.36 && b.clock >= self.next_plan {
                b.path.clear();
            }
            if self.blocked > 0.86 {
                self.stop(End::Route);
            }
        }
        if self.age >= 0.70 && self.age - dt < 0.70 {
            self.cue = Some(Cue::Breath);
        }
        if self.age >= 1.7 && self.age - dt < 1.7 {
            self.cue = Some(Cue::Vocal);
        }
    }
}

fn body_contact(f: &Frame, from: Vec2, to: Vec2, yaw: f32, reach: f32) -> bool {
    let toward = f.view.eye.xz() - from;
    let step = to - from;
    let fraction = (toward.dot(step) / step.length_squared().max(1.0e-6)).clamp(0.0, 1.0);
    let nearest = from + fraction * step;
    let offset = f.view.eye.xz() - nearest;
    f.view.eye.y < 2.55
        && offset.length_squared() <= reach * reach
        && (offset.length_squared() < 0.0025
            || forward(yaw).dot(offset.normalize_or_zero()) >= 0.51)
        && f.nav.clear(
            point(nearest).with_y(0.72),
            f.view.eye - Vec3::Y * 0.28,
            f.door_closed,
        )
}

fn pursuit_route(f: &Frame, b: &mut Brain, target: Vec2, reach: f32) -> Option<Vec2> {
    b.path.clear();
    let feasible = |at, aim| {
        f.nav.clear_body(at, at, half_at(aim), f.door_closed)
            && f.nav.clear_body(at, at, HUMAN, f.door_closed)
    };
    let bearing = angle(target - b.pos);
    if feasible(target, bearing) {
        if f.nav.sector(b.pos) == f.nav.sector(target)
            && f.nav
                .clear_body(b.pos, target, half_at(bearing), f.door_closed)
            && f.nav.clear_body(b.pos, target, HUMAN, f.door_closed)
        {
            b.path.push(target);
            return Some(target);
        }
        if f.nav.plan(b.pos, target, f.door_closed, true, &mut b.path) {
            return Some(target);
        }
    }
    // Um operador pode se espremer onde a postura quadrúpede não cabe.
    // Termine em um ponto de aproximação com linha livre para a referência.
    let toward = (b.pos - target).normalize_or_zero();
    let candidates = [toward, Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
        .into_iter()
        .filter(|direction| direction.length_squared() > 0.01)
        .map(|direction| target + direction * (reach * 0.94))
        .filter(|&p| {
            feasible(p, angle(target - p))
                && f.nav.clear(
                    point(p).with_y(0.8),
                    point(target).with_y(1.1),
                    f.door_closed,
                )
        });
    if let Some(at) = candidates
        .clone()
        .filter(|&p| {
            f.nav.sector(b.pos) == f.nav.sector(p)
                && f.nav
                    .clear_body(b.pos, p, half_at(angle(p - b.pos)), f.door_closed)
                && f.nav.clear_body(b.pos, p, HUMAN, f.door_closed)
        })
        .min_by(|a, c| {
            a.distance_squared(b.pos)
                .total_cmp(&c.distance_squared(b.pos))
        })
    {
        b.path.push(at);
        return Some(at);
    }
    candidates
        .take(2)
        .find(|&p| f.nav.plan(b.pos, p, f.door_closed, true, &mut b.path))
}

pub fn half_at(yaw: f32) -> Vec3 {
    let (s, c) = yaw.sin_cos();
    Vec3::new(
        c.abs() * LOW.x + s.abs() * LOW.z,
        LOW.y,
        s.abs() * LOW.x + c.abs() * LOW.z,
    )
}

pub fn motor_clear(f: &Frame, a: Vec2, b: Vec2, old: f32, yaw: f32) -> bool {
    let hull = half_at(old)
        .max(half_at(yaw))
        .max(half_at((old + yaw) * 0.5));
    f.nav.clear_body(a, b, hull, f.door_closed) && f.nav.clear_body(a, b, HUMAN, f.door_closed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lab::observer::{
        brain,
        nav::{Nav, Portal, Zone},
    };
    use ainda_vivo_core::{
        collision::Aabb,
        reality::{Awakening, Reality},
        story::{Action, Phase as ProgressPhase},
    };

    fn stage(door: bool) -> Nav {
        let gate = Aabb::from_center_size(Vec3::new(0.0, 1.35, 0.0), Vec3::new(8.0, 2.7, 0.20));
        Nav::new(
            vec![
                Zone {
                    min: Vec2::new(-4., -32.),
                    max: Vec2::new(4., 0.),
                },
                Zone {
                    min: Vec2::new(-4., 0.),
                    max: Vec2::new(4., 50.),
                },
            ],
            vec![Portal {
                rooms: [0, 1],
                at: Vec2::ZERO,
                width: 8.,
                height: 2.7,
            }],
            vec![],
            door.then_some(gate),
        )
    }

    fn rehearsal(nav: &Nav, z: f32, closed: bool) -> Frame<'_> {
        Frame {
            nav,
            door_closed: closed,
            view: brain::View {
                eye: Vec3::new(0., 1.62, z),
                facing: Vec3::NEG_Z,
                protected: false,
            },
            noises: &[],
            points: &[],
        }
    }

    fn begin(target: Vec2, bluff: bool) -> Brain {
        let mut b = Brain::seeded(562);
        b.spawn(Vec2::new(0., -10.), 0.);
        b.hunt = Hunt::start(
            Tuning {
                prepare: 1.5,
                ..default()
            },
            target,
            bluff,
        );
        b
    }

    #[test]
    fn lateral_escape_along_equipment_keeps_a_reachable_chase() {
        let nav = Nav::new(
            vec![Zone {
                min: Vec2::new(-8., -15.),
                max: Vec2::new(8., 10.),
            }],
            Vec::new(),
            vec![Aabb::from_center_size(
                Vec3::new(2.70, 1.4, 3.0),
                Vec3::new(0.55, 2.8, 6.0),
            )],
            None,
        );
        let y = 2.8;
        let mut b = begin(Vec2::new(0., y), false);
        let mut x = 0.0_f32;
        let mut running = false;
        let mut pending = 0;
        for _ in 0..525 {
            running |= b.hunt.phase == Phase::Run;
            if running {
                x = (x + 4.2 / 60.0).min(2.12);
            }
            let mut f = rehearsal(&nav, y, false);
            f.view.eye.x = x;
            b.step(1. / 60., &f, default(), true);
            pending += u32::from(std::mem::take(&mut b.hunt.pending));
            if b.hunt.end.is_some() {
                break;
            }
        }
        assert!(running && x > 2.0);
        assert_eq!(
            b.hunt.end,
            Some(End::Contact),
            "a rota executável termina a uma distância de contato do alvo lateral em {:?}",
            b.pos
        );
        assert_eq!(pending, 1);
    }

    #[test]
    fn stationary_contact_has_continuous_motion_and_synced_intense_stride() {
        let nav = stage(false);
        let frame = rehearsal(&nav, 0., false);
        let mut b = begin(Vec2::ZERO, false);
        let dt = 1. / 60.;
        let mut clips = std::collections::BTreeSet::new();
        for _ in 0..460 {
            let previous = b.pos;
            b.step(dt, &frame, default(), true);
            clips.insert(b.hunt.animation(b.moving).0);
            assert!(b.pos.distance(previous) <= 5.05 * dt);
            if b.hunt.pending {
                break;
            }
        }
        assert_eq!(b.hunt.end, Some(End::Contact));
        assert!(b.hunt.peak > 4.5);
        assert!(
            [10, 23, 25, 26, 28, 31]
                .into_iter()
                .all(|c| clips.contains(&c))
        );
        b.hunt.phase = Phase::Run;
        let (_, _, scale) = b.hunt.animation(4.84);
        assert!((scale * 2.20 - 4.84).abs() < 0.001);
    }

    #[test]
    fn running_player_can_escape_and_door_can_break_the_trail() {
        let nav = stage(true);
        let mut b = begin(Vec2::new(0., -1.), false);
        let mut z = -1.0;
        for _ in 0..690 {
            if b.hunt.phase == Phase::Run {
                z += 4.5 / 60.;
            }
            b.step(1. / 60., &rehearsal(&nav, z, false), default(), true);
            if b.hunt.end.is_some() {
                break;
            }
        }
        assert_eq!(b.hunt.end, Some(End::Time));
        assert!(!b.hunt.pending);
        let mut b = begin(Vec2::new(0., 4.), false);
        for i in 0..480 {
            let closed = i >= 105;
            b.step(1. / 60., &rehearsal(&nav, 4., closed), default(), true);
            if closed {
                assert!(b.pos.y < -1.24);
            }
            if b.hunt.end.is_some() {
                break;
            }
        }
        assert!(matches!(b.hunt.end, Some(End::Lost | End::Route)));
        assert!(!b.hunt.pending);
    }

    #[test]
    fn bluff_and_safety_never_become_contacts() {
        let nav = stage(false);
        for (bluff, safe, expected) in [(true, false, End::Bluff), (false, true, End::Safe)] {
            let mut b = begin(Vec2::ZERO, bluff);
            let mut f = rehearsal(&nav, 0., false);
            f.view.protected = safe;
            let original = b.pos;
            for _ in 0..180 {
                b.step(1. / 60., &f, default(), true);
                if b.hunt.finished {
                    break;
                }
            }
            assert_eq!(b.hunt.end, Some(expected));
            assert!(b.hunt.finished && !b.hunt.pending && original == b.pos);
        }
    }

    #[test]
    fn five_outcomes_are_bounded_and_capture_keeps_the_real_checkpoint() {
        let rolls = [0.1, 0.17, 0.39, 0.58, 0.92];
        let cases = [
            Contact::Capture,
            Contact::Illusion,
            Contact::Disorientation,
            Contact::Intimidation,
            Contact::Quiet,
        ];
        for (i, contact) in cases.into_iter().enumerate() {
            let mut s = Session {
                screen: crate::app::Screen::Playing,
                ..default()
            };
            assert_eq!(select_contact(rolls[i], i == 0, 0.0), contact);
            let post = crate::scene::station::RETURN_POST;
            for action in [Action::TakeFuse, Action::InspectPump, Action::RestorePower] {
                s.expedition.act(action, post);
            }
            assert_eq!(apply(&mut s, contact, true), contact);
            assert_eq!(s.expedition.truth().phase(), ProgressPhase::Return);
            assert_eq!(s.expedition.view().phase(), ProgressPhase::Return);
            if contact == Contact::Capture {
                assert_eq!(s.expedition.observer().reality(), Reality::False);
                assert!(matches!(s.expedition.awaken(), Awakening::Recovered(_)));
                assert_eq!(apply(&mut s, Contact::Capture, true), Contact::Quiet);
            } else {
                assert_eq!(s.expedition.observer().reality(), Reality::Real);
            }
            let intensity = s.psychology.value();
            if matches!(contact, Contact::Intimidation | Contact::Disorientation) {
                assert!(intensity > 0.0 && intensity < 16.0);
            } else {
                assert_eq!(intensity, 0.0);
            }
            assert_eq!(s.hunt_effect.tape(0.75), 0.0);
            assert_eq!(s.hunt_effect.motion(0.75), 0.0);
        }
    }
}
