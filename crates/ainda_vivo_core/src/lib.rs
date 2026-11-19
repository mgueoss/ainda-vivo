//! Simulation only: no window, operating system, renderer or network transport.
//! The desktop adapter translates hardware input into PlayerCommand.
pub mod collision;
pub mod movement;
pub mod reality;
pub mod story;

pub use bevy_math::{Vec2, Vec3};
