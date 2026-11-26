mod brain;
mod cognition;
#[cfg(feature = "dev-lab")]
pub(crate) mod cognition_qa;
pub(crate) mod hunt;
#[cfg(feature = "dev-lab")]
mod hunt_lab;
#[cfg(feature = "dev-lab")]
pub(crate) mod hunt_qa;
mod nav;
#[cfg(feature = "dev-lab")]
pub(crate) mod qa;
#[cfg(feature = "dev-lab")]
pub(crate) mod scenario;
pub(crate) mod strategy;
mod tactics;
#[cfg(feature = "dev-lab")]
mod test_lab;

use super::{motion, rig::Catalog};
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
#[cfg(feature = "dev-lab")]
use brain::forward;
use brain::{Brain, Frame, Noise, Tuning, View, angle, player_sees};
#[cfg(feature = "dev-lab")]
pub(crate) use hunt_lab::{
    HuntCommand, HuntKnob, HuntProfile, diagnostics as hunt_diagnostics, hunt_command,
};
use nav::{HUMAN, Nav, Portal, Zone, point};
#[cfg(feature = "dev-lab")]
pub(crate) use scenario::BotStyle;
use strategy::{Delivery, Illusion};
#[cfg(feature = "dev-lab")]
pub(crate) use test_lab::{Command, Knob, TestLab, command, update_lab};

#[derive(Component, Clone, Copy)]
pub(crate) struct Pose {
    pub clip: u8,
    pub sample: Option<f32>,
    pub speed: f32,
    pub paused: bool,
    pub lab: bool,
    pub hunting: bool,
}

impl Pose {
    fn new(lab: bool) -> Self {
        Self {
            clip: 8,
            sample: None,
            speed: 1.0,
            paused: false,
            lab,
            hunting: false,
        }
    }
}

pub(crate) fn presentation(brain: &Brain, lab: bool, paused: bool) -> Pose {
    let mut pose = Pose {
        paused,
        ..Pose::new(lab)
    };
    if brain.hunt.active() {
        (pose.clip, pose.sample, pose.speed) = brain.hunt.animation(brain.moving);
        pose.hunting = true;
        return pose;
    }
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

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub(crate) enum HuntReject {
    Absent,
    Protected,
    Busy,
    NoSight,
    Posture,
    Range,
    TooEarly,
    Mission,
    Strain,
    Escape,
    Cooldown,
    Pacing,
    Recent,
    Draw,
}

impl HuntReject {
    const ALL: [Self; 14] = [
        Self::Absent,
        Self::Protected,
        Self::Busy,
        Self::NoSight,
        Self::Posture,
        Self::Range,
        Self::TooEarly,
        Self::Mission,
        Self::Strain,
        Self::Escape,
        Self::Cooldown,
        Self::Pacing,
        Self::Recent,
        Self::Draw,
    ];
}

#[derive(Default)]
struct Opportunities {
    next: f32,
    rejected: [u32; HuntReject::ALL.len()],
    accepted: u32,
    bluff: u32,
    runs: u32,
    contacts: u32,
}

/// Messages in this channel disclose presentation state only. The audio system
/// may inspect the listener. No position or response travels back to the brain.
#[derive(Clone, Copy, Message)]
pub(crate) struct PerceptionOrder {
    pub stamp: (u64, u32),
    pub id: u32,
    pub kind: Illusion,
    pub sector: u8,
    pub actor_sector: u8,
    pub intensity: f32,
    pub earliest: f32,
    pub deadline: f32,
}

#[derive(Clone, Copy, Message)]
pub(crate) struct PerceptionReceipt {
    pub stamp: (u64, u32),
    pub id: u32,
    pub delivery: Delivery,
}

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
    relay: Relay,
    protected_until: f32,
    last_manifest: Option<f32>,
    gate_was_open: Option<bool>,
    retry: f32,
    catalog_revision: u32,
    opportunities: Opportunities,
    witnessed_for: f32,
    witness_at: f32,
    witnessed: bool,
    sustained: bool,
    tracing: bool,
    plans_traced: (u32, u32),
    pub(super) max_travel: f32,
    pub(super) started: u32,
}

impl Campaign {
    pub(crate) fn new(requested: bool) -> Self {
        Self {
            requested,
            tracing: std::env::args().any(|argument| argument == "--trace-presence"),
            ..default()
        }
    }
    pub(crate) fn physical(&self) -> bool {
        self.selected
    }

