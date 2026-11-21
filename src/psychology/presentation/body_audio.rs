use super::smooth;
use crate::{
    app::{Comfort, MainCamera, Screen, Session},
    atmosphere::sound::Voice,
    psychology::Effects,
};
use bevy::{
    audio::{AudioSinkPlayback, Volume},
    ecs::system::SystemParam,
    prelude::*,
};
use std::{
    f32::consts::{PI, TAU},
    sync::Arc,
};

const SAMPLE_RATE: f32 = 24_000.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodySound {
    Breath,
    Heart,
}

#[derive(Component)]
struct PhantomSteps {
    attempt: u64,
    revision: u32,
    end: f32,
}

#[derive(Resource)]
pub struct StepClip(Arc<[f32]>);

pub fn setup(commands: &mut Commands, voices: &mut Assets<Voice>) {
    let breath = samples(8.32, 827413, |now, low, band| {
        let shifted = now >= 4.25;
        let at = if shifted { now - 4.25 } else { now };
        let inhale = envelope(at - 0.23, if shifted { 1.29 } else { 1.17 });
        let exhale = envelope(at - 1.75, if shifted { 1.52 } else { 1.72 });
        inhale * (band * 0.49 + low * 0.32) + exhale * (band * 0.29 + low * 0.40)
    });
    let heart = samples(1.08, 1129, |now, low, _| {
        let beat = |age, span, hz: f32| {
            envelope(age, span) * ((TAU * (hz * age - 22.0 * age * age)).sin() * 0.47 + low * 0.09)
        };
        beat(now - 0.036, 0.145, 70.0) + 0.64 * beat(now - 0.229, 0.104, 87.0)
    });
    for (kind, clip) in [(BodySound::Breath, breath), (BodySound::Heart, heart)] {
        let voice = Voice {
            clip,
            repeats: true,
            target: default(),
        };
        voice.target.set(0.707, 0.707, 0.0);
        commands.spawn((
            Name::new(match kind {
                BodySound::Breath => "Respiração do operador",
                BodySound::Heart => "Pulso do operador",
            }),
            kind,
            AudioPlayer(voices.add(voice)),
            PlaybackSettings::ONCE.paused().with_volume(Volume::SILENT),
        ));
    }
    commands.insert_resource(StepClip(samples(1.29, 8405, |now, low, band| {
        let step = |a: f32, hz: f32| {
            envelope(a, 0.19) * (low * 0.49 + (TAU * hz * a).sin() * 0.29)
                + envelope(a - 0.052, 0.14) * band * 0.22
        };
        step(now - 0.12, 81.0) + 0.79 * step(now - 0.69, 72.0)
    })));
}

fn envelope(age: f32, len: f32) -> f32 {
    if (0.0..len).contains(&age) {
        (PI * age / len).sin().powi(2)
    } else {
        0.0
    }
}

fn samples(seconds: f32, mut seed: u32, shape: impl Fn(f32, f32, f32) -> f32) -> Arc<[f32]> {
    let mut slow = 0.0;
    let mut fast = 0.0;
    (0..(SAMPLE_RATE * seconds).round() as u32)
        .map(|sample| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let white = (seed >> 8) as f32 / (1 << 23) as f32 - 1.0;
            slow += 0.053 * (white - slow);
            fast += 0.28 * (white - fast);
            shape(sample as f32 / SAMPLE_RATE, slow, fast - slow)
        })
        .collect()
}

#[derive(Default)]
pub struct BodyMix {
    rate: f32,
    pulse: f32,
    breath_gain: f32,
    heart_gain: f32,
    last_phantom: Option<Vec3>,
}

#[derive(SystemParam)]
pub struct Sinks<'w, 's> {
    body: Query<
        'w,
        's,
        (
            &'static BodySound,
            &'static mut PlaybackSettings,
            Option<&'static mut AudioSink>,
        ),
        With<BodySound>,
    >,
    phantom: Query<
        'w,
        's,
        (Entity, &'static PhantomSteps, Option<&'static AudioSink>),
        Without<BodySound>,
    >,
}

