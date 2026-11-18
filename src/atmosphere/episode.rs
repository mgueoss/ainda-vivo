use crate::scene::station::PUMP_02_PANEL;
use ainda_vivo_core::story::Phase;
use bevy::prelude::*;

use super::smooth;

pub const PUMP_KNOCK: Vec3 = Vec3::new(6.75, 1.44, -43.12);
const PUMP_BEFORE_KNOCK: f32 = 1.35;
const RAIL_BEFORE_SCRAPE: f32 = 0.80;

#[derive(Default)]
struct Beat {
    start: Option<f32>,
    sounded: bool,
}

impl Beat {
    fn age(&mut self, elapsed: f32, eligible: bool) -> Option<f32> {
        if eligible && self.start.is_none() {
            self.start = Some(elapsed);
        }
        self.start.map(|start| (elapsed - start).max(0.0))
    }

    fn make_sound(&mut self, age: f32, onset: f32) -> bool {
        if !self.sounded && age >= onset {
            self.sounded = true;
            return true;
        }
        false
    }
}

#[derive(Clone, Copy)]
pub struct RailTrace {
    pub age: f32,
    side: f32,
}

impl RailTrace {
    pub fn sound_position(self) -> Vec3 {
        Vec3::new(self.side * 4.36, 0.84, -73.8)
    }

    pub fn water_position(self) -> Vec3 {
        Vec3::new(self.side * 2.56, -0.505, -73.8)
    }
}

#[derive(Default)]
pub struct Episode {
    pump: Beat,
    rail: Beat,
    far_side: f32,
    pub(crate) pressure: f32,
    pub(crate) pump_hush: f32,
    pub(crate) rail_hush: f32,
    pub(crate) trace: Option<RailTrace>,
    pub(crate) play_pump: bool,
    pub(crate) play_rail: bool,
}

impl Episode {
    pub fn update(&mut self, elapsed: f32, feet: Vec3, phase: Phase) {
        self.play_pump = false;
        self.play_rail = false;
        let in_nave = feet.x.abs() < 13.68
            && feet.xz().distance_squared(PUMP_02_PANEL.xz()) < 9.4_f32.powi(2);
        if let Some(age) = self.pump.age(elapsed, in_nave && phase >= Phase::Return) {
            let rising = smooth((age - 0.27) / 1.53);
            let tremor = (age * 13.0).sin() * 0.095 * smooth((4.7 - age) / 2.2);
            self.pressure = rising * (0.92 + tremor).clamp(0.0, 1.0);
            self.pump_hush = smooth(age / 0.86) * smooth((7.55 - age) / 2.8);
            self.play_pump = self.pump.make_sound(age, PUMP_BEFORE_KNOCK);
        }

        let on_walkway =
            (4.65..=11.82).contains(&feet.x.abs()) && (-77.9..=-67.4).contains(&feet.z);
        let start = phase == Phase::Exit && on_walkway;
        if start && self.rail.start.is_none() {
            self.far_side = -feet.x.signum();
        }
        if let Some(age) = self.rail.age(elapsed, start) {
            self.trace = (age < 4.7).then_some(RailTrace {
                age,
                side: self.far_side,
            });
            self.rail_hush = smooth(age / 0.64) * smooth((4.9 - age) / 2.55);
            self.play_rail = self.rail.make_sound(age, RAIL_BEFORE_SCRAPE);
        }
    }

    pub fn holding_silence(&self) -> bool {
        self.pump_hush > 0.16 || self.rail_hush > 0.16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pump_waits_for_rearm_and_plays_once_during_the_return() {
        let near = PUMP_02_PANEL.with_y(0.0) - Vec3::X * 2.0;
        let mut episode = Episode::default();
        episode.update(3.0, near, Phase::Baseline);
        episode.update(10.0, Vec3::new(20.0, 0.0, -56.0), Phase::Return);
        assert_eq!(episode.pressure, 0.0);
        episode.update(14.0, near, Phase::Return);
        episode.update(15.0, near, Phase::Return);
        assert!(episode.pump_hush > 0.9 && !episode.play_pump);
        episode.update(15.5, near, Phase::Return);
        assert!(episode.play_pump && episode.pressure > 0.60);
        episode.update(18.0, near, Phase::Exit);
        assert!(!episode.play_pump && episode.pressure > 0.80);
        episode.update(24.0, near, Phase::Exit);
        assert!(!episode.holding_silence() && !episode.play_pump);
    }

    #[test]
    fn the_basin_cue_uses_the_opposite_side_and_ignores_a_locked_gate() {
        let east = Vec3::new(7.0, 0.0, -70.0);
        let mut episode = Episode::default();
        episode.update(30.0, east, Phase::Return);
        assert!(episode.trace.is_none());
        episode.update(31.0, east, Phase::Exit);
        episode.update(32.0, east, Phase::Exit);
        assert!(episode.play_rail);
        assert!(episode.trace.unwrap().sound_position().x < 0.0);
        episode.update(35.95, east.with_x(-7.0), Phase::Exit);
        assert!(episode.trace.is_none() && !episode.play_rail);
    }
}
