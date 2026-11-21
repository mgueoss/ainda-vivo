//! Local observer's psychological simulation and presentation signals.
pub mod presentation;

use crate::{
    app::*,
    atmosphere::director::{Cue, Director},
    reality::{Developer, Threat},
    scene::{presence::Presence, station},
};
use ainda_vivo_core::psychology::{Level, Recovery};
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Resource, Default)]
pub struct Effects {
    pub intensity: f32,
    pub stage: u8,
    pub critical: f32,
    pub episode: f32,
    pub remaining: Option<f32>,
    pub farewell: Option<f32>,
    pub hallucination: Option<Vec3>,
    pub echo: Option<Vec3>,
}

pub fn recovery(session: &Session, player: &LocalPlayer) -> Recovery {
    let in_light = station::near_rest_post(player.body.feet)
        || station::lamp_refuge(session.memory().desk_light, player.body.feet);
    if !in_light || !player.body.grounded {
        Recovery::None
    } else if session.resting && player.body.velocity.xz().length_squared() < 0.16 {
        Recovery::Rest
    } else {
        Recovery::Refuge
    }
}

pub fn reserve(
    keys: Res<ButtonInput<KeyCode>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut session: ResMut<Session>,
) {
    if keys.just_pressed(KeyCode::KeyR)
        && !keys.just_pressed(KeyCode::Escape)
        && window.single().is_ok_and(|window| window.focused)
        && session.screen == Screen::Playing
        && !session.collapse_active()
        && session.psychology.consume_reserve()
    {
        session.say("O ritmo volta por um instante.");
    }
}

pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    dev: Res<Developer>,
    mut session: ResMut<Session>,
    mut director: ResMut<Director>,
    mut forced: ResMut<ForceCue>,
) {
    if !dev.psychology || session.screen != Screen::Playing {
        return;
    }
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keys.just_pressed(KeyCode::F10) || keys.just_pressed(KeyCode::F11) {
        session.farewell = None;
        let value = session.psychology.value();
        let desired = if keys.just_pressed(KeyCode::F11) {
            [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
                .into_iter()
                .find(|floor| *floor > value)
                .unwrap_or(0.0)
        } else if shift {
            value - 10.0
        } else {
            value + 10.0
        };
        session.psychology.set_for_development(desired);
    }
    if keys.just_pressed(KeyCode::F12) {
        if shift {
            session.psychology.set_for_development(100.0);
            session.farewell = Some(0.0);
        } else {
            forced.0 = true;
            director.request(Cue::Illusion);
        }
    }
    if keys.just_pressed(KeyCode::F9) {
        info!(
            "COLAPSO dev {:.1} {:?} janela={:?} trecho={:?} recuperação={:?} realidade={:?} fontes={:?}",
            session.psychology.value(),
            session.psychology.level(),
            session.psychology.critical_remaining(),
            session.psychology.critical_phase(),
            session.psychology.recovery(),
            session.expedition.observer().reality(),
            session.psychology.records().collect::<Vec<_>>()
        );
    }
}

#[derive(Default)]
struct Episode {
    area: Option<Vec3>,
    until: f32,
    next: f32,
    attempt: u64,
    revision: u32,
    forced: bool,
}

#[derive(Default)]
pub struct Perception(Episode);

#[derive(Resource, Default)]
pub struct ForceCue(bool);

pub fn perception(
    time: Res<Time>,
    session: Res<Session>,
    (presence, threat): (Res<Presence>, Res<Threat>),
    (world, camera): (Res<CollisionWorld>, Query<&Transform, With<MainCamera>>),
    mut effects: ResMut<Effects>,
    mut director: ResMut<Director>,
    (mut state, mut forced): (Local<Perception>, ResMut<ForceCue>),
) {
    // One persistent schedule for this observer; no per-frame entities or allocations here.
    let scene = &mut state.0;
    if (scene.attempt, scene.revision) != (session.attempt, session.perception_revision) {
        *scene = Episode {
            attempt: session.attempt,
            revision: session.perception_revision,
            next: session.elapsed + 7.0,
            ..default()
        };
        effects.intensity = 0.0;
    }
    scene.forced |= std::mem::take(&mut forced.0);
    let mind = &session.psychology;
    effects.stage = mind.level() as u8;
    effects.remaining = mind.critical_remaining();
    effects.critical = mind.critical_progress().unwrap_or(0.0);
    effects.farewell = session.farewell;
    effects.echo = None;
    effects.episode = match mind.motor() {
        ainda_vivo_core::psychology::Motor::Steady => 0.0,
        ainda_vivo_core::psychology::Motor::Hesitation => 0.42,
        ainda_vivo_core::psychology::Motor::Kneeling => 1.0,
    };
    if session.screen == Screen::Paused {
        return;
    }
    if session.screen != Screen::Playing || session.collapse_active() {
        effects.hallucination = None;
        return;
    }
    let smoothing = 1.0 - (-1.8 * time.delta_secs()).exp();
    let base = ((mind.value() - 12.0) / 88.0).clamp(0.0, 1.0);
    effects.intensity += (base - effects.intensity) * smoothing;
    let now = session.elapsed;
    if now < scene.until {
        effects.hallucination = scene.area;
        return;
    }
    effects.hallucination = None;
    if ((!scene.forced && mind.level() < Level::Paranoia) || now < scene.next)
        || mind.recovery() != Recovery::None
        || presence.active()
        || threat.active()
        || !director.allows(Cue::Illusion, now)
    {
        return;
    }
    let Ok(cam) = camera.single() else { return };
    let forward = cam.forward().with_y(0.0).normalize_or_zero();
    let sign = if director.order.is_multiple_of(2) {
        1.0
    } else {
        -1.0
    };
    let right = *cam.right() * sign;
    let sample = cam.translation + forward * 5.0 + right * 2.2;
    let clear = !world.occludes(cam.translation, sample, session.progress().exit_unlocked());
    if !clear {
        return;
    }
    let visual = scene.forced || (effects.stage >= 3 && !director.order.is_multiple_of(3));
    let point = sample.with_y(0.01);
    scene.area = visual.then_some(point);
    scene.until = now + 2.0;
    scene.next =
        now + match effects.stage {
            5 => 15.0,
            4 => 20.0,
            _ => 28.0,
        } + (director.order % 8) as f32;
    scene.forced = false;
    effects.hallucination = scene.area;
    effects.echo = (!visual).then_some(point + Vec3::Y * 0.15);
    director.claim(Cue::Illusion, now);
}