pub fn update(
    mut commands: Commands,
    (session, comfort, effects, time): (Res<Session>, Res<Comfort>, Res<Effects>, Res<Time>),
    mut voices: ResMut<Assets<Voice>>,
    clip: Res<StepClip>,
    cameras: Query<&Transform, With<MainCamera>>,
    mut sinks: Sinks,
    mut mix: Local<BodyMix>,
) {
    let playing = session.screen == Screen::Playing;
    let paused = session.screen == Screen::Paused;
    let settings = if comfort.ambience_muted {
        0.0
    } else {
        comfort.ambience_volume * comfort.psycho_av
    };
    let stress = effects.intensity;
    let critical = effects.critical;
    let terminal = effects
        .farewell
        .map_or(1.0, |t| 1.0 - smooth((t - 0.55) / 4.0));
    let blend = 1.0 - (-3.7 * time.delta_secs()).exp();
    if playing {
        let presence = smooth(stress / 0.78);
        let tremble = if effects.stage >= 4 {
            (session.elapsed * 1.92).sin() * (0.06 + 0.04 * critical)
        } else {
            0.0
        };
        mix.rate += (1.0 + 0.72 * presence + 0.25 * critical + tremble - mix.rate) * blend;
        mix.pulse += (1.0 + 0.64 * presence + 0.26 * critical - mix.pulse) * blend;
        let breath = (0.095 + presence * 0.38 + critical * 0.17) * terminal;
        let heartbeat = (0.35 * smooth((stress - 0.53) / 0.47) + 0.12 * critical) * terminal;
        mix.breath_gain += (breath - mix.breath_gain) * blend;
        mix.heart_gain += (heartbeat - mix.heart_gain) * blend;
    }
    for (kind, mut settings_component, sink) in &mut sinks.body {
        let (speed, gain) = match kind {
            BodySound::Breath => (mix.rate.max(0.9), mix.breath_gain),
            BodySound::Heart => (mix.pulse.max(0.9), mix.heart_gain),
        };
        let gain = gain * settings;
        let enabled = playing && gain > 0.0004;
        if let Some(mut sink) = sink {
            if enabled == sink.is_paused() {
                if enabled {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
            if enabled {
                sink.set_speed(speed);
                sink.set_volume(Volume::Linear(gain));
            }
        } else if settings_component.paused == enabled {
            settings_component.paused = !enabled;
        }
    }
    for (entity, cue, sink) in &sinks.phantom {
        if cue.attempt != session.attempt
            || cue.revision != session.perception_revision
            || (!playing && !paused)
            || session.elapsed >= cue.end
            || settings < 0.001
        {
            commands.entity(entity).despawn();
        } else if let Some(sink) = sink
            && sink.is_paused() == playing
        {
            if playing {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }
    if paused {
        return;
    }
    let next = playing.then_some(effects.hallucination).flatten();
    let previous = std::mem::replace(&mut mix.last_phantom, next);
    if let Some(position) = next.filter(|site| Some(*site) != previous)
        && settings > 0.001
        && effects.farewell.is_none()
        && let Ok(camera) = cameras.single()
    {
        let offset = position - camera.translation;
        let pan = offset
            .normalize_or_zero()
            .dot(*camera.right())
            .clamp(-0.82, 0.82);
        let gain = 0.37 * settings / (1.0 + (offset.length() / 8.0).powi(2));
        let voice = Voice {
            clip: clip.0.clone(),
            repeats: false,
            target: default(),
        };
        voice.target.set(
            gain * (0.5 * (1.0 - pan)).sqrt(),
            gain * (0.5 * (1.0 + pan)).sqrt(),
            0.37,
        );
        commands.spawn((
            AudioPlayer(voices.add(voice)),
            PlaybackSettings::DESPAWN,
            PhantomSteps {
                attempt: session.attempt,
                revision: session.perception_revision,
                end: session.elapsed + 1.8,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::audio::{Decodable, Source};

    #[test]
    fn generated_body_voices_are_non_silent_bounded_stereo_loops_with_silent_seams() {
        let mut app = App::new();
        app.init_resource::<Assets<Voice>>().add_systems(
            Startup,
            |mut commands: Commands, mut voices: ResMut<Assets<Voice>>| {
                setup(&mut commands, &mut voices);
            },
        );
        app.update();
        let world = app.world_mut();
        let mut players = world.query::<(&BodySound, &AudioPlayer<Voice>)>();
        let bank = world.resource::<Assets<Voice>>();
        let mut count = [0_u8; 2];
        for (kind, player) in players.iter(world) {
            let index = match kind {
                BodySound::Breath => 0,
                BodySound::Heart => 1,
            };
            count[index] += 1;
            let voice = bank.get(&player.0).expect("generated voice asset");
            let mut stream = voice.decoder();
            let frames = voice.clip.len();
            assert_eq!(stream.channels(), 2);
            assert_eq!(stream.sample_rate(), 24_000);
            assert!(stream.total_duration().is_none());
            let mut peak = [0.0_f32; 2];
            for frame in 0..2 * frames {
                let left = stream.next().expect("loop left channel");
                let right = stream.next().expect("loop right channel");
                assert!(left.is_finite() && right.is_finite(), "{kind:?}");
                assert!((left - right).abs() < 1e-6, "{kind:?} must be centred");
                peak[frame / frames] = peak[frame / frames].max(left.abs());
                if frame.is_multiple_of(frames) || frame % frames == frames - 1 {
                    assert!(left.abs().max(right.abs()) < 1e-5, "{kind:?} seam");
                }
            }
            assert!(
                peak.iter().all(|p| (0.02..0.45).contains(p)),
                "{kind:?} {peak:?}"
            );
        }
        assert_eq!(count, [1, 1]);
    }
}
