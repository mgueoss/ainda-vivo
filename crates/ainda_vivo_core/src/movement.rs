use crate::{
    Vec2, Vec3,
    collision::{Aabb, body_bounds, move_and_slide},
};

pub const STANDING_HEIGHT: f32 = 1.78;
pub const CROUCH_HEIGHT: f32 = 1.05;
pub const SPAWN: Vec3 = Vec3::new(0.0, 0.01, 8.0);

/// This is the future input boundary for touchscreen and LAN commands.
/// yaw is in radians; movement.y is forward. No hardware key codes here.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerCommand {
    pub movement: Vec2,
    pub yaw: f32,
    pub sprint: bool,
    pub crouch: bool,
    pub jump: bool,
}

#[derive(Clone, Debug)]
pub struct PlayerBody {
    pub feet: Vec3,
    pub previous_feet: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
    pub crouched: bool,
    pub contacts: u32,
}

impl Default for PlayerBody {
    fn default() -> Self {
        Self {
            feet: SPAWN,
            previous_feet: SPAWN,
            velocity: Vec3::ZERO,
            grounded: false,
            crouched: false,
            contacts: 0,
        }
    }
}

impl PlayerBody {
    pub fn height(&self) -> f32 {
        if self.crouched {
            CROUCH_HEIGHT
        } else {
            STANDING_HEIGHT
        }
    }

    pub fn step(&mut self, command: PlayerCommand, dt: f32, solids: &[Aabb]) {
        self.previous_feet = self.feet;
        self.crouched = command.crouch
            || (self.crouched
                && solids
                    .iter()
                    .any(|s| body_bounds(self.feet, STANDING_HEIGHT).overlaps(*s)));
        let speed = if self.crouched {
            1.25
        } else if command.sprint {
            4.5
        } else {
            2.5
        };
        let input = command.movement.clamp_length_max(1.0);
        let right = Vec3::new(command.yaw.cos(), 0.0, -command.yaw.sin());
        let forward = Vec3::new(-command.yaw.sin(), 0.0, -command.yaw.cos());
        let target = (right * input.x + forward * input.y) * speed;
        let blend = 1.0 - (-14.0 * dt).exp();
        self.velocity.x += (target.x - self.velocity.x) * blend;
        self.velocity.z += (target.z - self.velocity.z) * blend;
        if command.jump && self.grounded && !self.crouched {
            self.velocity.y = 4.8;
        }
        self.velocity.y = (self.velocity.y - 16.0 * dt).max(-40.0);
        let result = move_and_slide(self.feet, self.height(), self.velocity, dt, solids);
        self.feet = result.feet;
        self.velocity = result.velocity;
        self.grounded = result.grounded;
        self.contacts = result.contacts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn floor() -> Aabb {
        Aabb::from_center_size(Vec3::new(0.0, -0.5, 0.0), Vec3::new(100.0, 1.0, 100.0))
    }

    #[test]
    fn gravity_settles_without_sinking_or_jitter() {
        let mut p = PlayerBody::default();
        p.feet.y = 3.0;
        for _ in 0..600 {
            p.step(PlayerCommand::default(), 1.0 / 60.0, &[floor()]);
        }
        assert!(p.grounded && p.feet.y >= 0.0 && p.feet.y < 0.003);
        assert_eq!(p.velocity.y, 0.0);
    }

    #[test]
    fn crouched_player_cannot_stand_inside_low_beam() {
        let beam = Aabb::from_center_size(Vec3::new(0.0, 1.5, 8.0), Vec3::new(4.0, 0.4, 4.0));
        let mut p = PlayerBody {
            crouched: true,
            ..Default::default()
        };
        p.step(PlayerCommand::default(), 1.0 / 60.0, &[floor(), beam]);
        assert!(p.crouched);
        p.feet.x = 5.0;
        p.step(PlayerCommand::default(), 1.0 / 60.0, &[floor(), beam]);
        assert!(!p.crouched);
    }

    #[test]
    fn diagonal_input_has_same_speed_as_forward() {
        let mut p = PlayerBody::default();
        let mut q = p.clone();
        for _ in 0..180 {
            p.step(
                PlayerCommand {
                    movement: Vec2::ONE,
                    ..Default::default()
                },
                1.0 / 60.0,
                &[floor()],
            );
            q.step(
                PlayerCommand {
                    movement: Vec2::Y,
                    ..Default::default()
                },
                1.0 / 60.0,
                &[floor()],
            );
        }
        assert!((p.velocity.length() - q.velocity.length()).abs() < 0.001);
    }

    #[test]
    fn jump_lands_and_held_command_cannot_fly() {
        let mut p = PlayerBody::default();
        for _ in 0..3 {
            p.step(PlayerCommand::default(), 1.0 / 60.0, &[floor()]);
        }
        p.step(
            PlayerCommand {
                jump: true,
                ..Default::default()
            },
            1.0 / 60.0,
            &[floor()],
        );
        assert!(p.feet.y > 0.01 && !p.grounded);
        for _ in 0..120 {
            p.step(PlayerCommand::default(), 1.0 / 60.0, &[floor()]);
        }
        assert!(p.grounded);
    }
}
