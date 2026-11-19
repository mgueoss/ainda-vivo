use super::{
    Atmosphere, DUCT, PUMP_KNOCK, WORKSHOP_ECHO, WORKSHOP_HUM,
    director::{Cue, Director},
};
use crate::scene::presence::Presence;
use crate::{
    app::{CollisionWorld, Comfort, LocalPlayer, MainCamera, Screen, Session},
    reality::{Developer, Threat},
};
use ainda_vivo_core::story::Phase;
use bevy::{audio::AudioSinkPlayback, prelude::*};
use std::{f32::consts::TAU, sync::Arc};

mod incidents;
mod spatial;
pub use incidents::IncidentSchedule;
use incidents::Kind;
pub use spatial::Voice;
use spatial::{MixTarget, RATE, channels};

#[derive(Resource)]
pub struct SoundBank {
    echo: Arc<[f32]>,
    shoes: [Arc<[f32]>; 3],
    incidents: [[Arc<[f32]>; 3]; Kind::ALL.len()],
    pump_knocks: Arc<[f32]>,
    rail_scrape: Arc<[f32]>,
    contact: Arc<[f32]>,
    trailing: [Arc<[f32]>; 3],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EpisodeCue {
    Pump,
    Rail,
}

#[derive(Clone, Copy)]
enum Sound {
    Room,
    Electricity,
    AirDuct,
    PumpHall,
    Water,
    Echo,
    Shoes(f32),
    Incident(Kind),
    Episode(EpisodeCue),
    Contact,
    Trailing,
    Reception,
}

#[derive(Component)]
pub struct Emitter {
    sound: Sound,
    target: MixTarget,
    expiry: Option<(u64, u32, f32)>,
}

impl Emitter {
    pub fn is_echo(&self) -> bool {
        matches!(self.sound, Sound::Echo)
    }

    pub fn incident_kind(&self) -> Option<usize> {
        match self.sound {
            Sound::Incident(kind) => Some(kind as usize),
            _ => None,
        }
    }

    pub fn episode_cue(&self) -> Option<EpisodeCue> {
        match self.sound {
            Sound::Episode(cue) => Some(cue),
            _ => None,
        }
    }

