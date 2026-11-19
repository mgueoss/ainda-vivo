//! Authoritative expedition rules and one observer's continuity.
use crate::{
    Vec3,
    story::{Action, Phase, Progress},
};

/// Stable within the expedition. The desktop app currently owns one observer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObserverId(pub u32);

impl ObserverId {
    pub const LOCAL: Self = Self(0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reality {
    Real,
    False,
    Eliminated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Escaped,
    PerceivedExit,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SafePoint {
    pub feet: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Awakening {
    Recovered(SafePoint),
    Interrupted,
    Unavailable,
}

#[derive(Debug)]
enum Participation {
    Real,
    False { view: Progress, since: f32 },
    Eliminated,
}

#[derive(Debug)]
pub struct Observer {
    pub id: ObserverId,
    participation: Participation,
    checkpoint: Option<SafePoint>,
    recovered: bool,
    outcome: Option<Outcome>,
}

impl Observer {
    pub fn reality(&self) -> Reality {
        match self.participation {
            Participation::Real => Reality::Real,
            Participation::False { .. } => Reality::False,
            Participation::Eliminated => Reality::Eliminated,
        }
    }

    pub fn distortion_ready(&self, now: f32) -> bool {
        matches!(self.participation, Participation::False { since, .. } if now >= since + 4.0)
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }
}

#[derive(Debug)]
pub struct Expedition {
    truth: Progress,
    observer: Observer,
}

impl Default for Expedition {
    fn default() -> Self {
        Self::new(ObserverId::LOCAL)
    }
}

impl Expedition {
    pub fn new(id: ObserverId) -> Self {
        Self {
            truth: Progress::default(),
            observer: Observer {
                id,
                participation: Participation::Real,
                checkpoint: None,
                recovered: false,
                outcome: None,
            },
        }
    }

    pub fn truth(&self) -> &Progress {
        &self.truth
    }

    pub fn observer(&self) -> &Observer {
        &self.observer
    }

    /// All interactive presentation, including the local gate, reads this view.
    pub fn view(&self) -> &Progress {
        match &self.observer.participation {
            Participation::False { view, .. } => view,
            _ => &self.truth,
        }
    }

    pub fn act(&mut self, action: Action, pose: SafePoint) -> &'static str {
        match &mut self.observer.participation {
            Participation::Real => {
                let before = self.truth.phase();
                let message = self.truth.apply(action);
                if before == Phase::Electrical && self.truth.phase() == Phase::Return {
                    self.observer.checkpoint = Some(pose);
                }
                message
            }
            Participation::False { view, .. } => view.apply(action),
            Participation::Eliminated => "",
        }
    }

    pub fn can_capture(&self) -> bool {
        self.observer.reality() == Reality::Real
            && self.observer.checkpoint.is_some()
            && self.observer.outcome.is_none()
            && !self.observer.recovered
    }

    /// Leaves the body, clock, audio and currently visible view untouched.
    pub fn capture(&mut self, now: f32) -> bool {
        if !self.can_capture() {
            return false;
        }
        self.observer.participation = Participation::False {
            view: self.truth.clone(),
            since: now,
        };
        true
    }

    pub fn cross_exit(&mut self) -> Option<Outcome> {
        if !self.view().exit_unlocked() || self.observer.outcome.is_some() {
            return None;
        }
        let outcome = match self.observer.reality() {
            Reality::Real => Outcome::Escaped,
            Reality::False => Outcome::PerceivedExit,
            Reality::Eliminated => return None,
        };
        self.observer.outcome = Some(outcome);
        Some(outcome)
    }

    pub fn awaken(&mut self) -> Awakening {
        match self.observer.reality() {
            Reality::False => {
                let checkpoint = self
                    .observer
                    .checkpoint
                    .expect("capture requires a checkpoint");
                self.observer.participation = Participation::Real;
                self.observer.recovered = true;
                self.observer.outcome = None;
                Awakening::Recovered(checkpoint)
            }
            Reality::Real if self.observer.outcome.is_none() => {
                self.observer.participation = Participation::Eliminated;
                self.observer.outcome = Some(Outcome::Interrupted);
                Awakening::Interrupted
            }
            _ => Awakening::Unavailable,
        }
    }
}

pub const CONTACT_RADIUS: f32 = 4.4;
pub const CONTACT_LIFETIME: f32 = 24.0;
const WARNING_WINDOW: f32 = 6.0;
const EXPOSURE: f32 = 2.15;

/// The authored contact zone needs a witnessed mark and a clear approach.
/// Once warned, stepping out of reach resolves this encounter.
#[derive(Clone, Copy, Debug, Default)]
pub enum Approach {
    #[default]
    Waiting,
    Warned {
        since: f32,
        exposed_since: Option<f32>,
    },
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactEvent {
    None,
    Warning,
    Capture,
}

impl Approach {
    pub fn active(self) -> bool {
        matches!(self, Self::Warned { .. })
    }

    pub fn step(&mut self, now: f32, within_reach: bool, exposed: bool) -> ContactEvent {
        match self {
            Self::Waiting if within_reach => {
                *self = Self::Warned {
                    since: now,
                    exposed_since: None,
                };
                ContactEvent::Warning
            }
            Self::Warned {
                since,
                exposed_since,
            } => {
                if !within_reach || now - *since >= WARNING_WINDOW {
                    *self = Self::Closed;
                } else if exposed {
                    let start = *exposed_since.get_or_insert(now);
                    if now - start >= EXPOSURE && now - *since >= 2.4 {
                        *self = Self::Closed;
                        return ContactEvent::Capture;
                    }
                } else {
                    *exposed_since = None;
                }
                ContactEvent::None
            }
            _ => ContactEvent::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAFE: SafePoint = SafePoint {
        feet: Vec3::Y,
        yaw: 0.4,
        pitch: 0.0,
    };

    fn powered() -> Expedition {
        let mut expedition = Expedition::new(ObserverId(7));
        for action in [Action::TakeFuse, Action::InspectPump, Action::RestorePower] {
            expedition.act(action, SAFE);
        }
        expedition
    }

    #[test]
    fn a_capture_forks_the_visible_mission_only_after_the_real_rearm() {
        assert!(!Expedition::default().capture(12.0));
        let mut expedition = powered();
        assert_eq!(expedition.observer().id, ObserverId(7));
        assert!(expedition.capture(24.0));
        assert_eq!(expedition.observer().reality(), Reality::False);
        assert_eq!(expedition.view().phase(), Phase::Return);
        assert!(!expedition.observer().distortion_ready(27.9));
        assert!(expedition.observer().distortion_ready(28.0));
        expedition.act(Action::InspectPump, SAFE);
        expedition.act(Action::ReviewRecording, SAFE);
        assert!(expedition.view().exit_unlocked() && expedition.view().recording_reviewed());
        assert!(!expedition.truth().exit_unlocked() && !expedition.truth().recording_reviewed());
        assert!(!expedition.capture(40.0));
    }

    #[test]
    fn exits_and_awakening_are_idempotent_and_preserve_real_work() {
        let mut expedition = powered();
        assert_eq!(expedition.cross_exit(), None);
        assert!(expedition.capture(30.0));
        expedition.act(Action::InspectPump, SAFE);
        assert_eq!(expedition.cross_exit(), Some(Outcome::PerceivedExit));
        assert_eq!(expedition.cross_exit(), None);
        assert_eq!(expedition.awaken(), Awakening::Recovered(SAFE));
        assert_eq!(expedition.truth().phase(), Phase::Return);
        assert_eq!(expedition.view().phase(), Phase::Return);
        assert_eq!(expedition.observer().outcome(), None);
        assert!(!expedition.capture(60.0));
        expedition.act(Action::InspectPump, SAFE);
        assert_eq!(expedition.cross_exit(), Some(Outcome::Escaped));
        assert_eq!(expedition.awaken(), Awakening::Unavailable);
    }

    #[test]
    fn awakening_while_alive_ends_the_attempt() {
        let mut expedition = powered();
        assert_eq!(expedition.awaken(), Awakening::Interrupted);
        assert_eq!(expedition.observer().reality(), Reality::Eliminated);
        assert_eq!(expedition.observer().outcome(), Some(Outcome::Interrupted));
        assert_eq!(expedition.act(Action::InspectPump, SAFE), "");
        assert_eq!(expedition.view().phase(), Phase::Return);
        assert_eq!(expedition.awaken(), Awakening::Unavailable);
        assert!(powered().can_capture());
    }

    #[test]
    fn a_recorded_real_gate_stays_open_after_recovery() {
        let mut expedition = powered();
        expedition.act(Action::InspectPump, SAFE);
        assert!(expedition.capture(31.0));
        assert_eq!(expedition.cross_exit(), Some(Outcome::PerceivedExit));
        assert_eq!(expedition.awaken(), Awakening::Recovered(SAFE));
        assert!(expedition.view().exit_unlocked());
        assert_eq!(expedition.cross_exit(), Some(Outcome::Escaped));
    }

    #[test]
    fn sustained_exposure_after_a_warning_can_capture() {
        let mut approach = Approach::Waiting;
        assert_eq!(approach.step(30.0, false, true), ContactEvent::None);
        assert_eq!(approach.step(31.0, true, true), ContactEvent::Warning);
        assert_eq!(approach.step(31.2, true, true), ContactEvent::None);
        assert_eq!(approach.step(33.2, true, true), ContactEvent::None);
        assert_eq!(approach.step(33.5, true, true), ContactEvent::Capture);
        assert_eq!(approach.step(34.0, true, true), ContactEvent::None);
    }

    #[test]
    fn turning_back_resets_exposure_and_retreat_closes_the_encounter() {
        let mut approach = Approach::Waiting;
        assert_eq!(approach.step(10.0, true, true), ContactEvent::Warning);
        approach.step(10.1, true, true);
        approach.step(12.0, true, false);
        assert_eq!(approach.step(12.2, true, true), ContactEvent::None);
        assert_eq!(approach.step(12.8, false, true), ContactEvent::None);
        assert!(!approach.active());
        assert_eq!(approach.step(15.0, true, true), ContactEvent::None);
        let mut waiting = Approach::Waiting;
        waiting.step(10.0, true, false);
        assert_eq!(waiting.step(16.1, true, false), ContactEvent::None);
        assert!(!waiting.active());
    }
}
