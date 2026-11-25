pub(crate) use super::presence_bank::Family;
use super::{
    presence_bank::{Bank, Surface, VARIANTS, next},
    spatial::{MixTarget, RATE, Voice, channels},
};
use crate::{
    app::{CollisionWorld, Comfort, MainCamera, Screen, Session},
    atmosphere::{
        Atmosphere,
        director::{Cue as Beat, Director},
    },
    lab::{
        Lab, LabCamera,
        observer::{
            Campaign,
            hunt::{Cue, Signal},
        },
    },
    scene::{
        presence::Presence,
        station::{
            self,
            plan::{PORTALS, Sector, get},
        },
    },
};
use ainda_vivo_core::psychology::{Origin, Stimulus};
use bevy::{audio::AudioSinkPlayback, prelude::*};

const RANGE: f32 = 36.0;
pub(crate) const MAX_VOICES: usize = 7;

pub(crate) struct PresenceAudio;
impl Plugin for PresenceAudio {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bank>()
            .add_message::<Signal>()
            .add_message::<Request>()
            .add_systems(
                Update,
                drive_illusions
                    .after(super::emit)
                    .before(super::emit_incidents)
                    .run_if(crate::lab::campaign_running),
            )
            .add_systems(Update, (physical, mix_presence).chain().after(super::mix));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Physical,
    Illusion,
    FalseReality,
}

impl Category {
    pub const ALL: [Self; 3] = [Self::Physical, Self::Illusion, Self::FalseReality];
    pub fn label(self) -> &'static str {
        match self {
            Self::Physical => "Manifestação",
            Self::Illusion => "Ilusão do diretor",
            Self::FalseReality => "Falsa realidade",
        }
    }
}

#[derive(Message, Clone, Copy)]
pub struct Request {
    pub family: Family,
    pub category: Category,
    pub at: Vec3,
    pub variant: Option<usize>,
    pub gain: f32,
    pub pitch: f32,
    pub reverb: bool,
    pub occlusion: bool,
    pub lab: bool,
}

impl Request {
    pub fn new(family: Family, category: Category, at: Vec3, lab: bool) -> Self {
        Self {
            family,
            category,
            at,
            lab,
            variant: None,
            gain: 1.0,
            pitch: 1.0,
            reverb: true,
            occlusion: true,
        }
    }
}

fn physical(
    mut signals: MessageReader<Signal>,
    mut feed: MessageWriter<Request>,
    mut rng: Local<u32>,
    lab: Option<Res<Lab>>,
) {
    for signal in signals.read() {
        let family = match signal.cue {
            Cue::Walk => Family::Walk,
            Cue::Hurry => Family::Hurry,
            Cue::Approach => Family::Approach,
            Cue::Hand => Family::Hand,
            Cue::Foot => Family::Foot,
            Cue::Acceleration => Family::Launch,
            Cue::Brake => Family::Brake,
            Cue::Stop | Cue::Contact => Family::Stop,
            Cue::Breath | Cue::Ready => Family::Breath,
            Cue::Rupture => Family::Throat,
            Cue::Vocal => {
                if next(&mut rng) < 0.29 {
                    Family::Thin
                } else {
                    Family::Cry
                }
            }
            Cue::Retreat => Family::Moan,
        };
        let mut request = Request::new(family, Category::Physical, signal.at, signal.lab);
        request.pitch = 0.94 + 0.12 * next(&mut rng);
        request.gain = if signal.cue == Cue::Ready { 0.67 } else { 1.0 };
        if signal.lab
            && let Some(lab) = &lab
        {
            lab.tune_sound(&mut request);
        }
        feed.write(request);
    }
}

#[derive(Component)]
pub(crate) struct PresenceEmitter {
    pub request: Request,
    clock: (u64, u32),
    expires: f32,
    target: MixTarget,
    update_at: f32,
    pub transmission: f32,
    pub muffle: f32,
    pub audible: f32,
}