    pub fn reality_sound(&self) -> u8 {
        match self.sound {
            Sound::Contact => 1,
            Sound::Reception => 2,
            Sound::Trailing => 4,
            _ => 0,
        }
    }
}

pub fn setup(mut commands: Commands, mut audio: ResMut<Assets<Voice>>) {
    let room = loop_clip(12.0, 413, |t, noise| {
        let current = tone(t, 47.0) * 0.032 + tone(t, 84.0) * 0.013;
        (0.82 + 0.18 * tone(t, 0.25)) * (noise.low * 0.42 + noise.band * 0.028 + current)
    });
    let lamp = loop_clip(8.0, 31, |t, noise| {
        let ballast = tone(t, 120.0) * 0.125 + tone(t, 240.0) * 0.065 + tone(t, 480.0) * 0.024;
        let sway = 0.80 + 0.13 * tone(t, 1.0) + 0.06 * tone(t, 2.625);
        ballast * sway + noise.band * 0.095
    });
    let ventilation = loop_clip(12.0, 191, |t, noise| {
        let blades = tone(t, 52.0) * 0.041 + tone(t, 103.75) * 0.021;
        (0.76 + 0.12 * tone(t, 6.83) + 0.11 * tone(t, 1.25))
            * (blades + noise.low * 0.42 + noise.band * 0.093)
    });
    let pumps = loop_clip(16.0, 449, |t, noise| {
        let beat = 0.64 + tone(t, 0.375) * 0.21 + tone(t, 0.625) * 0.15;
        beat * (tone(t, 53.0) * 0.058
            + tone(t, 106.0) * 0.038
            + tone(t, 211.75) * 0.019
            + noise.low * 0.42
            + noise.band * 0.026)
    });
    let drain = loop_clip(19.0, 982, |t, noise| {
        let air = (0.74 + tone(t, 0.4375) * 0.15) * (noise.low * 0.37 + noise.band * 0.128);
        let mut drops = 0.0;
        for (start, hz, gain) in [
            (2.29, 420.0, 0.073),
            (7.85, 381.0, 0.061),
            (12.51, 374.0, 0.076),
            (13.05, 378.0, 0.036),
            (17.27, 563.0, 0.042),
        ] {
            let age = t - start;
            if (0.0..0.48).contains(&age) {
                drops += gain
                    * (age / 0.025).min(1.0)
                    * (-15.0 * age).exp()
                    * tone(age, hz - age * 144.0);
            }
        }
        air + drops
    });
    let echo = samples(1.65, 731, |t, noise| {
        // Two weight shifts with an irregular gap and small early reflections.
        shoe(t - 0.025, noise, 96.0) * 0.87
            + shoe(t - 0.63, noise, 82.0) * 0.68
            + shoe(t - 0.11, noise, 96.0) * 0.14
            + shoe(t - 0.75, noise, 82.0) * 0.13
    })
    .into();
    let shoes = std::array::from_fn(|index| {
        samples(0.32, 91 + index as u32 * 41, |t, noise| {
            shoe(t, noise, 101.0 + index as f32 * 11.0)
        })
        .into()
    });
    let pump_knocks = samples(2.35, 1901, |t, noise| {
        hollow_knock(t - 0.03, noise, 118.0) * 0.83
            + hollow_knock(t - 0.49, noise, 108.0) * 0.70
            + hollow_knock(t - 1.13, noise, 129.0) * 0.95
            + hollow_knock(t - 1.34, noise, 97.0) * 0.16
    })
    .into();
    let rail_scrape = samples(2.75, 2017, |t, noise| {
        let scrape = (t / 0.36).min(1.0) * ((2.1 - t) / 0.84).clamp(0.0, 1.0);
        let friction = noise.band * 0.60
            + noise.low * 0.56
            + tone(t, 173.0 + 13.0 * t) * 0.070
            + tone(t, 431.0 - 23.0 * t) * 0.022;
        let drips = hollow_knock(t - 0.41, noise, 240.0) * 0.19
            + hollow_knock(t - 1.89, noise, 216.0) * 0.27;
        scrape * friction * (0.72 + 0.27 * tone(t, 8.1)) + drips
    })
    .into();
    let contact = samples(1.65, 807, |t, noise| {
        shoe(t - 0.05, noise, 81.0) * 0.50
            + hollow_knock(t - 0.12, noise, 109.0) * 0.49
            + shoe(t - 0.78, noise, 67.0) * 0.36
            + hollow_knock(t - 0.86, noise, 98.0) * 0.38
    })
    .into();
    let trailing = std::array::from_fn(|index| {
        samples(0.49, 1177 + index as u32 * 61, |t, noise| {
            shoe(t - 0.02, noise, 73.0 + index as f32 * 9.0) * 0.57
                + hollow_knock(t - 0.12, noise, 138.0 - index as f32 * 7.0) * 0.16
        })
        .into()
    });
    commands.insert_resource(SoundBank {
        echo,
        shoes,
        incidents: Kind::ALL.map(|kind| std::array::from_fn(|index| kind.clip(index))),
        pump_knocks,
        rail_scrape,
        contact,
        trailing,
    });
    for (kind, clip, position) in [
        (Sound::Room, room, None),
        (Sound::Electricity, lamp, Some(WORKSHOP_HUM)),
        (Sound::AirDuct, ventilation, Some(DUCT)),
        (
            Sound::PumpHall,
            pumps.clone(),
            Some(Vec3::new(-5.8, 1.68, -33.0)),
        ),
        (Sound::PumpHall, pumps, Some(Vec3::new(6.1, 1.83, -43.6))),
        (Sound::Water, drain, Some(Vec3::new(3.65, 0.95, -72.15))),
    ] {
        let target = MixTarget::default();
        let voice = audio.add(Voice {
            clip,
            repeats: true,
            target: target.clone(),
        });
        commands.spawn((
            AudioPlayer(voice),
            PlaybackSettings::ONCE.paused(),
            Emitter {
                sound: kind,
                target,
                expiry: None,
            },
            Transform::from_translation(position.unwrap_or_default()),
        ));
    }
}

#[derive(Default)]
pub struct Gait {
    previous: Option<Vec3>,
    distance: f32,
    index: usize,
    attempt: u64,
    revision: u32,
    reception_played: bool,
}

pub fn emit(
    mut commands: Commands,
    mut audio: ResMut<Assets<Voice>>,
    session: Res<Session>,
    (scene, threat): (Res<Atmosphere>, Res<Threat>),
    player: Res<LocalPlayer>,
    bank: Res<SoundBank>,
    mut gait: Local<Gait>,
) {
    if (gait.attempt, gait.revision) != (session.attempt, session.perception_revision) {
        *gait = Gait {
            attempt: session.attempt,
            revision: session.perception_revision,
            ..default()
        };
    }
    let feet = player.body.feet;
    let previous = gait.previous.replace(feet).unwrap_or(feet);
    if session.screen != Screen::Playing {
        return;
    }
    if let Some(position) = threat.warning {
        one_shot(
            &mut commands,
            &mut audio,
            &session,
            bank.contact.clone(),
            Sound::Contact,
            position + Vec3::Y * 0.27,
            2.0,
        );
    }
    if !gait.reception_played && crate::reality::exit_line(&session) {
        gait.reception_played = true;
        one_shot(
            &mut commands,
            &mut audio,
            &session,
            bank.echo.clone(),
            Sound::Reception,
            Vec3::new(-1.45, 0.21, -91.6),
            2.0,
        );
    }
    if scene.play_echo {
        one_shot(
            &mut commands,
            &mut audio,
            &session,
            bank.echo.clone(),
            Sound::Echo,
            WORKSHOP_ECHO,
            2.0,
        );
    }
    if scene.episode.play_pump {
        one_shot(
            &mut commands,
            &mut audio,
            &session,
            bank.pump_knocks.clone(),
            Sound::Episode(EpisodeCue::Pump),
            PUMP_KNOCK,
            2.8,
        );
    }
    if scene.episode.play_rail
        && let Some(trace) = scene.episode.trace
    {
        one_shot(
            &mut commands,
            &mut audio,
            &session,
            bank.rail_scrape.clone(),
            Sound::Episode(EpisodeCue::Rail),
            trace.sound_position(),
            3.0,
        );
    }
    let travelled = (feet - previous).with_y(0.0).length();
    if !player.body.grounded || travelled > 0.70 {
        gait.distance = 0.0;
        return;
    }
    let speed = player.body.velocity.with_y(0.0).length();
    let (stride, level) = if player.body.crouched {
        (0.79, 0.10)
    } else if speed > 3.0 {
        (1.32, 0.31)
    } else {
        (1.03, 0.22)
    };
    gait.distance += travelled;
    if gait.distance >= stride {
        gait.distance -= stride;
        let side = if gait.index.is_multiple_of(2) {
            -0.14
        } else {
            0.14
        };
        let offset = Quat::from_rotation_y(player.yaw) * Vec3::new(side, 0.11, -0.15);
        one_shot(
            &mut commands,
            &mut audio,
            &session,
            bank.shoes[gait.index % 3].clone(),
            Sound::Shoes(level),
            feet + offset,
            0.65,
        );
        gait.index = gait.index.wrapping_add(1);
    }
}

fn one_shot(
    commands: &mut Commands,
    audio: &mut Assets<Voice>,
    session: &Session,
    clip: Arc<[f32]>,
    sound: Sound,
    position: Vec3,
    lifetime: f32,
) {
    let target = MixTarget::default();
    let voice = audio.add(Voice {
        clip,
        repeats: false,
        target: target.clone(),
    });
    commands.spawn((
        AudioPlayer(voice),
        PlaybackSettings::DESPAWN,
        Transform::from_translation(position),
        Emitter {
            sound,
            target,
            expiry: Some((
                session.attempt,
                session.perception_revision,
                session.elapsed + lifetime,
            )),
        },
    ));
}

#[derive(Default)]
pub struct Trail {
    until: f32,
    next_step: f32,
    index: usize,
    attempt: u64,
    revision: u32,
}

impl Trail {
    fn active(&self, now: f32) -> bool {
        now < self.until
    }

