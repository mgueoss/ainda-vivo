//! A small record of what one observer has experienced. Mission truth lives elsewhere.
use crate::story::Action;

const SECTORS: usize = 16;
const LOOK_TIME: f32 = 0.30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Prop {
    File,
    IsolatedCircuit,
    Clock,
    ShiftTag,
    Relay,
}

#[derive(Clone, Copy, Debug, Default)]
struct Glance {
    candidate: Option<(u8, f32)>,
    known_variant: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ServiceTrail {
    #[default]
    Unknown,
    Briefed(f32),
    Footfalls(f32),
    Relay(f32),
    Checked(f32),
    Reconciled,
}

#[derive(Debug, Default)]
pub struct FieldMemory {
    visits: [u8; SECTORS],
    route: [Option<u8>; 4],
    actions: u16,
    ever_seen: u8,
    glances: [Glance; 5],
    last_step: Option<f32>,
    rhythm: [f32; 2],
    steps: u8,
    last_sighting: Option<f32>,
    pub service: ServiceTrail,
    pub desk_light: bool,
}

impl FieldMemory {
    pub fn enter(&mut self, sector: u8) {
        if usize::from(sector) >= SECTORS || self.route[0] == Some(sector) {
            return;
        }
        let count = &mut self.visits[usize::from(sector)];
        *count = count.saturating_add(1);
        self.route.rotate_right(1);
        self.route[0] = Some(sector);
    }

    pub fn visited(&self, sector: u8) -> bool {
        self.visits
            .get(usize::from(sector))
            .is_some_and(|count| *count > 0)
    }

    pub fn returned(&self, sector: u8) -> bool {
        self.visits
            .get(usize::from(sector))
            .is_some_and(|count| *count > 1)
    }

    pub fn recent_route(&self) -> &[Option<u8>; 4] {
        &self.route
    }

    pub fn read(&self, action: Action) -> bool {
        self.actions & action_bit(action) != 0
    }

    pub fn note_action(&mut self, action: Action, now: f32) {
        self.actions |= action_bit(action);
        match (action, self.service) {
            (Action::ReadServiceOrder, ServiceTrail::Unknown) => {
                self.service = ServiceTrail::Briefed(now)
            }
            (Action::CheckRelay, ServiceTrail::Relay(_)) => {
                self.service = ServiceTrail::Checked(now)
            }
            (Action::ToggleDeskLight, _) => self.desk_light = !self.desk_light,
            _ => (),
        }
    }

    /// A brief glance can be interrupted. Only a sustained, unobstructed look
    /// remembers the current variant. A previous observation is retained.
    pub fn look(&mut self, prop: Prop, variant: u8, now: f32, visible: bool) {
        let glance = &mut self.glances[prop as usize];
        if !visible {
            glance.candidate = None;
            return;
        }
        let since = match glance.candidate {
            Some((last, since)) if last == variant => since,
            _ => {
                glance.candidate = Some((variant, now));
                now
            }
        };
        if now - since >= LOOK_TIME {
            glance.known_variant = Some(variant);
            self.ever_seen |= 1 << prop as u8;
        }
    }

    pub fn knows_variant(&self, prop: Prop, variant: u8) -> bool {
        self.glances[prop as usize].known_variant == Some(variant)
    }

    pub fn has_seen(&self, prop: Prop) -> bool {
        self.ever_seen & (1 << prop as u8) != 0
    }

    pub fn reset_variants(&mut self) {
        self.glances = [Glance::default(); 5];
        self.ever_seen = 0;
    }

    pub fn note_sighting(&mut self, now: f32) {
        self.last_sighting = Some(now);
    }

    pub fn recent_sighting(&self, now: f32) -> bool {
        self.last_sighting.is_some_and(|at| now - at < 8.0)
    }

    pub fn footstep(&mut self, now: f32) {
        if let Some(previous) = self.last_step.replace(now) {
            let interval = now - previous;
            if (0.23..=1.10).contains(&interval) {
                self.rhythm.rotate_left(1);
                self.rhythm[1] = interval;
                self.steps = self.steps.saturating_add(1);
            }
        }
    }