fn surface(at: Vec3) -> Surface {
    match station::sector_id(at) {
        Some(x)
            if [Sector::Filters, Sector::Tanks, Sector::Basin]
                .into_iter()
                .any(|sector| sector as u8 == x) =>
        {
            Surface::Wet
        }
        Some(x)
            if [
                Sector::Pumps,
                Sector::Generators,
                Sector::Electrical,
                Sector::Dock,
            ]
            .into_iter()
            .any(|sector| sector as u8 == x) =>
        {
            Surface::Metal
        }
        _ => Surface::Tile,
    }
}

fn connectivity(at: Vec3, listener: Vec3, closed: bool) -> f32 {
    let (Some(a), Some(b)) = (station::sector_id(at), station::sector_id(listener)) else {
        return 0.54;
    };
    if a == b {
        return 1.0;
    }
    let mut reached = 1u32 << a;
    for level in [0.80, 0.63, 0.47] {
        let mut expanded = reached;
        for portal in PORTALS {
            if closed && portal.rooms.contains(&Sector::Exit) {
                continue;
            }
            let [x, y] = portal.rooms;
            let ports = (1 << x as u8) | (1 << y as u8);
            if reached & ports != 0 {
                expanded |= ports;
            }
        }
        if expanded & (1 << b) != 0 {
            return level;
        }
        reached = expanded;
    }
    0.36
}

fn obstruction(blocks: usize, portal: f32) -> (f32, f32) {
    let transmission = [1.0, 0.51, 0.33, 0.24][blocks.min(3)] * portal;
    let muffle = ([0., 0.66, 0.82, 0.91][blocks.min(3)] + 0.21 * (1.0 - portal)).min(0.96);
    (transmission, muffle)
}

fn blocked(
    bounds: impl Iterator<Item = ainda_vivo_core::collision::Aabb>,
    at: Vec3,
    listener: Vec3,
) -> usize {
    let offset = at - listener;
    let limit = (offset.length() - 0.09).max(0.0);
    let ray = offset.normalize_or_zero();
    bounds
        .filter(|b| b.ray(listener, ray, limit).is_some_and(|d| d > 0.04))
        .take(3)
        .count()
}

