//! Connects witnessed encounters to the authoritative expedition.
use crate::{
    app::{CollisionWorld, Comfort, LocalPlayer, MainCamera, Screen, Session},
    atmosphere::director::{Cue, Director},
    scene::presence::Presence,
};
use ainda_vivo_core::reality::{Approach, CONTACT_LIFETIME, CONTACT_RADIUS, ContactEvent};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Developer {
    pub enabled: bool,
    pub psychology: bool,
    pub seed: Option<u32>,
}

impl Developer {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args().collect();
        Self {
            enabled: args.iter().any(|arg| arg == "--dev-reality"),
            psychology: args.iter().any(|arg| arg == "--dev-psychology"),
            seed: args
                .windows(2)
                .find(|pair| pair[0] == "--seed")
                .and_then(|pair| pair[1].parse().ok()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Trace {
    pub at: f32,
    pub position: Vec3,
}

#[derive(Resource, Default)]
pub struct Threat {
    attempt: u64,
    revision: u32,
    trace: Option<Trace>,
    approach: Approach,
    pub warning: Option<Vec3>,
    pub warnings: u32,
    pub captures: u32,
}

impl Threat {
    pub fn active(&self) -> bool {
        self.approach.active()
    }
}

pub fn update(
    mut session: ResMut<Session>,
    player: Res<LocalPlayer>,
    presence: Res<Presence>,
    comfort: Res<Comfort>,
    world: Res<CollisionWorld>,
    camera: Query<&Transform, With<MainCamera>>,
    mut threat: ResMut<Threat>,
) {
    if (threat.attempt, threat.revision) != (session.attempt, session.perception_revision) {
        *threat = Threat {
            attempt: session.attempt,
            revision: session.perception_revision,
            ..default()
        };
    }
    threat.warning = None;
    if !session.expedition.can_capture() {
        threat.approach = Approach::Closed;
        return;
    }
    if session.screen != Screen::Playing || session.collapse_active() {
        return;
    }
    if let Some(trace) = presence.contact
        && threat.trace.is_none_or(|last| last.at < trace.at)
    {
        threat.trace = Some(trace);
        threat.approach = Approach::Waiting;
    }
    let Some(trace) = threat.trace else { return };
    let now = session.elapsed;
    if now - trace.at >= CONTACT_LIFETIME {
        threat.approach = Approach::Closed;
        return;
    }
    let Ok(camera) = camera.single() else { return };
    let target = trace.position + Vec3::Y * 1.4;
    let offset = target - camera.translation;
    let within_reach = offset.xz().length_squared() <= CONTACT_RADIUS * CONTACT_RADIUS
        && offset.y.abs() < 1.45
        && !world.occludes(
            camera.translation,
            target,
            session.progress().exit_unlocked(),
        );
    let cover = if comfort.flashlight { 0.46 } else { 0.71 };
    let exposed = player.body.grounded
        && player.body.velocity.xz().length_squared() < 1.55 * 1.55
        && camera.forward().dot(offset.normalize_or_zero()) < cover;
    match threat.approach.step(now, within_reach, exposed) {
        ContactEvent::Warning => {
            threat.warning = Some(trace.position);
            threat.warnings += 1;
            session.stimulate(
                ainda_vivo_core::psychology::Stimulus::Contact,
                ainda_vivo_core::psychology::Origin::Presence,
                0.78,
            );
        }
        ContactEvent::Capture if session.expedition.capture(now) => threat.captures += 1,
        _ => (),
    }
}

pub fn tools(
    developer: Res<Developer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    mut director: ResMut<Director>,
) {
    if !developer.enabled || session.screen != Screen::Playing || session.collapse_active() {
        return;
    }
    if keys.just_pressed(KeyCode::F6) {
        let now = session.elapsed;
        let captured = session.expedition.capture(now);
        info!("DEV captura solicitada: {captured} (requer rearme real e tentativa elegível)");
    }
    if keys.just_pressed(KeyCode::F7) {
        director.request(Cue::Illusion);
        info!("DEV ilusão solicitada para a próxima oportunidade válida");
    }
    if keys.just_pressed(KeyCode::F8) {
        let observer = session.expedition.observer();
        info!(
            "DEV {:?}: {:?}; mundo {:?}, visor {:?}, desfecho {:?}; seed {}, modo {:?}, intensidade {:.2}; últimos eventos {:?}",
            observer.id,
            observer.reality(),
            session.expedition.truth().phase(),
            session.progress().phase(),
            observer.outcome(),
            director.initial_seed(),
            director.mood,
            director.intensity(),
            director.recent(),
        );
        info!(
            "DEV OS 17: {:?}; setores recentes {:?}; aparição recente {}",
            observer.memory.service,
            observer.memory.recent_route(),
            observer.memory.recent_sighting(session.elapsed),
        );
    }
}

pub fn exit_line(session: &Session) -> bool {
    session.false_exit == Some(crate::app::ExitPresentation::Contradiction)
}