    fn footfall(&mut self, now: f32, feet: Vec3, velocity: Vec3) -> Option<(usize, Vec3)> {
        let velocity = velocity.with_y(0.0);
        let speed = velocity.length();
        if !self.active(now) || now < self.next_step || speed < 1.65 {
            return None;
        }
        let heading = velocity / speed;
        let side = Vec3::Y.cross(heading);
        let sign = if self.index.is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        let position = feet - heading * (3.4 + speed * 0.42) + side * sign * 0.48 + Vec3::Y * 0.15;
        self.next_step = now
            + if self.index.is_multiple_of(2) {
                0.62
            } else {
                0.91
            };
        self.index += 1;
        Some((self.index % 3, position))
    }
}

#[allow(clippy::too_many_arguments, reason = "Bevy system parameters.")]
pub fn emit_incidents(
    mut commands: Commands,
    mut audio: ResMut<Assets<Voice>>,
    time: Res<Time>,
    session: Res<Session>,
    atmosphere: Res<Atmosphere>,
    (presence, threat): (Res<Presence>, Res<Threat>),
    listener: Query<&Transform, With<MainCamera>>,
    bank: Res<SoundBank>,
    mut schedule: Local<IncidentSchedule>,
    mut director: ResMut<Director>,
    developer: Res<Developer>,
    player: Res<LocalPlayer>,
    mut trail: Local<Trail>,
) {
    let seed = developer
        .seed
        .map_or(time.elapsed().as_millis() as u32, |seed| {
            seed ^ session.attempt as u32 ^ 1
        });
    schedule.reset(session.attempt, seed);
    if (trail.attempt, trail.revision) != (session.attempt, session.perception_revision) {
        *trail = Trail {
            attempt: session.attempt,
            revision: session.perception_revision,
            ..default()
        };
    }
    if session.screen != Screen::Playing {
        return;
    }
    if atmosphere.quiet > 0.14
        || atmosphere.episode.holding_silence()
        || presence.active()
        || threat.active()
        || session.false_exit.is_some()
    {
        trail.until = 0.0;
        schedule.defer_until(session.elapsed + 2.0);
        return;
    }
    let now = session.elapsed;
    let speed = player.body.velocity.xz().length();
    let Ok(camera) = listener.single() else {
        return;
    };
    if director.allows(Cue::Sound, now)
        && session.progress().phase() >= Phase::Electrical
        && (1.85..3.0).contains(&speed)
        && player.body.grounded
        && director
            .order
            .is_multiple_of(if session.expedition.observer().distortion_ready(now) {
                2
            } else {
                4
            })
        && player
            .body
            .velocity
            .normalize_or_zero()
            .dot(*camera.forward())
            > 0.72
    {
        trail.until = now + 3.1;
        trail.next_step = now + 0.24;
        trail.index = director.order % 3;
        director.claim(Cue::Sound, now);
        schedule.defer_until(now + 11.0);
    }
    if trail.active(now) {
        if let Some((variant, position)) =
            trail.footfall(now, player.body.feet, player.body.velocity)
        {
            one_shot(
                &mut commands,
                &mut audio,
                &session,
                bank.trailing[variant].clone(),
                Sound::Trailing,
                position,
                0.85,
            );
        }
        return;
    }
    if director.allows(Cue::Sound, now)
        && let Some((kind, position)) = schedule.next(now, camera)
    {
        one_shot(
            &mut commands,
            &mut audio,
            &session,
            bank.incidents[kind as usize][director.order % 3].clone(),
            Sound::Incident(kind),
            position,
            kind.duration() + 0.45,
        );
        director.claim(Cue::Sound, now);
    }
}

type AudioEmitters<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static Emitter,
        &'static mut PlaybackSettings,
        Option<&'static AudioSink>,
    ),
    Without<MainCamera>,
