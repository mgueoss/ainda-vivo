mod brain;
mod nav;
pub(super) mod qa;
mod test_lab;

use super::{library::Catalog, motion};
use crate::{
    app::{CollisionWorld, LocalPlayer, Screen, Session},
    atmosphere::{
        Atmosphere,
        director::{Cue, Director},
    },
    scene::{
        presence::Presence,
        station::plan::{PORTALS, ROOMS, WATER_MAX, WATER_MIN},
    },
};
use ainda_vivo_core::{collision::Aabb, story::Action};
use bevy::prelude::*;
pub(crate) use brain::State;
use brain::{Brain, Frame, Intent, Noise, Tuning, View, forward, player_sees};
use nav::{HUMAN, Nav, Portal, Zone, point};
pub(super) use test_lab::{Command, Knob, TestLab, command, update_lab};

#[derive(Component, Clone, Copy)]
pub(super) struct Pose {
    pub clip: u8,
    pub sample: Option<f32>,
    pub speed: f32,
    pub paused: bool,
    pub lab: bool,
}

impl Pose {
    fn new(lab: bool) -> Self {
        Self {
            clip: 8,
            sample: None,
            speed: 1.0,
            paused: false,
            lab,
        }
    }
}

fn presentation(brain: &Brain, lab: bool, paused: bool) -> Pose {
    let mut pose = Pose {
        paused,
        ..Pose::new(lab)
    };
    if brain.moving > 0.01 {
        let (clip, nominal) = if brain.state == State::Approach {
            (19, 0.31)
        } else if brain.moving > 0.66 {
            (14, 1.12)
        } else {
            (13, 0.43)
        };
        pose.clip = clip;
        pose.speed = (brain.moving / nominal).clamp(0.15, 2.4);
    } else {
        let elapsed = brain.clock - brain.entered;
        (pose.clip, pose.sample) = match brain.state {
            State::Watch => (12, Some(((brain.head + 0.471) / 0.978).clamp(0.0, 1.0))),
            State::Peek if brain.goal.is_none() => (17, Some((elapsed * 0.44).min(0.51))),
            State::Threaten if elapsed < 1.14 => (23, Some((elapsed / 1.14).min(1.0))),
            State::Threaten if elapsed < 2.16 => (37, None),
            State::Threaten => (32, Some(((elapsed - 2.16) / 1.22).clamp(0.0, 1.0))),
            _ => (motion::State::Idle.preferred_clip(), None),
        };
    }
    pose
}

const STATION_ANCHORS: [Vec2; 17] = [
    Vec2::new(2.55, -21.36),
    Vec2::new(-5.4, -16.0),
    Vec2::new(-20.0, 4.6),
    Vec2::new(-14.0, -23.4),
    Vec2::new(17.0, -4.0),
    Vec2::new(24.0, -17.3),
    Vec2::new(-5.9, -34.0),
    Vec2::new(5.0, -30.6),
    Vec2::new(-3.62, -54.1),
    Vec2::new(-22.8, -40.0),
    Vec2::new(18.0, -40.0),
    Vec2::new(-27.0, -55.0),
    Vec2::new(22.0, -60.0),
    Vec2::new(-22.0, -72.0),
    Vec2::new(20.2, -78.0),
    Vec2::new(7.16, -80.25),
    Vec2::new(0.0, -89.5),
];

#[derive(Resource, Default)]
pub(crate) struct Campaign {
    requested: bool,
    selected: bool,
    actor: Option<Entity>,
    nav: Option<Nav>,
    brain: Brain,
    attempt: (u64, u32),
    noise: Vec<Noise>,
    footstep: Footstep,
    protected_until: f32,
    last_manifest: Option<f32>,
    gate_was_open: Option<bool>,
    retry: f32,
    catalog_revision: u32,
    pub(super) max_travel: f32,
    pub(super) started: u32,
}

impl Campaign {
    pub(super) fn new(requested: bool) -> Self {
        Self {
            requested,
            ..default()
        }
    }
    pub(crate) fn physical(&self) -> bool {
        self.selected
    }
    pub(crate) fn active(&self) -> bool {
        self.selected && self.brain.state != State::Absent
    }