#[derive(bevy::ecs::system::SystemParam)]
struct Space<'w, 's> {
    world: Res<'w, CollisionWorld>,
    lab: Option<Res<'w, Lab>>,
    game: Query<'w, 's, &'static Transform, With<MainCamera>>,
    camera: Query<'w, 's, &'static Transform, With<LabCamera>>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Independent audio assets, geometry and source state."
)]
fn mix_presence(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<Session>,
    comfort: Res<Comfort>,
    bank: Res<Bank>,
    space: Space,
    mut requests: MessageReader<Request>,
    mut voices: ResMut<Assets<Voice>>,
    mut sources: Query<(
        Entity,
        &mut PresenceEmitter,
        Option<&AudioSink>,
        &mut PlaybackSettings,
    )>,
    mut seed: Local<u32>,
) {
    let room = space.lab.as_ref().and_then(|lab| lab.audio_geometry());
    let lab_open = room.is_some();
    let listener = if lab_open {
        space.camera.single()
    } else {
        space.game.single()
    };
    let Ok(listener) = listener else {
        requests.clear();
        return;
    };
    let clock = (session.attempt, session.perception_revision);
    let now_lab = time.elapsed_secs();
    let closed = !session.progress().exit_unlocked();
    let volume = if comfort.ambience_muted {
        0.0
    } else {
        comfort.ambience_volume * comfort.psycho_av
    };
    let mut count = 0;
    let mut vocals = 0;
    let running = lab_open || session.screen == Screen::Playing;
    for (id, source, sink, mut settings) in &mut sources {
        let mut source = source;
        let now = if source.request.lab {
            now_lab
        } else {
            session.elapsed
        };
        if source.request.lab && !lab_open
            || !source.request.lab && source.clock != clock
            || now >= source.expires
            || !source.request.lab && matches!(session.screen, Screen::Menu | Screen::Complete)
        {
            commands.entity(id).despawn();
            continue;
        }
        let play = source.request.lab == lab_open && running;
        if let Some(sink) = sink {
            if sink.is_paused() == play {
                if play {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
        } else {
            settings.paused = !play;
        }
        if !play {
            continue;
        }
        count += 1;
        vocals += usize::from(source.request.family.vocal());
        if now >= source.update_at {
            source.update_at = now + 0.095;
            (source.transmission, source.muffle) =
                measure(&space, room, source.request, listener.translation, closed);
        }
    }
    let mut pending = [None; MAX_VOICES];
    let mut added = 0;
    for &request in requests.read() {
        if request.lab != lab_open
            || !running
            || volume <= 0.0001
            || count + added >= MAX_VOICES
            || request.family.vocal() && vocals >= 2
            || request.at.distance_squared(listener.translation) > RANGE * RANGE
        {
            continue;
        }
        pending[added] = Some(request);
        added += 1;
        vocals += usize::from(request.family.vocal());
    }
    let keep = count + added;
    for (_, mut source, _, _) in &mut sources {
        if source.request.lab != lab_open || !running {
            continue;
        }
        let gain = output_gain(source.request, volume, source.transmission, keep);
        let (l, r) = channels(
            source.request.at - listener.translation,
            *listener.right(),
            gain,
        );
        source.audible = l.max(r);
        source.target.set(l, r, source.muffle);
    }
    for request in pending.into_iter().flatten() {
        let variant = request
            .variant
            .unwrap_or_else(|| (next(&mut seed) * VARIANTS as f32) as usize);
        let sample = bank.get(
            request.family,
            variant,
            if request.lab {
                Surface::Tile
            } else {
                surface(request.at)
            },
        );
        let clip = if request.reverb {
            &sample.reflected
        } else {
            &sample.dry
        };
        let target = MixTarget::default();
        let pitch = request.pitch.clamp(0.65, 1.45);
        let (transmission, muffle) = measure(&space, room, request, listener.translation, closed);
        let gain = output_gain(request, volume, transmission, keep);
        let (l, r) = channels(request.at - listener.translation, *listener.right(), gain);
        target.set(l, r, muffle);
        let handle = voices.add(Voice {
            clip: clip.clone(),
            repeats: false,
            target: target.clone(),
        });
        let now = if request.lab {
            now_lab
        } else {
            session.elapsed
        };
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings {
                speed: pitch,
                ..PlaybackSettings::DESPAWN
            },
            PresenceEmitter {
                request,
                clock,
                expires: now + clip.len() as f32 / (RATE as f32 * pitch) + 0.12,
                update_at: now + 0.095,
                target,
                transmission,
                muffle,
                audible: l.max(r),
            },
        ));
    }
}

fn measure(
    space: &Space,
    room: Option<(
        &[ainda_vivo_core::collision::Aabb],
        Option<ainda_vivo_core::collision::Aabb>,
    )>,
    request: Request,
    listener: Vec3,
    closed: bool,
) -> (f32, f32) {
    if !request.occlusion {
        return (1.0, 0.0);
    }
    if let Some((solids, gate)) = room {
        obstruction(
            blocked(solids.iter().copied().chain(gate), request.at, listener),
            1.0,
        )
    } else {
        obstruction(
            blocked(space.world.active(!closed), request.at, listener),
            connectivity(request.at, listener, closed),
        )
    }
}

fn output_gain(request: Request, master: f32, transmission: f32, voices: usize) -> f32 {
    master * transmission * request.family.gain() * request.gain.clamp(0.0, 1.0)
        / (1.0 + 0.64 * voices.saturating_sub(1) as f32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    Knock,
    Breath,
    Trail,
    Rush,
    Far,
}
impl Pattern {
    pub fn label(self) -> &'static str {
        match self {
            Self::Knock => "Impacto e silêncio",
            Self::Breath => "Respiração oculta",
            Self::Trail => "Passos acompanhando",
            Self::Rush => "Caçada invisível",
            Self::Far => "Voz distante",
        }
    }
}

