use bevy::{
    audio::{Decodable, Source},
    prelude::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

pub const RATE: u32 = 24_000;

/// Send a coherent stereo/filter target from the frame loop to the audio thread.
/// The audio thread interpolates it, so a door edge never clicks in the mix.
#[derive(Clone, Default)]
pub struct MixTarget(Arc<AtomicU32>);

impl MixTarget {
    pub fn set(&self, left: f32, right: f32, muffle: f32) {
        let encode = |v: f32| (1023.0 * v.clamp(0.0, 1.0)).round() as u32;
        let packed = encode(left) | encode(right) << 10 | encode(muffle) << 20;
        self.0.store(packed, Ordering::Relaxed);
    }

    fn read(&self) -> [f32; 3] {
        let value = self.0.load(Ordering::Relaxed);
        [value, value >> 10, value >> 20].map(|bits| (bits & 1023) as f32 / 1023.0)
    }
}

#[derive(Asset, TypePath, Clone)]
pub struct Voice {
    pub clip: Arc<[f32]>,
    pub repeats: bool,
    pub target: MixTarget,
}

impl Decodable for Voice {
    type DecoderItem = f32;
    type Decoder = Stereo;

    fn decoder(&self) -> Stereo {
        Stereo {
            sound: self.clone(),
            frame: 0,
            right: None,
            filtered: 0.0,
            mix: [0.0; 3],
        }
    }
}

pub struct Stereo {
    sound: Voice,
    frame: usize,
    right: Option<f32>,
    filtered: f32,
    mix: [f32; 3],
}

impl Iterator for Stereo {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if let Some(right) = self.right.take() {
            return Some(right);
        }
        if self.sound.repeats && self.frame == self.sound.clip.len() {
            self.frame = 0;
        }
        let raw = *self.sound.clip.get(self.frame)?;
        self.frame += 1;
        self.filtered += 0.124 * (raw - self.filtered);
        for (current, target) in self.mix.iter_mut().zip(self.sound.target.read()) {
            *current += 0.008 * (target - *current);
        }
        let sample = raw + (self.filtered - raw) * self.mix[2];
        self.right = Some(sample * self.mix[1]);
        Some(sample * self.mix[0])
    }
}

impl Source for Stereo {
    fn current_frame_len(&self) -> Option<usize> {
        (!self.sound.repeats)
            .then(|| (self.sound.clip.len() - self.frame) * 2 + usize::from(self.right.is_some()))
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        (!self.sound.repeats)
            .then(|| Duration::from_secs_f64(self.sound.clip.len() as f64 / RATE as f64))
    }
}

/// Apply a distance rolloff and constant-power pan relative to the camera.
pub fn channels(offset: Vec3, right: Vec3, gain: f32) -> (f32, f32) {
    let distance = offset.length();
    let pan = (offset.with_y(0.0).normalize_or_zero().dot(right) * 0.84).clamp(-0.84, 0.84);
    let level = gain / (1.0 + (distance / 5.8).powi(2));
    (
        level * (0.5 * (1.0 - pan)).sqrt(),
        level * (0.5 * (1.0 + pan)).sqrt(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendered_stereo_follows_the_listeners_right_and_left() {
        let target = MixTarget::default();
        let (left, right) = channels(Vec3::NEG_X * 3.0, Vec3::X, 1.0);
        target.set(left, right, 0.0);
        let voice = Voice {
            clip: vec![0.3; 128].into(),
            repeats: true,
            target: target.clone(),
        };
        let mut stream = voice.decoder();
        let mut read_pair = || stream.by_ref().skip(800).take(2).collect::<Vec<_>>();
        let pair = read_pair();
        assert!(pair[0] > pair[1] * 2.0);
        let (left, right) = channels(Vec3::NEG_X * 3.0, Vec3::NEG_X, 1.0);
        target.set(left, right, 0.0);
        let pair = read_pair();
        assert!(pair[1] > pair[0] * 2.0);
    }
}