    pub(crate) fn activity(&mut self, action: Action, at: Vec3, now: f32) {
        if !self.selected {
            return;
        }
        let (duration, power) = match action {
            Action::ReadNote
            | Action::ReadArchive
            | Action::ReadShiftLog
            | Action::ReadReturnMemo
            | Action::ReadBadge
            | Action::ReadServiceOrder
            | Action::ReviewRecording => (6.5, 0.0),
            Action::RestorePower | Action::CheckGate => (3.5, 2.0),
            Action::RestAtPost => (4.0, 0.0),
            _ => (2.8, 0.83),
        };
        self.protected_until = now + duration;
        if power > 0.0 && self.noise.len() < 6 {
            self.noise.push(Noise { at: at.xz(), power });
        }
    }
}

#[derive(Default)]
struct Footstep {
    before: Option<Vec2>,
    distance: f32,
}

impl Footstep {
    fn sample(&mut self, at: Vec2, speed: f32, crouched: bool) -> Option<Noise> {
        let delta = self.before.replace(at).map_or(0.0, |old| old.distance(at));
        if delta > 1.5 {
            self.distance = 0.0;
            return None;
        }
        self.distance += delta;
        let running = speed >= 3.0 && !crouched;
        let stride = if running { 1.15 } else { 0.80 };
        if self.distance < stride {
            return None;
        }
        self.distance %= stride;
        Some(Noise {
            at,
            power: if crouched {
                0.36
            } else if running {
                1.95
            } else {
                0.96
            },
        })
    }
}

pub(crate) fn legacy(mode: Option<Res<Campaign>>) -> bool {
    mode.is_none_or(|mode| !mode.physical())
}

fn station_nav(world: &CollisionWorld) -> Nav {
    let mut solids: Vec<_> = world.active(true).collect();
    // O tanque contém um rebaixo. Para este controlador de piso plano ele
    // é uma área vedada, independente da geometria das margens.
    solids.push(Aabb {
        min: point(WATER_MIN).with_y(-0.1),
        max: point(WATER_MAX).with_y(0.16),
    });
    Nav::new(
        ROOMS
            .iter()
            .map(|room| Zone {
                min: room.min,
                max: room.max,
            })
            .collect(),
        PORTALS
            .iter()
            .map(|p| Portal {
                rooms: p.rooms.map(|room| room as usize),
                at: p.at,
                width: p.width,
                height: p.height,
            })
            .collect(),
        solids,
        world
            .0
            .iter()
            .find(|solid| solid.gate)
            .map(|solid| solid.bounds),
    )
}