pub struct Echo {
    pub pattern: Pattern,
    pub category: Category,
    pub begin: [Vec3; 2],
    pub elapsed: f32,
    duration: f32,
    next: f32,
    beats: u8,
    ending: u8,
    rng: u32,
}

impl Echo {
    pub fn new(pattern: Pattern, category: Category, begin: [Vec3; 2], mut seed: u32) -> Self {
        let duration = match pattern {
            Pattern::Rush => 4.0 + next(&mut seed) * 2.8,
            Pattern::Trail => 3.6 + next(&mut seed) * 2.3,
            _ => 1.7 + next(&mut seed) * 1.5,
        };
        let ending = (next(&mut seed) * 3.0) as u8;
        Self {
            pattern,
            category,
            begin,
            elapsed: 0.0,
            duration,
            next: 0.12,
            beats: 0,
            ending,
            rng: seed,
        }
    }

    pub fn tick(
        &mut self,
        dt: f32,
        listener: Transform,
        speed: f32,
        lab: bool,
    ) -> (bool, Option<Request>) {
        self.elapsed += dt;
        let t = (self.elapsed / self.duration).clamp(0., 1.);
        if t >= 1.0 {
            return (true, None);
        }
        if self.elapsed < self.next {
            return (false, None);
        }
        let advance = if self.ending == 1 && t > 0.74 {
            1. - (t - 0.74) * 1.45
        } else {
            t * t * (3.0 - 2.0 * t)
        };
        let at = self.begin[0].lerp(self.begin[1], advance);
        let delta = at - listener.translation;
        if delta.length_squared() < 3.0 * 3.0 {
            return (true, None);
        }
        if self.pattern == Pattern::Breath
            && delta.normalize_or_zero().dot(*listener.forward()) > 0.87
        {
            return (true, None);
        }
        if self.pattern == Pattern::Trail && speed < 0.7 && !lab {
            self.next = self.elapsed + 0.18;
            return (false, None);
        }
        let step = self.beats;
        self.beats = self.beats.saturating_add(1);
        let family = match self.pattern {
            Pattern::Knock if step == 0 => Family::Stop,
            Pattern::Breath if step < 2 => Family::Breath,
            Pattern::Far if step == 0 => {
                if self.ending == 0 {
                    Family::Thin
                } else {
                    Family::Moan
                }
            }
            Pattern::Trail => {
                if step == 0 {
                    Family::Walk
                } else {
                    Family::Approach
                }
            }
            Pattern::Rush if step == 0 => Family::Walk,
            Pattern::Rush if t < 0.33 => Family::Hurry,
            Pattern::Rush if step == 6 && self.ending == 1 => Family::Cry,
            Pattern::Rush if matches!(step % 4, 0 | 3) => Family::Foot,
            Pattern::Rush => Family::Hand,
            _ => return (true, None),
        };
        if self.pattern == Pattern::Rush && self.ending == 2 && t > 0.85 {
            return (true, None);
        }
        let spacing = match self.pattern {
            Pattern::Knock | Pattern::Far => 8.0,
            Pattern::Breath => 1.45,
            Pattern::Trail => 0.49 + next(&mut self.rng) * 0.20,
            Pattern::Rush => (0.68 - 0.47 * t) * [1.11, 0.71, 0.91, 1.31][step as usize % 4],
        };
        self.next = self.elapsed + spacing;
        let mut sound = Request::new(family, self.category, at, lab);
        sound.pitch = 0.9 + next(&mut self.rng) * 0.20;
        sound.gain = (0.62 + 0.34 * advance).clamp(0., 1.);
        (false, Some(sound))
    }
}