>;

pub fn mix(
    mut commands: Commands,
    session: Res<Session>,
    (scene, director): (Res<Atmosphere>, Res<Director>),
    comfort: Res<Comfort>,
    world: Res<CollisionWorld>,
    listener: Query<&Transform, With<MainCamera>>,
    mut sources: AudioEmitters,
) {
    let Ok(listener) = listener.single() else {
        return;
    };
    let playing = session.screen == Screen::Playing;
    let volume = if comfort.ambience_muted {
        0.0
    } else {
        comfort.ambience_volume
    };
    for (entity, transform, emitter, mut settings, sink) in &mut sources {
        if let Some((attempt, revision, ends_at)) = emitter.expiry
            && (attempt != session.attempt
                || revision != session.perception_revision
                || (sink.is_none() && session.elapsed >= ends_at)
                || session.screen == Screen::Complete)
        {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(sink) = sink {
            if sink.is_paused() == playing {
                if playing {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
        } else {
            settings.paused = !playing;
        }
        if !playing {
            continue;
        }
        let gain = volume
            * match emitter.sound {
                Sound::Room => {
                    0.48 * (1.0 - 0.76 * scene.quiet)
                        * (1.0 - 0.26 * scene.relief)
                        * (1.0 - 0.23 * scene.receipt)
                        * (1.0 - 0.65 * director.hush)
                        * (1.0 - 0.70 * scene.episode.pump_hush.max(scene.episode.rail_hush))
                }
                Sound::Electricity => 0.77 * (1.0 - 0.90 * scene.quiet),
                Sound::AirDuct => {
                    0.42 * (1.0 - 0.47 * scene.quiet)
                        * (1.0 + 0.42 * scene.relief)
                        * (1.0 - 0.32 * director.hush)
                }
                Sound::PumpHall => {
                    0.53 * (0.78 + scene.relief * 0.22) * (1.0 - 0.91 * scene.episode.pump_hush)
                }
                Sound::Water => 0.71 * (1.0 - 0.76 * scene.episode.rail_hush),
                Sound::Echo => 0.72,
                Sound::Shoes(level) => level * (1.0 - 0.27 * scene.quiet),
                Sound::Incident(kind) => kind.gain() * (1.0 - 0.18 * scene.relief),
                Sound::Episode(EpisodeCue::Pump) => 0.94,
                Sound::Episode(EpisodeCue::Rail) => 0.82,
                Sound::Contact => 0.66,
                Sound::Trailing => 0.47,
                Sound::Reception => 0.47,
            };
        if matches!(emitter.sound, Sound::Room) {
            let centre = gain * std::f32::consts::FRAC_1_SQRT_2;
            emitter.target.set(centre, centre, 0.0);
            continue;
        }
        let mut source = transform.translation;
        if matches!(emitter.sound, Sound::Episode(EpisodeCue::Rail))
            && let Some(trace) = scene.episode.trace
        {
            source.z -= ((trace.age - 0.80) / 1.75).clamp(0.0, 1.0) * 2.6;
        }
        let offset = source - listener.translation;
        let blocked = !matches!(emitter.sound, Sound::Shoes(_))
            && world.occludes(
                listener.translation,
                source,
                session.progress().exit_unlocked(),
            );
        let (left, right) = channels(
            offset,
            *listener.right(),
            gain * if blocked { 0.55 } else { 1.0 },
        );
        emitter
            .target
            .set(left, right, if blocked { 0.74 } else { 0.0 });
    }
}

#[cfg(test)]
mod trail_tests {
    use super::*;

    #[test]
    fn a_following_footfall_stays_behind_motion_and_stops_when_stationary() {
        let mut trail = Trail {
            until: 12.0,
            next_step: 9.0,
            ..default()
        };
        assert!(
            trail
                .footfall(8.99, Vec3::ZERO, Vec3::NEG_Z * 2.5)
                .is_none()
        );
        let (_, source) = trail.footfall(9.0, Vec3::ZERO, Vec3::NEG_Z * 2.5).unwrap();
        assert!(source.z > 3.0);
        assert!(trail.footfall(10.0, Vec3::ZERO, Vec3::ZERO).is_none());
        assert!(
            trail
                .footfall(12.01, Vec3::ZERO, Vec3::NEG_Z * 2.5)
                .is_none()
        );
    }
}

#[derive(Clone, Copy)]
struct Noise {
    low: f32,
    band: f32,
}

fn tone(time: f32, hz: f32) -> f32 {
    (TAU * time * hz).sin()
}

fn hollow_knock(age: f32, noise: Noise, hz: f32) -> f32 {
    if !(0.0..0.65).contains(&age) {
        return 0.0;
    }
    let onset = (age / 0.028).min(1.0);
    let body = tone(age, hz) * 0.24
        + tone(age, hz * 1.79) * 0.115
        + tone(age, hz * 2.87) * 0.054
        + noise.band * 0.37;
    onset * (-10.0 * age).exp() * body
}

fn shoe(time: f32, noise: Noise, hz: f32) -> f32 {
    if !(0.0..0.32).contains(&time) {
        return 0.0;
    }
    let attack = (time / 0.026).clamp(0.0, 1.0).powi(2);
    let tail = ((0.32 - time) / 0.07).min(1.0);
    let impact =
        (tone(time, hz - time * 64.0) * 0.33 + tone(time, hz * 1.82) * 0.084 + noise.low * 0.21)
            * (-23.0 * time).exp();
    let scuff = (noise.band * 0.65 + noise.low * 0.23) * (-13.0 * time).exp();
    let roll = ((time - 0.058) / 0.075).clamp(0.0, 1.0);
    attack * tail * (impact + scuff + roll * noise.band * (-9.0 * time).exp() * 0.15)
}

fn samples(seconds: f32, mut seed: u32, mut shape: impl FnMut(f32, Noise) -> f32) -> Vec<f32> {
    let mut low = 0.0;
    let mut high = 0.0;
    (0..(seconds * RATE as f32).round() as u32)
        .map(|i| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let white = (seed >> 8) as f32 / 8_388_608.0 - 1.0;
            low += 0.023 * (white - low);
            high += 0.22 * (white - high);
            shape(
                i as f32 / RATE as f32,
                Noise {
                    low,
                    band: high - low,
                },
            )
            .clamp(-0.65, 0.65)
        })
        .collect()
}

fn loop_clip(seconds: f32, seed: u32, shape: impl FnMut(f32, Noise) -> f32) -> Arc<[f32]> {
    const OVERLAP: usize = 1920;
    let len = (seconds * RATE as f32).round() as usize;
    let mut wave = samples(seconds + OVERLAP as f32 / RATE as f32, seed, shape);
    // Crossfade the noise floor around the seam. The pitched components repeat in phase.
    for index in 0..OVERLAP {
        let mix = index as f32 / OVERLAP as f32;
        wave[index] = wave[len + index] * (1.0 - mix) + wave[index] * mix;
    }
    wave.truncate(len);
    wave.into()
}
