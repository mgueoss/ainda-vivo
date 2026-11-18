use super::{Atmosphere, DUCT, WORKSHOP_ECHO, WORKSHOP_HUM};
use crate::app::{CollisionWorld, Comfort, LocalPlayer, MainCamera, Screen, Session};
use bevy::{audio::AudioSinkPlayback, prelude::*};
use std::{f32::consts::TAU, sync::Arc};

mod spatial;
pub use spatial::Voice;
use spatial::{MixTarget, RATE, channels};

#[derive(Resource)]
pub struct SoundBank {
    echo: Arc<[f32]>,
    shoes: [Arc<[f32]>; 3],
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
}

#[derive(Component)]
pub struct Emitter {
    sound: Sound,
    target: MixTarget,
    expiry: Option<(u64, f32)>,
}

impl Emitter {
    pub fn is_echo(&self) -> bool {
        matches!(self.sound, Sound::Echo)
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
        let blades = tone(t, 52.0) * 0.06 + tone(t, 104.0) * 0.032;
        (0.73 + 0.18 * tone(t, 7.0) + 0.09 * tone(t, 1.25))
            * (blades + noise.low * 0.53 + noise.band * 0.075)
    });
    let pumps = loop_clip(16.0, 449, |t, noise| {
        let beat = 0.64 + tone(t, 0.375) * 0.21 + tone(t, 0.625) * 0.15;
        beat * (tone(t, 53.0) * 0.069
            + tone(t, 106.0) * 0.038
            + tone(t, 211.75) * 0.019
            + noise.low * 0.46)
    });
    let drain = loop_clip(16.0, 982, |t, noise| {
        let air = (0.74 + tone(t, 0.4375) * 0.15) * (noise.low * 0.37 + noise.band * 0.128);
        let mut drops = 0.0;
        for (start, hz, gain) in [
            (2.29, 420.0, 0.073),
            (7.85, 381.0, 0.061),
            (12.51, 374.0, 0.076),
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
    commands.insert_resource(SoundBank { echo, shoes });
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
}

pub fn emit(
    mut commands: Commands,
    mut audio: ResMut<Assets<Voice>>,
    session: Res<Session>,
    scene: Res<Atmosphere>,
    player: Res<LocalPlayer>,
    bank: Res<SoundBank>,
    mut gait: Local<Gait>,
) {
    if gait.attempt != session.attempt {
        *gait = Gait {
            attempt: session.attempt,
            ..default()
        };
    }
    let feet = player.body.feet;
    let previous = gait.previous.replace(feet).unwrap_or(feet);
    if session.screen != Screen::Playing {
        return;
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
            expiry: Some((session.attempt, session.elapsed + lifetime)),
        },
    ));
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
    scene: Res<Atmosphere>,
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
        if let Some((attempt, ends_at)) = emitter.expiry
            && (attempt != session.attempt
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
                Sound::Room => 0.48 * (1.0 - 0.76 * scene.quiet) * (1.0 - 0.26 * scene.relief),
                Sound::Electricity => 0.77 * (1.0 - 0.90 * scene.quiet),
                Sound::AirDuct => 0.42 * (1.0 - 0.47 * scene.quiet) * (1.0 + 0.42 * scene.relief),
                Sound::PumpHall => 0.53 * (0.78 + scene.relief * 0.22),
                Sound::Water => 0.71,
                Sound::Echo => 0.72,
                Sound::Shoes(level) => level * (1.0 - 0.27 * scene.quiet),
            };
        if matches!(emitter.sound, Sound::Room) {
            let centre = gain * std::f32::consts::FRAC_1_SQRT_2;
            emitter.target.set(centre, centre, 0.0);
            continue;
        }
        let offset = transform.translation - listener.translation;
        let blocked = !matches!(emitter.sound, Sound::Shoes(_))
            && occluded(
                &world,
                listener.translation,
                offset,
                session.progress.power_restored,
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

fn occluded(world: &CollisionWorld, origin: Vec3, offset: Vec3, power: bool) -> bool {
    let distance = offset.length() - 0.07;
    let direction = offset.normalize_or_zero();
    distance > 0.0
        && world
            .0
            .iter()
            .filter(|solid| !solid.gate || !power)
            .any(|solid| solid.bounds.ray(origin, direction, distance).is_some())
}

#[derive(Clone, Copy)]
struct Noise {
    low: f32,
    band: f32,
}

fn tone(time: f32, hz: f32) -> f32 {
    (TAU * time * hz).sin()
}

fn shoe(time: f32, noise: Noise, hz: f32) -> f32 {
    if !(0.0..0.32).contains(&time) {
        return 0.0;
    }
    let attack = (time / 0.026).clamp(0.0, 1.0).powi(2);
    let tail = ((0.32 - time) / 0.07).min(1.0);
    let impact = (tone(time, hz) * 0.44 + tone(time, hz * 1.87) * 0.16) * (-23.0 * time).exp();
    let scuff = (noise.band * 0.74 + noise.low * 0.22) * (-12.5 * time).exp();
    attack * tail * (impact + scuff)
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