    pub(crate) fn supports(&self, order: PerceptionOrder) -> bool {
        self.active()
            && self.attempt == order.stamp
            && self.brain.mind.waiting_for(order.id)
            && self.nav.as_ref().and_then(|nav| nav.sector(self.brain.pos))
                != Some(order.sector as usize)
    }

    pub(crate) fn trace_enabled(&self) -> bool {
        self.tracing
    }

    pub(crate) fn qa_counts(&self) -> (u32, u32, u32) {
        (
            self.started,
            self.opportunities.runs,
            self.brain.mind.finished,
        )
    }

    pub(crate) fn report(&self) -> String {
        use std::fmt::Write;
        let o = &self.opportunities;
        let mut data = format!(
            "physical={} manifests={} accepted={} bluffs={} runs={} contacts={} plans={:?} finished={}\n",
            self.physical(),
            self.started,
            o.accepted,
            o.bluff,
            o.runs,
            o.contacts,
            self.brain.mind.totals,
            self.brain.mind.finished
        );
        for reason in HuntReject::ALL {
            let _ = writeln!(data, "reject {reason:?} {}", o.rejected[reason as usize]);
        }
        let _ = writeln!(
            data,
            "buffers_astar_bytes={} route_replans={} blocks={} recovered={}",
            self.brain.route.bytes(),
            self.brain.memory.replans,
            self.brain.memory.blocked,
            self.brain.memory.recovered
        );
        for item in self.brain.memory.episodes.iter().flatten().take(4) {
            let _ = writeln!(
                data,
                "episódio {:.2}s setor={:?} {}",
                item.when, item.sector, item.reason
            );
        }
        data.push_str(&self.brain.mind.report(self.brain.clock));
        data
    }
    pub(crate) fn active(&self) -> bool {
        self.selected && self.brain.state != State::Absent
    }

    pub(crate) fn hunt_intensity(&self) -> u8 {
        if !self.selected || !self.brain.hunt.active() {
            return 0;
        }
        let h = &self.brain.hunt;
        if h.phase == hunt::Phase::Run
            && matches!(
                h.strategy,
                hunt::Strategy::Direct | hunt::Strategy::Intercept
            )
        {
            2
        } else {
            1
        }
    }

    pub(crate) fn quiet_sector(&self) -> Option<u8> {
        if !self.selected
            || self.brain.visible
            || self.brain.hunt.active()
            || !matches!(
                self.brain.state,
                State::Follow | State::Investigate | State::Peek
            )
        {
            return None;
        }
        self.nav.as_ref()?.sector(self.brain.pos).map(|it| it as u8)
    }