    pub fn echo_pattern(&self, now: f32, distorted: bool) -> Option<[f32; 2]> {
        let delay = if distorted { 5.0 } else { 7.0 };
        (matches!(self.service, ServiceTrail::Briefed(at) if now - at >= delay) && self.steps >= 2)
            .then_some(self.rhythm)
    }

    pub fn heard_echo(&mut self, now: f32) {
        debug_assert!(matches!(self.service, ServiceTrail::Briefed(_)));
        self.service = ServiceTrail::Footfalls(now);
    }

    pub fn relay_due(&self, now: f32, distorted: bool) -> bool {
        let delay = if distorted { 6.5 } else { 9.0 };
        self.knows_variant(Prop::Relay, 0)
            && matches!(self.service, ServiceTrail::Footfalls(at) if now - at >= delay)
    }

    pub fn changed_relay(&mut self, now: f32) {
        debug_assert!(matches!(self.service, ServiceTrail::Footfalls(_)));
        self.service = ServiceTrail::Relay(now);
    }

    pub fn reconcile(&mut self, now: f32) -> bool {
        if matches!(self.service, ServiceTrail::Checked(at) if now - at >= 8.0) {
            self.service = ServiceTrail::Reconciled;
        }
        self.service == ServiceTrail::Reconciled
    }
}

fn action_bit(action: Action) -> u16 {
    1 << action as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_only_sustained_observation_and_keeps_the_variant_until_seen_again() {
        let mut memory = FieldMemory::default();
        memory.look(Prop::File, 0, 1.0, true);
        memory.look(Prop::File, 0, 1.1, false);
        memory.look(Prop::File, 0, 1.9, true);
        assert!(!memory.knows_variant(Prop::File, 0));
        memory.look(Prop::File, 0, 2.23, true);
        assert!(memory.knows_variant(Prop::File, 0));
        assert!(!memory.knows_variant(Prop::File, 1));
        memory.look(Prop::File, 1, 3.0, true);
        memory.look(Prop::File, 1, 3.35, true);
        assert!(memory.knows_variant(Prop::File, 1));
    }

    #[test]
    fn visiting_is_per_observer_and_a_continuous_stay_is_not_a_return() {
        let mut memory = FieldMemory::default();
        memory.enter(3);
        memory.enter(3);
        assert!(!memory.returned(3));
        memory.enter(4);
        memory.enter(3);
        assert!(memory.returned(3));
        assert!(memory.visited(4));
        assert!(!FieldMemory::default().visited(3));
        assert_eq!(memory.recent_route(), &[Some(3), Some(4), Some(3), None]);
    }

    #[test]
    fn evidence_requires_the_preceding_cue_original_observation_and_an_interval() {
        let mut memory = FieldMemory::default();
        memory.note_action(Action::CheckRelay, 0.0);
        assert!(!memory.reconcile(150.0));
        memory.note_action(Action::ReadServiceOrder, 3.0);
        memory.footstep(4.0);
        memory.footstep(4.4);
        memory.footstep(4.9);
        assert!(memory.echo_pattern(9.9, false).is_none());
        let pattern = memory.echo_pattern(10.0, false).unwrap();
        assert!((pattern[0] - 0.4).abs() < 0.01 && (pattern[1] - 0.5).abs() < 0.01);
        memory.heard_echo(10.0);
        assert!(!memory.relay_due(23.0, false));
        memory.look(Prop::Relay, 0, 11.0, true);
        memory.look(Prop::Relay, 0, 11.5, true);
        assert!(!memory.relay_due(18.9, false));
        assert!(memory.relay_due(19.0, false));
        memory.changed_relay(19.0);
        assert!(!memory.reconcile(35.0));
        memory.note_action(Action::CheckRelay, 20.0);
        assert!(!memory.reconcile(27.0));
        assert!(memory.reconcile(28.0));
        assert!(memory.echo_pattern(60.0, true).is_none());
    }
}
