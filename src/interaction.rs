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
    #[cfg(feature = "dev-lab")] mut physical: Option<ResMut<crate::lab::observer::Campaign>>,
) {
    session.prompt.clear();
    if session.screen != Screen::Playing || session.collapse_active() {
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
        let Some(mut label) = session.progress().prompt(item.action) else {
            continue;
        };
        if item.action == Action::ToggleDeskLight {
            label = if session.memory().desk_light {
                "apagar luminária da mesa"
            } else {
                "acender luminária da mesa"
            };
        }
        #[cfg(feature = "dev-lab")]
        if item.action == Action::CheckRelay
            && physical
                .as_ref()
                .is_some_and(|mode| mode.relay_ready(session.elapsed))
        {
            label = "conferir via e testar o relé";
        }
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
    for solid in world.active(session.progress().exit_unlocked()) {
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
        #[cfg(feature = "dev-lab")]
        if let Some(physical) = &mut physical {
            physical.activity(
                target.action,
                (target.bounds.min + target.bounds.max) * 0.5,
                session.elapsed,
            );
        }
        session.act(target.action);
    }
}
