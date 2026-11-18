use crate::app::*;
use ainda_vivo_core::{collision::Aabb, story::Action};
use bevy::prelude::*;

pub const REACH: f32 = 2.4;

#[derive(Component)]
pub struct Interactable {
    pub bounds: Aabb,
    pub action: Action,
}

pub fn interact(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::preferences::Preferences>,
    world: Res<CollisionWorld>,
    mut session: ResMut<Session>,
    camera: Query<&Transform, With<MainCamera>>,
    targets: Query<&Interactable>,
) {
    session.prompt.clear();
    if session.screen != Screen::Playing {
        return;
    }
    let Ok(camera) = camera.single() else {
        return;
    };
    let origin = camera.translation;
    let direction = *camera.forward();
    let mut best = REACH;
    let mut target = None;
    for item in &targets {
        let Some(label) = session.progress.prompt(item.action) else {
            continue;
        };
        if let Some(distance) = item.bounds.ray(origin, direction, REACH)
            && distance <= best
        {
            best = distance;
            target = Some((item, label));
        }
    }
    let Some((target, label)) = target else {
        return;
    };
    // Prevent interaction through walls. Target surfaces can overlap their own
    // supporting geometry slightly, so use a small, bounded tolerance.
    for solid in world.active(session.progress.exit_unlocked()) {
        if solid
            .ray(origin, direction, best)
            .is_some_and(|d| d + 0.035 < best)
        {
            return;
        }
    }
    let key = format!("{:?}", bindings.key("interact", KeyCode::KeyE));
    session.prompt = format!("[{}] {}", key.strip_prefix("Key").unwrap_or(&key), label);
    if keys.just_pressed(bindings.key("interact", KeyCode::KeyE)) {
        let message = session.progress.apply(target.action);
        session.say(message);
    }
}