#[derive(Default)]
struct Schedule {
    stamp: (u64, u32),
    after: f32,
    sample: f32,
    current: Option<Echo>,
    rng: u32,
    charged: bool,
}

fn candidate(
    world: &CollisionWorld,
    closed: bool,
    view: Transform,
    order: usize,
) -> Option<[Vec3; 2]> {
    let here = station::sector_id(view.translation)?;
    (0..PORTALS.len())
        .map(|i| PORTALS[(i + order) % PORTALS.len()])
        .find_map(|p| {
            let [_, side] = if p.rooms[0] as u8 == here {
                p.rooms
            } else if p.rooms[1] as u8 == here {
                [p.rooms[1], p.rooms[0]]
            } else {
                return None;
            };
            let room = get(side);
            let away = ((room.min + room.max) * 0.5 - p.at).normalize_or_zero();
            [4.3, 3.0, 2.2].into_iter().find_map(|length| {
                let from = p.at + away * length;
                let toward = p.at + away * 0.68;
                let a = Vec3::new(from.x, 0.63, from.y);
                let b = Vec3::new(toward.x, 0.63, toward.y);
                let offset = a - view.translation;
                (room.contains(from)
                    && (5.0..22.0).contains(&offset.length())
                    && (view.forward().dot(offset.normalize_or_zero()) < 0.67
                        || world.occludes(view.translation, a, !closed))
                    && !world.occludes(a, b, !closed))
                .then_some([a, b])
            })
        })
}

