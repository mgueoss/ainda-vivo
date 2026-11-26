use super::spatial::RATE;
use bevy::prelude::*;
use std::{f32::consts::TAU, sync::Arc};

pub const VARIANTS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Family {
    Walk,
    Hurry,
    Approach,
    Hand,
    Foot,
    Launch,
    Brake,
    Stop,
    Breath,
    Throat,
    Moan,
    Cry,
    Thin,
}

impl Family {
    pub const ALL: [Self; 13] = [
        Self::Walk,
        Self::Hurry,
        Self::Approach,
        Self::Hand,
        Self::Foot,
        Self::Launch,
        Self::Brake,
        Self::Stop,
        Self::Breath,
        Self::Throat,
        Self::Moan,
        Self::Cry,
        Self::Thin,
    ];

    #[cfg(feature = "dev-lab")]
    pub fn label(self) -> &'static str {
        match self {
            Self::Walk => "Passada lenta",
            Self::Hurry => "Passada acelerada",
            Self::Approach => "Aproximação",
            Self::Hand => "Apoio de mão",
            Self::Foot => "Apoio de pé",
            Self::Launch => "Arrancada",
            Self::Brake => "Atrito de freada",
            Self::Stop => "Parada brusca",
            Self::Breath => "Respiração irregular",
            Self::Throat => "Garganta / ruptura",
            Self::Moan => "Gemido distante",
            Self::Cry => "Grito deformado",
            Self::Thin => "Vocalização aguda",
        }
    }

    pub fn vocal(self) -> bool {
        self as usize >= Self::Breath as usize
    }

    pub fn gain(self) -> f32 {
        match self {
            Self::Cry | Self::Thin => 0.42,
            Self::Breath | Self::Moan => 0.34,
            _ => 0.50,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Surface {
    Tile,
    Metal,
    Wet,
}

#[derive(Clone)]
pub struct Sample {
    pub dry: Arc<[f32]>,
    pub reflected: Arc<[f32]>,
}

#[derive(Resource)]
pub struct Bank(Vec<Sample>);

impl Default for Bank {
    fn default() -> Self {
        let mut clips = Vec::with_capacity(Family::ALL.len() * VARIANTS * 3);
        for family in Family::ALL {
            for i in 0..VARIANTS {
                for surface in [Surface::Tile, Surface::Metal, Surface::Wet] {
                    clips.push(synthesize(family, i, surface));
                }
            }
        }
        Self(clips)
    }
}

impl Bank {
    pub fn get(&self, family: Family, variant: usize, surface: Surface) -> &Sample {
        &self.0[(family as usize * VARIANTS + variant % VARIANTS) * 3 + surface as usize]
    }
}

pub fn next(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 8) as f32 / 16_777_216.0
}

fn synthesize(family: Family, variant: usize, surface: Surface) -> Sample {
    let mut seed = 1327 + family as u32 * 11_731 + variant as u32 * 919;
    let variation = variant as f32 - 1.0;
    let duration = match family {
        Family::Walk | Family::Hurry | Family::Approach | Family::Foot => 0.37,
        Family::Hand => 0.25,
        Family::Launch | Family::Brake | Family::Stop => 0.46,
        Family::Throat => 0.55,
        Family::Breath => 0.92,
        Family::Thin => 0.68,
        Family::Moan => 1.30,
        Family::Cry => 1.05,
    } * (1.0 + variation * 0.095);
    let count = (RATE as f32 * duration).round() as usize;
    let mut mono = Vec::with_capacity(count);
    let (mut phase, mut rough, mut air, mut drift, mut dc) =
        (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let base = match family {
        Family::Moan => 91.,
        Family::Throat => 74.,
        Family::Cry => 174.,
        Family::Thin => 254.,
        _ => 116.,
    };
    for n in 0..count {
        let t = n as f32 / RATE as f32;
        let unit = t / duration;
        let white = next(&mut seed) * 2.0 - 1.0;
        air += 0.28 * (white - air);
        rough += 0.046 * (white - rough);
        drift += 0.0028 * (white - drift);
        let wave = if family.vocal() {
            let cry = matches!(family, Family::Cry | Family::Thin);
            let pitch = base
                * (1.0 + variation * 0.09)
                * (1.0 + 0.18 * (TAU * unit * 1.36).sin() - 0.35 * unit + 0.30 * drift);
            phase = (phase + pitch / RATE as f32).fract();
            let formant = |freq: f32, width: f32, detune: f32| {
                let age = phase / pitch;
                (TAU * freq * (1.0 + detune * unit) * age).sin()
                    * (-age * width).exp()
                    * (1. - (-age * 1700.).exp())
            };
            let gate = (0.7
                + 0.22 * (TAU * unit * 2.45 + variation).sin()
                + 0.08 * (TAU * unit * 6.6).sin())
            .max(0.);
            let throat = (TAU * phase).sin() * 0.17
                + formant(530., 260., -0.27) * 0.53
                + formant(1180., 425., 0.24) * 0.38
                + formant(2260., 880., -0.19) * 0.13;
            let fracture = (TAU * (unit * 15.7 + drift) + variation).sin();
            let voicing = match family {
                Family::Breath => 0.082,
                Family::Throat => 0.48,
                Family::Moan => 0.73,
                _ => 0.86,
            };
            let attack = (unit / if cry { 0.16 } else { 0.23 }).min(1.0);
            let release = ((1. - unit) / 0.46).clamp(0., 1.);
            (throat * voicing * (0.87 + fracture * 0.13)
                + (air - rough * 0.6) * (1.0 - voicing * 0.62))
                * gate
                * attack
                * release
        } else {
            let (body, scrape, double) = match family {
                Family::Hand => (62., 0.36, 0.23),
                Family::Foot => (45., 0.19, 0.17),
                Family::Approach => (62., 0.31, 0.10),
                Family::Walk => (49., 0.19, 0.23),
                Family::Hurry => (52., 0.24, 0.15),
                Family::Launch => (46., 0.46, 0.19),
                Family::Brake => (58., 0.70, 0.20),
                _ => (42., 0.51, 0.26),
            };
            let thump = (TAU
                * (body * (1.0 + variation * 0.045) * t + 0.76 * (1.0 - (-t * 20.).exp())))
            .sin()
                * (-t * 21.).exp();
            let follow = (t - 0.038 - variant as f32 * 0.009).max(0.);
            let drag = (air * 0.24 + rough * 0.92)
                * scrape
                * (-t * 11.).exp()
                * (0.78 + 0.22 * (TAU * t * 37.).sin());
            let mode = match surface {
                Surface::Tile => 0.075 * (TAU * t * 232.).sin() * (-t * 22.).exp(),
                Surface::Metal => {
                    ((TAU * t * 312.).sin() * 0.13 + (TAU * t * 547.).sin() * 0.07)
                        * (-t * 16.).exp()
                }
                Surface::Wet => {
                    0.22 * (air - rough) * (-follow * 15.).exp() * (TAU * unit * 8.).sin().abs()
                }
            };
            (thump * 0.62
                + (TAU * follow * 88.).sin() * (-follow * 30.).exp() * double
                + drag
                + mode)
                * (1. - (-t * 425.).exp())
        };
        dc += 0.0036 * (wave - dc);
        let fade = ((1. - unit) * 20.).clamp(0., 1.);
        mono.push((((wave - dc) * 0.90).tanh() * fade).clamp(-0.78, 0.78));
    }
    let delays: [(usize, f32); 3] = match surface {
        Surface::Tile => [(679, 0.18), (1241, -0.105), (2123, 0.065)],
        Surface::Metal => [(907, 0.22), (1709, -0.14), (2951, 0.08)],
        Surface::Wet => [(1049, 0.19), (2143, -0.12), (3467, 0.08)],
    };
    let mut wet = vec![0.0; count + delays[2].0];
    let mut tail = 0.0;
    for (i, output) in wet.iter_mut().enumerate() {
        let echo = delays
            .iter()
            .filter_map(|&(d, g)| i.checked_sub(d).and_then(|j| mono.get(j)).map(|s| s * g))
            .sum::<f32>();
        tail += 0.23 * (echo - tail);
        *output = (mono.get(i).copied().unwrap_or(0.0) * 0.87 + tail).clamp(-0.78, 0.78);
    }
    Sample {
        dry: mono.into(),
        reflected: wet.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_bank_is_bounded_and_variants_are_distinct() {
        let bank = Bank::default();
        for f in Family::ALL {
            let a = bank.get(f, 0, Surface::Metal);
            let b = bank.get(f, 1, Surface::Metal);
            assert_ne!(a.dry, b.dry);
            assert!(a.reflected.len() > a.dry.len());
            for clip in [&a.dry, &a.reflected] {
                assert!(clip.iter().all(|v| v.is_finite() && v.abs() <= 0.79));
                assert!(clip.iter().map(|v| v * v).sum::<f32>() / clip.len() as f32 > 0.00001);
            }
        }
    }

    #[test]
    fn audition_pcm_uses_game_decoder() {
        use crate::atmosphere::sound::spatial::{MixTarget, Voice, channels};
        use bevy::audio::Decodable;
        let bank = Bank::default();
        let mut stereo = vec![0.0_f32; RATE as usize * 2 * 6];
        for (i, (second, family, x, z, muffle)) in [
            (0.10, Family::Walk, -3.0, 4.0, 0.66),
            (0.84, Family::Hurry, -2.7, 3.5, 0.60),
            (1.36, Family::Throat, -2.2, 3.0, 0.55),
            (2.05, Family::Launch, -1.8, 2.9, 0.24),
            (2.32, Family::Hand, -1.1, 2.7, 0.18),
            (2.49, Family::Foot, -0.4, 2.5, 0.09),
            (2.68, Family::Hand, 0.4, 2.3, 0.0),
            (2.77, Family::Cry, 0.6, 2.5, 0.0),
            (2.88, Family::Foot, 1.0, 2.2, 0.0),
            (3.12, Family::Hand, 1.3, 2.4, 0.0),
            (3.21, Family::Foot, 1.7, 2.7, 0.0),
            (3.46, Family::Brake, 2.0, 3.0, 0.11),
            (3.79, Family::Stop, 3.0, 3.55, 0.30),
            (4.52, Family::Moan, 4.8, 5.6, 0.65),
        ]
        .into_iter()
        .enumerate()
        {
            let target = MixTarget::default();
            let (l, r) = channels(
                Vec3::new(x, 0., z),
                Vec3::X,
                family.gain() * (1.0 - muffle * 0.4),
            );
            target.set(l, r, muffle);
            let voice = Voice {
                clip: bank
                    .get(family, i % VARIANTS, Surface::Metal)
                    .reflected
                    .clone(),
                repeats: false,
                target,
            };
            let first = (second * RATE as f32) as usize * 2;
            for (out, value) in stereo[first..].iter_mut().zip(voice.decoder()) {
                *out += value;
            }
        }
        assert!(stereo.iter().all(|v| v.is_finite() && v.abs() < 0.65));
        assert!(stereo.iter().any(|v| v.abs() > 0.04));
        if let Some(file) = std::env::var_os("AV_PRESENCE_DEMO") {
            use std::io::Write;
            let mut file = std::io::BufWriter::new(std::fs::File::create(file).unwrap());
            let len = stereo.len() as u32 * 2;
            file.write_all(b"RIFF").unwrap();
            file.write_all(&(36 + len).to_le_bytes()).unwrap();
            file.write_all(b"WAVEfmt \x10\0\0\0\x01\0\x02\0").unwrap();
            file.write_all(&RATE.to_le_bytes()).unwrap();
            file.write_all(&(RATE * 4).to_le_bytes()).unwrap();
            file.write_all(b"\x04\0\x10\0data").unwrap();
            file.write_all(&len.to_le_bytes()).unwrap();
            for v in stereo {
                file.write_all(&((v * 32767.0).round() as i16).to_le_bytes())
                    .unwrap();
            }
            file.flush().unwrap();
        }
    }
}
