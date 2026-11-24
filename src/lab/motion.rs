use bevy::prelude::*;
use serde::Deserialize;

const MANIFEST: &str = include_str!("../../assets/models/movimentos.json");

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum State {
    #[default]
    Idle,
    Watch,
    Walk,
    Peek,
    Retreat,
    Rupture,
    QuadrupedIdle,
    QuadrupedSprint,
    Stop,
    Contact,
    Farewell,
}

impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Imóvel",
            Self::Watch => "Observação",
            Self::Walk => "Passada",
            Self::Peek => "Espiada",
            Self::Retreat => "Recuo",
            Self::Rupture => "Ruptura",
            Self::QuadrupedIdle => "Apoio quadrúpede",
            Self::QuadrupedSprint => "Corrida quadrúpede",
            Self::Stop => "Parada",
            Self::Contact => "Contato",
            Self::Farewell => "Despedida",
        }
    }

    pub const fn preferred_clip(self) -> u8 {
        match self {
            Self::Idle => 8,
            Self::Watch => 9,
            Self::Walk => 13,
            Self::Peek => 17,
            Self::Retreat => 18,
            Self::Rupture => 22,
            Self::QuadrupedIdle => 37,
            Self::QuadrupedSprint => 27,
            Self::Stop => 30,
            Self::Contact => 35,
            Self::Farewell => 33,
        }
    }

    pub fn blend_seconds(self, outgoing: Self) -> f32 {
        match (outgoing, self) {
            (_, Self::Contact) => 0.08,
            (Self::QuadrupedSprint, Self::QuadrupedSprint | Self::Stop) => 0.10,
            (Self::Rupture, Self::QuadrupedIdle) => 0.12,
            _ => 0.20,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Posture {
    #[default]
    Human,
    Quad,
    Fold,
    Threat,
    Rise,
}

impl Posture {
    pub fn dimensions(self, phase: f32) -> Vec3 {
        let human = Vec3::new(0.82, 2.24, 0.78);
        let quad = Vec3::new(1.18, 1.30, 2.43);
        let smooth = |t: f32| {
            let t = ((t - 0.20) / 0.72).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let weight = match self {
            Self::Human => 0.0,
            Self::Quad => 1.0,
            Self::Fold => smooth(phase),
            Self::Threat => smooth(0.62 * (1.0 - (phase * 2.0 - 1.0).abs())),
            Self::Rise => smooth(1.0 - phase),
        };
        human.lerp(quad, weight)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Limb {
    FootLeft,
    FootRight,
    HandLeft,
    HandRight,
}

impl Limb {
    pub const ALL: [Self; 4] = [
        Self::FootLeft,
        Self::FootRight,
        Self::HandLeft,
        Self::HandRight,
    ];

    pub fn joint(self) -> &'static str {
        match self {
            Self::FootLeft => "dedos_pe.L",
            Self::FootRight => "dedos_pe.R",
            Self::HandLeft => "medio.3.L",
            Self::HandRight => "medio.3.R",
        }
    }

    fn event(self) -> EventKind {
        match self {
            Self::FootLeft => EventKind::FootLeft,
            Self::FootRight => EventKind::FootRight,
            Self::HandLeft => EventKind::HandLeft,
            Self::HandRight => EventKind::HandRight,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum EventKind {
    FootLeft,
    FootRight,
    HandLeft,
    HandRight,
    RuptureStart,
    SprintStart,
    Impact,
    Contact,
    Finished,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::FootLeft => "Pé E",
            Self::FootRight => "Pé D",
            Self::HandLeft => "Mão E",
            Self::HandRight => "Mão D",
            Self::RuptureStart => "Início da ruptura",
            Self::SprintStart => "Início da corrida",
            Self::Impact => "Impacto",
            Self::Contact => "Contato",
            Self::Finished => "Fim do clipe",
        }
    }
}

#[derive(Clone, Copy, Debug, Message)]
pub(super) struct MotionEvent {
    pub actor: Entity,
    pub kind: EventKind,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Direction {
    #[default]
    Forward,
    Side,
    Back,
}

impl Direction {
    pub fn vector(self) -> Vec3 {
        match self {
            Self::Forward => Vec3::Z,
            Self::Side => Vec3::X,
            Self::Back => Vec3::NEG_Z,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct Marker {
    pub at: f32,
    pub kind: EventKind,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct Stance {
    pub start: f32,
    pub end: f32,
    limb: Limb,
    plant: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct ClipMotion {
    pub label: String,
    pub fps: f32,
    pub duration: f32,
    pub state: State,
    pub posture: Posture,
    pub repeat: bool,
    pub direction: Direction,
    speed: [f32; 2],
    markers: Vec<Marker>,
    stance: Vec<Stance>,
}

impl ClipMotion {
    pub fn load() -> Result<Vec<Self>, serde_json::Error> {
        let mut clips: Vec<Self> = serde_json::from_str(MANIFEST)?;
        for clip in &mut clips {
            clip.markers.extend(
                clip.stance
                    .iter()
                    .filter(|span| span.plant)
                    .map(|span| Marker {
                        at: span.start,
                        kind: span.limb.event(),
                    }),
            );
            clip.markers.sort_by(|a, b| a.at.total_cmp(&b.at));
        }
        Ok(clips)
    }

    pub fn legacy(label: &str) -> Self {
        let (state, posture) = match label.get(..2) {
            Some("02") => (State::Watch, Posture::Human),
            Some("03") => (State::Peek, Posture::Human),
            Some("04") => (State::QuadrupedIdle, Posture::Quad),
            Some("05" | "07") => (State::Farewell, Posture::Human),
            _ => (State::Idle, Posture::Human),
        };
        Self {
            fps: 30.0,
            state,
            posture,
            repeat: true,
            ..default()
        }
    }

    pub fn grounded(&self, limb: Limb, phase: f32) -> bool {
        if self.stance.is_empty() {
            return matches!(self.posture, Posture::Quad)
                || matches!(limb, Limb::FootLeft | Limb::FootRight);
        }
        self.stance
            .iter()
            .any(|span| span.limb == limb && (span.start..=span.end).contains(&phase))
    }

    pub fn distance(&self, time: f32, duration: f32) -> f32 {
        let [start, end] = self.speed;
        time * (start + 0.5 * (end - start) * phase(time, duration))
    }

    pub fn marks(&self) -> impl Iterator<Item = f32> + '_ {
        self.markers.iter().map(|marker| marker.at)
    }
}

pub(super) fn phase(time: f32, duration: f32) -> f32 {
    if duration > 0.0 {
        (time / duration).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(super) fn gaze_phase(actor: Transform, observer: Vec3) -> f32 {
    let local = actor.rotation.inverse() * (observer - actor.translation);
    let angle = local.x.atan2(local.z.max(0.25)).to_degrees();
    let facing = (local.z * 2.0 + 0.5).clamp(0.0, 1.0);
    ((angle.clamp(-27.0, 29.0) * facing + 27.0) / 56.0).clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Clock {
    pub time: f32,
    pub distance: f32,
}

impl Clock {
    pub fn seek(&mut self, time: f32, clip: &ClipMotion, duration: f32) -> f32 {
        let next = time.clamp(0.0, duration);
        let delta = clip.distance(next, duration) - clip.distance(self.time, duration);
        self.time = next;
        self.distance = (self.distance + delta).max(0.0);
        delta
    }

    pub fn advance(
        &mut self,
        mut dt: f32,
        clip: &ClipMotion,
        duration: f32,
        repeat: bool,
        mut emit: impl FnMut(EventKind),
    ) -> u8 {
        if duration <= 0.0 || !dt.is_finite() {
            return 0;
        }
        let mut finished: u8 = 0;
        while dt > 0.0 {
            if self.time >= duration {
                if !repeat {
                    break;
                }
                self.time = 0.0;
            }
            let step = dt.min(duration - self.time);
            let from = self.time;
            let to = (from + step).min(duration);
            self.seek(to, clip, duration);
            for marker in &clip.markers {
                let when = marker.at * duration;
                if (when > from || (from == 0.0 && when == 0.0)) && when <= to {
                    emit(marker.kind);
                }
            }
            dt -= step;
            if to >= duration {
                emit(EventKind::Finished);
                finished = finished.saturating_add(1);
            }
        }
        finished
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Sequence,
    Track,
    Approach,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sequence => "Sequência de poses",
            Self::Track => "Pista com curvas",
            Self::Approach => "Aproximação em primeira pessoa",
        }
    }

    pub fn steps(self) -> &'static [(u8, u8)] {
        match self {
            Self::Sequence => {
                const {
                    &[
                        (State::Watch.preferred_clip(), 1),
                        (11, 1),
                        (State::Peek.preferred_clip(), 1),
                        (State::Rupture.preferred_clip(), 1),
                        (State::QuadrupedIdle.preferred_clip(), 1),
                        (32, 1),
                        (State::Farewell.preferred_clip(), 1),
                    ]
                }
            }
            Self::Track => {
                const {
                    &[
                        (23, 1),
                        (25, 1),
                        (26, 1),
                        (State::QuadrupedSprint.preferred_clip(), 2),
                        (29, 2),
                        (28, 3),
                        (State::Stop.preferred_clip(), 1),
                        (37, 1),
                        (32, 1),
                    ]
                }
            }
            Self::Approach => &[(23, 1), (25, 1), (26, 1), (28, 4), (31, 1), (39, 1)],
        }
    }
}

pub(super) const APPROACH_X: f32 = -6.57;
pub(super) const APPROACH_VIEW_Z: f32 = -1.65;
const APPROACH_START_Z: f32 = -10.10;
pub(super) const TRACK_START: Vec3 = Vec3::new(APPROACH_X, 0.0, -14.3);

#[derive(Clone, Debug)]
pub(super) struct Run {
    pub mode: Mode,
    pub stage: usize,
    pub cycles: u8,
    pub distance: f32,
    pub complete: bool,
}

impl Run {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            stage: 0,
            cycles: 0,
            distance: 0.0,
            complete: false,
        }
    }

    pub fn current(&self) -> (u8, u8) {
        self.mode.steps()[self.stage]
    }

    pub fn finish_cycles(&mut self, cycles: u8, loop_run: bool) {
        self.cycles = self.cycles.saturating_add(cycles);
        if self.cycles < self.current().1 {
            return;
        }
        if self.stage + 1 < self.mode.steps().len() {
            self.stage += 1;
            self.cycles = 0;
        } else if loop_run {
            *self = Self::new(self.mode);
        } else {
            self.complete = true;
        }
    }

    pub fn translate(&mut self, delta: f32) {
        let end = match self.mode {
            Mode::Sequence => 0.0,
            Mode::Track => 13.8,
            Mode::Approach => APPROACH_VIEW_Z - APPROACH_START_Z - 1.85,
        };
        self.distance = (self.distance + delta).clamp(0.0, end);
    }

    pub fn path(&self) -> Option<(Vec3, f32)> {
        match self.mode {
            Mode::Sequence => None,
            Mode::Track => Some(track(self.distance)),
            Mode::Approach => Some((
                Vec3::new(APPROACH_X, 0.0, APPROACH_START_Z + self.distance),
                0.0,
            )),
        }
    }
}

// Quatro arcos tangentes de mesmo raio mantêm a pista na faixa livre da sala.
pub(super) fn track(distance: f32) -> (Vec3, f32) {
    const STRAIGHT: f32 = 2.45;
    const RADIUS: f32 = 4.65;
    const TURN: f32 = 0.37;
    let mut position = TRACK_START + Vec3::Z * distance.min(STRAIGHT);
    let mut remaining = (distance - STRAIGHT).max(0.0);
    let mut yaw: f32 = 0.0;
    for sign in [1.0_f32, -1.0, -1.0, 1.0] {
        let arc = remaining.min(TURN * RADIUS);
        let next = yaw + sign * arc / RADIUS;
        position.x += sign * RADIUS * (yaw.cos() - next.cos());
        position.z += sign * RADIUS * (next.sin() - yaw.sin());
        yaw = next;
        remaining -= arc;
    }
    (position + Vec3::Z * remaining, yaw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exported(number: u8) -> ClipMotion {
        ClipMotion::load()
            .unwrap()
            .into_iter()
            .find(|clip| clip.label.starts_with(&format!("{number:02} -")))
            .unwrap()
    }

    #[test]
    fn manifest_has_ordered_in_range_contacts_and_all_demo_clips() {
        let clips = ClipMotion::load().unwrap();
        assert_eq!(clips.len(), 32);
        for clip in &clips {
            assert!(clip.duration > 0.0 && clip.fps == 30.0);
            assert!(clip.markers.windows(2).all(|w| w[0].at <= w[1].at));
            assert!(clip.markers.iter().all(|m| (0.0..=1.0).contains(&m.at)));
            assert!(
                clip.stance
                    .iter()
                    .all(|s| s.start >= 0.0 && s.end <= 1.0 && s.end > s.start)
            );
        }
        for mode in [Mode::Sequence, Mode::Track, Mode::Approach] {
            for &(id, count) in mode.steps() {
                assert!(count > 0);
                assert!(
                    clips
                        .iter()
                        .any(|c| c.label.starts_with(&format!("{id:02} -")))
                );
            }
        }
    }

    #[test]
    fn clock_steps_and_speeds_agree_and_scrubbing_emits_nothing() {
        let clip = exported(28);
        let render = |frames| {
            let mut clock = Clock::default();
            let mut current = Vec::new();
            for _ in 0..frames {
                clock.advance(1.10 / frames as f32, &clip, clip.duration, true, |kind| {
                    current.push(kind)
                });
            }
            (clock, current)
        };
        let (slow, a) = render(22);
        let (fast, b) = render(132);
        assert_eq!(a, b);
        assert!((slow.distance - 2.42).abs() < 0.001);
        assert!((slow.distance - fast.distance).abs() < 0.001);
        assert_eq!(a.iter().filter(|&&e| e == EventKind::Finished).count(), 2);
        assert!(Limb::ALL.iter().all(|limb| a.contains(&limb.event())));
        let mut pose = Clock::default();
        assert!(pose.seek(1.0 / clip.fps, &clip, clip.duration) > 0.0);
        assert_eq!(pose.time, 1.0 / 30.0);
        assert!(pose.seek(-1.0, &clip, clip.duration) < 0.0);
        assert_eq!(pose.time, 0.0);
    }

    #[test]
    fn one_shot_finishes_once_and_postures_have_continuous_dimensions() {
        let clip = exported(22);
        let mut clock = Clock::default();
        let mut events = Vec::new();
        for _ in 0..3 {
            clock.advance(5.0, &clip, clip.duration, false, |kind| events.push(kind));
        }
        assert_eq!(events.last(), Some(&EventKind::Finished));
        assert_eq!(
            events.iter().filter(|&&e| e == EventKind::Finished).count(),
            1
        );
        assert_eq!(events[0], EventKind::RuptureStart);
        assert_eq!(
            Posture::Fold.dimensions(0.0),
            Posture::Human.dimensions(0.0)
        );
        assert_eq!(Posture::Fold.dimensions(1.0), Posture::Quad.dimensions(1.0));
        assert_eq!(
            Posture::Rise.dimensions(1.0),
            Posture::Human.dimensions(1.0)
        );
    }

    #[test]
    fn track_preserves_tangent_and_approach_has_clearance() {
        for i in 1..130 {
            let s = i as f32 * 0.1;
            let (p, yaw) = track(s);
            let (q, _) = track(s + 0.01);
            assert!((p.distance(q) - 0.01).abs() < 0.000_02);
            assert!(
                (q - p)
                    .normalize()
                    .dot(Quat::from_rotation_y(yaw) * Vec3::Z)
                    > 0.999
            );
            assert!((-6.7..-5.7).contains(&p.x));
        }
        let mut demo = Run::new(Mode::Approach);
        demo.translate(500.0);
        assert!((demo.path().unwrap().0.z - (APPROACH_VIEW_Z - 1.85)).abs() < 0.001);
        for &(_, cycles) in Mode::Approach.steps() {
            demo.finish_cycles(cycles, false);
        }
        assert!(demo.complete);
        assert_eq!(demo.current().0, 39);
    }

    #[test]
    fn gaze_uses_the_body_basis_and_comes_to_rest_when_observer_is_behind() {
        let actor = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
        assert!((gaze_phase(actor, Vec3::X) - 27.0 / 56.0).abs() < 0.000_001);
        assert!(gaze_phase(actor, Vec3::new(1.0, 0.0, -1.0)) > 0.95);
        assert!(gaze_phase(actor, Vec3::new(1.0, 0.0, 1.0)) < 0.05);
        assert!((gaze_phase(actor, Vec3::NEG_X) - 27.0 / 56.0).abs() < 0.000_001);
    }
}