#[allow(
    clippy::too_many_arguments,
    reason = "Independent director and game resources."
)]
fn drive_illusions(
    mut state: Local<Schedule>,
    time: Res<Time>,
    mut session: ResMut<Session>,
    mut director: ResMut<Director>,
    (world, ambience, comfort): (Res<CollisionWorld>, Res<Atmosphere>, Res<Comfort>),
    (physical, old, smoke, threat): (
        Option<Res<Campaign>>,
        Res<Presence>,
        Res<crate::smoke::SmokeTest>,
        Res<crate::reality::Threat>,
    ),
    (player, camera): (
        Res<crate::app::LocalPlayer>,
        Query<&Transform, With<MainCamera>>,
    ),
    mut requests: MessageWriter<Request>,
) {
    if smoke.enabled && !smoke.sound {
        return;
    }
    let stamp = (session.attempt, session.perception_revision);
    if state.stamp != stamp {
        *state = Schedule {
            stamp,
            after: session.elapsed + 39.0,
            rng: director.initial_seed() ^ 0x819ec2ad,
            ..default()
        };
    }
    if session.screen != Screen::Playing {
        return;
    }
    let now = session.elapsed;
    if session.collapse_active()
        || session.resting
        || session.wake_confirmation
        || session.message_left > 1.0
        || ambience.quiet > 0.14
        || ambience.episode.holding_silence()
        || physical.is_some_and(|p| p.active())
        || old.active()
        || threat.active()
        || session.false_exit.is_some()
        || comfort.ambience_muted
        || comfort.psycho_av < 0.10
        || station::lamp_refuge(session.memory().desk_light, player.body.feet)
        || session.psychology.value() >= 77.0
    {
        state.current = None;
        return;
    }
    let Ok(view) = camera.single() else { return };
    if state.current.is_none() && now >= state.after && now >= state.sample {
        state.sample = now + 1.35;
        if director.allows(Beat::Sound, now) {
            let closed = !session.progress().exit_unlocked();
            if let Some(points) = candidate(&world, closed, *view, director.order) {
                let ghost = if session.expedition.observer().reality()
                    == ainda_vivo_core::reality::Reality::False
                {
                    Category::FalseReality
                } else {
                    Category::Illusion
                };
                let roll = next(&mut state.rng);
                let pattern = if roll < 0.37 {
                    Pattern::Rush
                } else if roll < 0.59 {
                    Pattern::Trail
                } else if roll < 0.76 {
                    Pattern::Breath
                } else if roll < 0.90 {
                    Pattern::Knock
                } else {
                    Pattern::Far
                };
                state.current = Some(Echo::new(pattern, ghost, points, state.rng));
                state.after = now + 92.0 + next(&mut state.rng) * 58.0;
                state.charged = false;
                director.claim(Beat::Sound, now);
            }
        }
    }
    if let Some(echo) = &mut state.current {
        let pattern = echo.pattern;
        let (end, pulse) = echo.tick(
            time.delta_secs(),
            *view,
            player.body.velocity.xz().length(),
            false,
        );
        if end {
            state.current = None;
            if pattern == Pattern::Rush && next(&mut state.rng) < 0.12 {
                director.request(Beat::Presence);
            }
        }
        if let Some(sound) = pulse {
            if !state.charged
                && matches!(pattern, Pattern::Rush | Pattern::Knock)
                && sound.at.distance(view.translation) < 9.5
                && !world.occludes(
                    view.translation,
                    sound.at,
                    session.progress().exit_unlocked(),
                )
                && comfort.ambience_volume * comfort.psycho_av > 0.2
            {
                session.stimulate(Stimulus::Noise, Origin::External, 0.22);
                state.charged = true;
            }
            requests.write(sound);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ainda_vivo_core::collision::Aabb;

    #[test]
    fn invisible_chase_is_reproducible_and_has_same_timbres_in_both_realities() {
        let run = |category| {
            let mut echo = Echo::new(
                Pattern::Rush,
                category,
                [Vec3::new(6., 0., -10.), Vec3::new(3., 0., -4.)],
                70,
            );
            let mut events = Vec::new();
            for tick in 0..500 {
                let (end, event) = echo.tick(1.0 / 60.0, Transform::IDENTITY, 2.0, false);
                if let Some(event) = event {
                    assert_eq!(event.category, category);
                    events.push((tick, event.family, event.at, event.pitch));
                }
                if end {
                    break;
                }
            }
            events
        };
        let events = run(Category::Illusion);
        assert_eq!(events, run(Category::FalseReality));
        assert_eq!(events[0].1, Family::Walk);
        assert!(events.iter().any(|e| e.1 == Family::Hand));
        assert!(events.iter().any(|e| e.1 == Family::Foot));
        assert!(events.last().unwrap().2.length() < events[0].2.length());
        assert!(events[1].0 - events[0].0 > events.last().unwrap().0 - events[events.len() - 2].0);
    }

    #[test]
    fn approach_can_fall_silent_and_gate_changes_spatial_response() {
        let points = [Vec3::new(0., 0., -9.), Vec3::new(0., 0., -4.)];
        let mut echo = Echo::new(Pattern::Breath, Category::Illusion, points, 70);
        assert!(echo.tick(0.13, Transform::IDENTITY, 1.0, false).0);
        let mut echo = Echo::new(Pattern::Trail, Category::Illusion, points, 70);
        assert!(echo.tick(0.25, Transform::IDENTITY, 0.0, false).1.is_none());
        assert!(echo.tick(0.25, Transform::IDENTITY, 1.5, false).1.is_some());
        let at = Vec3::new(-4., 0.7, -5.);
        let ear = Vec3::Y;
        let wall = Aabb::from_center_size(Vec3::new(-2., 1., -2.5), Vec3::new(5., 3., 0.2));
        let (closed, closed_muffle) = obstruction(blocked([wall].into_iter(), at, ear), 1.);
        let (open, open_muffle) = obstruction(blocked(std::iter::empty(), at, ear), 1.);
        assert!(closed < open && closed_muffle > open_muffle);
        let (l, r) = channels(at - ear, Vec3::X, open);
        let (far_l, _) = channels((at - ear) * 2., Vec3::X, open);
        assert!(l > r && far_l < l);
    }
}