fn anchor(f: &Frame, order: usize) -> Option<(Vec2, f32)> {
    (0..f.points.len())
        .map(|i| f.points[(i + order) % f.points.len()])
        .filter(|&p| {
            let distance = p.distance(f.view.eye.xz());
            (7.5..29.0).contains(&distance)
                && f.nav.clear_body(p, p, HUMAN, f.door_closed)
                && !player_sees(f, point(p).with_y(1.75))
                && !player_sees(f, point(p).with_y(1.0))
                && f.nav.acoustic(p, f.view.eye.xz(), f.door_closed) > 0.0
        })
        .min_by(|a, b| {
            let distance = |p: &Vec2| (p.distance(f.view.eye.xz()) - 13.0).abs();
            distance(a).total_cmp(&distance(b))
        })
        .map(|p| {
            // A orientação inicial é a do posto escolhido, em direções cardeais.
            let yaw = ((order % 4) as f32) * std::f32::consts::FRAC_PI_2;
            (p, yaw)
        })
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Context<'w, 's> {
    session: Res<'w, Session>,
    atmosphere: Res<'w, Atmosphere>,
    silhouette: Res<'w, Presence>,
    world: Res<'w, CollisionWorld>,
    player: Res<'w, LocalPlayer>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    poses: Query<
        'w,
        's,
        (
            &'static mut Pose,
            &'static mut Transform,
            &'static mut Visibility,
        ),
    >,
}

pub(super) fn update_campaign(
    mut commands: Commands,
    catalog: Res<Catalog>,
    mut mode: ResMut<Campaign>,
    mut director: ResMut<Director>,
    time: Res<Time>,
    mut ctx: Context,
) {
    if ctx.keys.just_pressed(KeyCode::F5) {
        mode.requested = !mode.requested;
        info!("Presença física solicitada: {}", mode.requested);
    }
    let session = &ctx.session;
    let distinct = (session.attempt, session.perception_revision);
    if mode.attempt != distinct {
        mode.attempt = distinct;
        mode.brain = Brain::seeded(director.initial_seed() ^ 0x72ea_71cc);
        mode.brain.next_manifest = 12.0;
        mode.noise.clear();
        mode.footstep = Footstep::default();
        mode.protected_until = 0.0;
        mode.retry = 0.0;
        mode.last_manifest = None;
        mode.gate_was_open = None;
    }
    if mode.requested && !mode.selected && !ctx.silhouette.active() && catalog.number(12).is_some()
    {
        mode.selected = true;
    } else if !mode.requested && !mode.active() {
        mode.selected = false;
    }
    if !mode.selected {
        if let Some(id) = mode.actor
            && let Ok((mut pose, _, mut show)) = ctx.poses.get_mut(id)
        {
            pose.paused = true;
            show.set_if_neq(Visibility::Hidden);
        }
        return;
    }
    if session.screen != Screen::Playing || session.collapse_active() {
        if let Some(id) = mode.actor
            && let Ok((mut pose, _, mut show)) = ctx.poses.get_mut(id)
        {
            pose.paused = true;
            if session.screen != Screen::Paused {
                *show = Visibility::Hidden;
                mode.brain = Brain::seeded(director.initial_seed() ^ 0x72ea_71cc);
                mode.brain.next_manifest = 12.0;
            }
        }
        return;
    }
    if mode.nav.is_none() {
        mode.nav = Some(station_nav(&ctx.world));
    }
    if mode.catalog_revision != catalog.revision() {
        mode.catalog_revision = catalog.revision();
        if let Some(actor) = mode.actor.take() {
            commands.entity(actor).despawn();
        }
    }
    let player = &ctx.player;
    let feet = player.body.feet;
    let direction = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0) * Vec3::NEG_Z;
    let protected = session.elapsed < mode.protected_until
        || session.resting
        || session.wake_confirmation
        || ctx.atmosphere.quiet > 0.14
        || ctx.atmosphere.episode.holding_silence()
        || session.false_exit.is_some()
        || crate::scene::station::lamp_refuge(session.memory().desk_light, feet);
    let dt = time.delta_secs();
    let mode = mode.as_mut();
    let was = mode.brain.pos;
    let unlocked = session.progress().exit_unlocked();
    if mode.gate_was_open.replace(unlocked) == Some(false) && unlocked {
        mode.noise.push(Noise {
            at: crate::scene::station::EXIT_GATE.xz(),
            power: 3.2,
        });
    }
    if let Some(noise) = mode.footstep.sample(
        feet.xz(),
        player.body.velocity.xz().length(),
        player.body.crouched,
    ) && mode.noise.len() < 6
    {
        mode.noise.push(noise);
    }
    let frame = Frame {
        nav: mode.nav.as_ref().unwrap(),
        door_closed: !unlocked,
        view: View {
            eye: feet + Vec3::Y * player.eye_height,
            facing: direction,
            protected,
        },
        noises: &mode.noise,
        points: &STATION_ANCHORS,
    };
    let brain = &mut mode.brain;
    if brain.state == State::Absent
        && brain.clock >= brain.next_manifest
        && brain.clock >= mode.retry
        && mode.requested
        && !protected
        && brain.accepts(Intent::Observe)
        && director.allows(Cue::Presence, session.elapsed)
    {
        mode.retry = brain.clock + 0.7;
        if let Some((point, yaw)) = anchor(&frame, director.order) {
            brain.spawn(point, yaw);
            director.claim(Cue::Presence, session.elapsed);
            mode.last_manifest = Some(session.elapsed);
            mode.started += 1;
        }
    }
    let in_presence = brain.state != State::Absent;
    brain.step(dt, &frame, Tuning::default(), true);
    if in_presence {
        mode.max_travel += was.distance(brain.pos).min(dt * 2.0);
    }
    mode.noise.clear();
    if brain.state == State::Absent {
        if let Some(id) = mode.actor
            && let Ok((mut pose, _, mut show)) = ctx.poses.get_mut(id)
        {
            pose.paused = true;
            show.set_if_neq(Visibility::Hidden);
        }
        return;
    }
    let pose = presentation(brain, false, false);
    let transform = Transform::from_translation(point(brain.pos) + Vec3::Y * 0.022)
        .with_rotation(Quat::from_rotation_y(brain.yaw));
    if let Some(id) = mode.actor {
        if let Ok((mut current, mut t, mut show)) = ctx.poses.get_mut(id) {
            *current = pose;
            t.set_if_neq(transform);
            show.set_if_neq(Visibility::Inherited);
        }
    } else {
        mode.actor = catalog.spawn_observer(&mut commands, pose, transform, None);
    }
}