    pub(crate) fn activity(&mut self, action: Action, at: Vec3, now: f32) {
        if !self.selected {
            return;
        }
        if action == Action::CheckRelay {
            self.relay.arm(at, now);
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

    pub(crate) fn relay_ready(&self, now: f32) -> bool {
        self.selected && self.relay.at.is_none() && now >= self.relay.ready_at
    }
}

#[derive(Message)]
pub(crate) struct RelayClick(pub(crate) Vec3);

#[derive(Default)]
struct Relay {
    at: Option<(f32, Vec3)>,
    ready_at: f32,
}

impl Relay {
    fn arm(&mut self, at: Vec3, now: f32) {
        if self.at.is_none() && now >= self.ready_at {
            self.at = Some((now + 1.45, at));
            self.ready_at = now + 24.0;
        }
    }

    fn tick(&mut self, now: f32) -> Option<Vec3> {
        if self.at.is_some_and(|(due, _)| now >= due) {
            return self.at.take().map(|(_, at)| at);
        }
        None
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

pub(crate) fn legacy(mode: Res<Campaign>, catalog: Res<Catalog>) -> bool {
    !mode.physical() && (!mode.requested || catalog.error.is_some())
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
            // The staging camera chooses hiding space. Orientation comes only
            // from that post's exits and its static sightlines.
            let yaw = f
                .nav
                .sector(p)
                .and_then(|sector| {
                    f.nav
                        .exits(sector, f.door_closed)
                        .map(|(_, exit)| exit)
                        .filter(|&exit| {
                            f.nav.clear(
                                point(p).with_y(1.80),
                                point(exit).with_y(1.62),
                                f.door_closed,
                            )
                        })
                        .min_by(|a, b| p.distance_squared(*a).total_cmp(&p.distance_squared(*b)))
                })
                .map_or(((order % 4) as f32) * std::f32::consts::FRAC_PI_2, |exit| {
                    angle(exit - p)
                });
            (p, yaw)
        })
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Context<'w, 's> {
    session: ResMut<'w, Session>,
    #[cfg(feature = "dev-lab")]
    lab: Option<Res<'w, crate::lab::Lab>>,
    atmosphere: Res<'w, Atmosphere>,
    comfort: Res<'w, crate::app::Comfort>,
    silhouette: Res<'w, Presence>,
    world: Res<'w, CollisionWorld>,
    player: Res<'w, LocalPlayer>,
    #[cfg(feature = "dev-lab")]
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
    signals: MessageWriter<'w, hunt::Signal>,
    relay: MessageWriter<'w, RelayClick>,
    #[cfg(feature = "dev-lab")]
    hunt_qa: Option<Res<'w, hunt_qa::Check>>,
    orders: MessageWriter<'w, PerceptionOrder>,
    receipts: MessageReader<'w, 's, PerceptionReceipt>,
}

pub(super) fn update_campaign(
    mut commands: Commands,
    catalog: Res<Catalog>,
    mut mode: ResMut<Campaign>,
    mut director: ResMut<Director>,
    time: Res<Time>,
    mut ctx: Context,
) {
    #[cfg(feature = "dev-lab")]
    if ctx.lab.is_some() && ctx.keys.just_pressed(KeyCode::F5) {
        mode.requested = !mode.requested;
        info!("Presença física solicitada: {}", mode.requested);
    }
    let session = &mut *ctx.session;
    let distinct = (session.attempt, session.perception_revision);
    if mode.attempt != distinct {
        mode.attempt = distinct;
        mode.brain = Brain::campaign(director.initial_seed() ^ 0x72ea_71cc);
        mode.noise.clear();
        mode.footstep = Footstep::default();
        mode.relay = Relay::default();
        mode.started = 0;
        mode.protected_until = 0.0;
        mode.retry = 0.0;
        mode.last_manifest = None;
        mode.gate_was_open = None;
        mode.opportunities = Opportunities::default();
        mode.witnessed_for = 0.0;
        mode.witness_at = 0.0;
        mode.witnessed = false;
        mode.sustained = false;
        mode.plans_traced = (0, 0);
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
                if mode.brain.state != State::Absent {
                    mode.brain.state = State::Absent;
                }
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
    let safe_post = crate::scene::station::near_rest_post(feet);
    let dt = time.delta_secs();
    let mode = mode.as_mut();
    let was = mode.brain.pos;
    let unlocked = session.progress().exit_unlocked();
    if let Some(at) = mode.relay.tick(session.elapsed) {
        if mode.noise.len() < 6 {
            mode.noise.push(Noise {
                at: at.xz(),
                power: 4.1,
            });
        }
        ctx.relay.write(RelayClick(at));
    }
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
            protected: protected || mode.brain.hunt.active() && safe_post,
        },
        noises: &mode.noise,
        points: &STATION_ANCHORS,
    };
    let brain = &mut mode.brain;
    for receipt in ctx
        .receipts
        .read()
        .filter(|message| message.stamp == distinct)
    {
        brain
            .mind
            .feedback(receipt.id, receipt.delivery, brain.clock);
    }
    if brain.state == State::Absent
        && brain.clock >= brain.next_manifest
        && brain.clock >= mode.retry
        && mode.requested
        && !protected
        && director.allows(Cue::Presence, session.elapsed)
    {
        mode.retry = brain.clock + 0.7;
        if let Some((point, yaw)) = anchor(&frame, director.order) {
            brain.spawn(point, yaw);
            director.claim(Cue::Presence, session.elapsed);
            mode.last_manifest = Some(session.elapsed);
            mode.started += 1;
            mode.witnessed_for = 0.0;
            mode.witnessed = false;
            mode.sustained = false;
        }
    }
    let now = session.elapsed;
    if now >= mode.opportunities.next && mode.requested {
        mode.opportunities.next = now + 0.35;
        let assessment = opportunity(brain, &frame, protected || safe_post).and_then(|clue| {
            director
                .hunt_window(
                    now,
                    session.progress().phase(),
                    session.psychology.value(),
                    clue.velocity.length(),
                    escape_available(&frame, feet.xz(), brain.pos),
                )
                .map(|(prepare, bluff)| (clue, prepare, bluff))
        });
        match assessment {
            Ok((clue, prepare, bluff)) => {
                mode.opportunities.accepted += 1;
                mode.opportunities.bluff += u32::from(bluff);
                if mode.tracing {
                    info!(
                        "CAÇA {now:.2}s aceita blefe={bluff} distância={:.1}",
                        brain.pos.distance(clue.at)
                    );
                }
                brain.mind.hunting(brain.clock);
                brain.change(State::Threaten, "início da ameaça", prepare, None);
                brain.hunt = hunt::Hunt::start(
                    hunt::Tuning {
                        prepare,
                        ..default()
                    },
                    clue.at,
                    bluff,
                );
            }
            Err(reason) => {
                let count = &mut mode.opportunities.rejected[reason as usize];
                *count += 1;
                if mode.tracing && (*count == 1 || count.is_multiple_of(25)) {
                    info!("CAÇA {now:.2}s recusada {reason:?} (n={count})");
                }
            }
        }
    }
    let in_presence = brain.state != State::Absent;
    let was_running = brain.hunt.phase == hunt::Phase::Run;
    brain.step(dt, &frame, Tuning::default(), true);
    if !was_running && brain.hunt.phase == hunt::Phase::Run {
        mode.opportunities.runs += 1;
    }
    if let Some(order) = brain.mind.request() {
        ctx.orders.write(PerceptionOrder {
            stamp: distinct,
            id: order.id,
            kind: order.kind,
            sector: order.sector as u8,
            actor_sector: order.actor_sector as u8,
            intensity: order.intensity,
            earliest: now + 0.08,
            deadline: now + 2.25,
        });
    }
    let plans = (brain.mind.totals.iter().sum(), brain.mind.finished);
    if mode.tracing && mode.plans_traced != plans {
        info!("ESTRATÉGIA {now:.2}s {}", brain.mind.report(brain.clock));
        mode.plans_traced = plans;
    }
    if std::mem::take(&mut brain.hunt.pending) {
        mode.opportunities.contacts += 1;
        #[cfg(feature = "dev-lab")]
        let forced = ctx.hunt_qa.is_some();
        #[cfg(not(feature = "dev-lab"))]
        let forced = false;
        let selected = if forced {
            hunt::Contact::Capture
        } else {
            director.hunt_contact(session.expedition.can_capture(), session.psychology.value())
        };
        brain.hunt.result = Some(hunt::apply(session, selected, true));
    }
    if let Some(cue) = brain.hunt.cue.take() {
        let silent = matches!(
            brain.hunt.result,
            Some(hunt::Contact::Capture | hunt::Contact::Quiet)
        );
        let quiet_entrance = (director.initial_seed() ^ mode.started.wrapping_mul(0x9e3779b9))
            .is_multiple_of(3)
            && matches!(
                cue,
                hunt::Cue::Ready | hunt::Cue::Rupture | hunt::Cue::Acceleration
            );
        if !silent
            && !quiet_entrance
            && !(cue == hunt::Cue::Contact
                && brain.hunt.phase == hunt::Phase::Brake
                && brain.hunt.result == Some(hunt::Contact::Intimidation))
        {
            ctx.signals.write(hunt::Signal {
                cue,
                at: point(brain.pos).with_y(0.45),
                lab: false,
            });
        }
        if cue == hunt::Cue::Acceleration && session.psychology.value() < 75.0 {
            session.stimulate(
                ainda_vivo_core::psychology::Stimulus::Gaze,
                ainda_vivo_core::psychology::Origin::Presence,
                0.30,
            );
        }
    }
    if in_presence {
        mode.max_travel += was.distance(brain.pos).min(dt * 2.0);
    }
    if now >= mode.witness_at {
        mode.witness_at = now + 0.10;
        let range = if ctx.comfort.flashlight {
            17.0_f32
        } else {
            9.0
        };
        let witnesses = brain.state != State::Absent
            && !brain.hunt.hidden()
            && !protected
            && brain.pos.distance_squared(feet.xz()) < range.powi(2)
            && player_sees(&frame, brain.eye());
        if witnesses {
            mode.witnessed_for += 0.10;
            if !mode.witnessed && mode.witnessed_for >= 0.35 {
                mode.witnessed = true;
                session.memory_mut().note_sighting(now);
                session.stimulate(
                    ainda_vivo_core::psychology::Stimulus::Sighting,
                    ainda_vivo_core::psychology::Origin::Presence,
                    0.45,
                );
            }
            if !mode.sustained && mode.witnessed_for >= 2.25 {
                mode.sustained = true;
                session.stimulate(
                    ainda_vivo_core::psychology::Stimulus::Gaze,
                    ainda_vivo_core::psychology::Origin::Presence,
                    0.37,
                );
            }
        } else {
            mode.witnessed_for = 0.0;
        }
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
            show.set_if_neq(if brain.hunt.hidden() {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            });
        }
    } else {
        mode.actor = catalog.spawn_observer(&mut commands, pose, transform, None);
    }
}

fn opportunity(brain: &Brain, frame: &Frame, protected: bool) -> Result<brain::Memory, HuntReject> {
    use HuntReject::*;
    if brain.state == State::Absent {
        return Err(Absent);
    }
    if protected {
        return Err(Protected);
    }
    if brain.hunt.active()
        || brain.walking_portal()
        || brain.mind.active() && !brain.mind.permit_hunt()
    {
        return Err(Busy);
    }
    let Some(clue) = brain
        .seen
        .filter(|m| brain.visible && m.age(brain.clock) < 0.22)
    else {
        return Err(NoSight);
    };
    if !(5.5..17.2).contains(&brain.pos.distance(clue.at)) {
        return Err(Range);
    }
    if !matches!(
        brain.state,
        State::Watch | State::Approach | State::Threaten | State::Follow
    ) || brain.clock - brain.entered < 0.45
        || !hunt::motor_clear(
            frame,
            brain.pos,
            brain.pos,
            brain.yaw,
            angle(clue.at - brain.pos),
        )
    {
        return Err(Posture);
    }
    Ok(clue)
}

fn escape_available(frame: &Frame, player: Vec2, threat: Vec2) -> bool {
    let half = Vec3::new(0.27, 0.91, 0.27);
    let away = (player - threat).normalize_or_zero();
    let side = Vec2::new(away.y, -away.x);
    [
        away,
        side,
        -side,
        (away + side).normalize_or_zero(),
        (away - side).normalize_or_zero(),
    ]
    .into_iter()
    .filter(|d| {
        d.length_squared() > 0.5
            && frame
                .nav
                .clear_body(player, player + *d * 3.0, half, frame.door_closed)
    })
    .take(2)
    .count()
        == 2
}

#[cfg(test)]
mod relay_tests {
    use super::*;

    #[test]
    fn access_portals_are_classified_from_actual_station_colliders() {
        use crate::scene::station::plan::Sector::{Access, Dock, Records};
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<CollisionWorld>()
            .add_systems(Startup, crate::scene::setup);
        app.update();
        let nav = station_nav(app.world().resource::<CollisionWorld>());
        let half_low = Vec3::new(nav::LOW.z, nav::LOW.y, nav::LOW.x);
        for sector in [Dock, Records] {
            let p = PORTALS
                .iter()
                .find(|p| p.rooms == [Access, sector])
                .unwrap();
            let mut pair = None;
            for sample in -8..=8 {
                let center = p.at + Vec2::Y * (p.width * 0.44 * sample as f32 / 8.0);
                let start = center - Vec2::X * 1.12;
                let goal = center + Vec2::X * 1.12;
                assert!(!nav.clear_body(start, goal, half_low, false));
                if nav.clear_body(start, goal, HUMAN, false) {
                    pair = Some((start, goal));
                }
            }
            let (start, goal) = pair.expect("deve existir uma passagem em pé");
            let mut route = Vec::new();
            assert!(nav.uses_narrow_crossing(start, &[goal]));
            assert!(nav.route(start, goal, false, &mut route));
            assert!(nav.uses_narrow_crossing(start, &route));
            println!("Acesso/{sector:?}: rota em pé de {start:?} a {goal:?} {route:?}");
        }
    }

    #[test]
    fn relay_click_is_delayed_and_cannot_be_rearmed_during_cooldown() {
        let at = Vec3::new(2., 1.1, 3.);
        let mut relay = Relay::default();
        relay.arm(at, 0.);
        relay.arm(at * 2., 1.);
        assert_eq!(relay.tick(1.44), None);
        assert_eq!(relay.tick(1.45), Some(at));
        relay.arm(at, 10.);
        assert_eq!(relay.tick(22.), None);
        relay.arm(at, 24.);
        assert_eq!(relay.tick(25.46), Some(at));
    }
}
