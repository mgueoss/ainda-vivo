use super::{
    Brain, Frame, HUMAN, Nav, Noise, Portal, State, Tuning, View, Zone,
    brain::STEP,
    hunt::{self, End, Hunt, Phase, Strategy},
    point,
};
use ainda_vivo_core::collision::Aabb;
use bevy::prelude::*;
use std::{fmt::Write, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BotStyle {
    Side,
    Reverse,
    Circle,
}

impl BotStyle {
    pub(in crate::lab) const ALL: [Self; 3] = [Self::Side, Self::Reverse, Self::Circle];

    pub(in crate::lab) fn label(self) -> &'static str {
        match self {
            Self::Side => "Desvio",
            Self::Reverse => "Reversão",
            Self::Circle => "Circuito",
        }
    }

    fn offset(self, t: f32, radius: f32) -> Vec2 {
        match self {
            Self::Side => Vec2::X * (t * 4.20).min(radius),
            Self::Reverse => Vec2::X * (t * 1.48).sin() * radius + Vec2::Y * (0.74 * t).min(4.1),
            Self::Circle => Vec2::new((t * 0.96).sin(), (t * 0.96).cos() - 1.0) * radius,
        }
    }
}

pub(super) struct Bot {
    pub style: BotStyle,
    pub at: Vec2,
    origin: Vec2,
    clock: f32,
    sign: f32,
}

impl Bot {
    pub(super) fn new(style: BotStyle, origin: Vec2, seed: u32) -> Self {
        let sign = if style == BotStyle::Side || seed.is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        Self {
            style,
            at: origin,
            origin,
            clock: 0.0,
            sign,
        }
    }

    pub(super) fn tick(&mut self, dt: f32, nav: &Nav, closed: bool, running: bool) {
        if !running {
            return;
        }
        self.clock += dt;
        let mut offset = self.style.offset(self.clock, 1.44);
        offset.x *= self.sign;
        let desired = self.origin + offset;
        let next = self.at + (desired - self.at).clamp_length_max(dt * 4.55);
        if nav.clear_body(self.at, next, Vec3::new(0.28, 0.9, 0.28), closed) {
            self.at = next;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Case {
    Side,
    Reverse,
    Circle,
    Reopen,
    Hide,
    Detour,
    Distraction,
    Interrupted,
    Long,
}

impl Case {
    const ALL: [Self; 9] = [
        Self::Side,
        Self::Reverse,
        Self::Circle,
        Self::Reopen,
        Self::Hide,
        Self::Detour,
        Self::Distraction,
        Self::Interrupted,
        Self::Long,
    ];

    fn duration(self) -> f32 {
        if self == Self::Long { 180.0 } else { 14.0 }
    }

    fn unarmed(self) -> bool {
        matches!(self, Self::Distraction | Self::Long)
    }

    fn is_door(self) -> bool {
        matches!(self, Self::Reopen | Self::Hide)
    }

    fn target(self, dt: f32, run: f32, side: f32) -> Vec2 {
        let base = Vec2::new(0.0, 2.8);
        match self {
            Self::Side => base + BotStyle::Side.offset(run, 2.12),
            Self::Reverse => base + BotStyle::Reverse.offset(run, 2.50) * Vec2::new(side, 1.0),
            Self::Circle => base + BotStyle::Circle.offset(run, 3.15) * Vec2::new(side, 1.0),
            Self::Detour => Vec2::new((3.10 - run * 2.6).max(0.30), 3.4),
            Self::Distraction => Vec2::new(0., -14.1),
            Self::Long => Vec2::new(5.5 * (dt * 0.83).sin(), -1.0 + 6.3 * (dt * 0.51).cos()),
            _ => base,
        }
    }
}

fn box_at(position: Vec2, width: f32, depth: f32) -> Aabb {
    Aabb::from_center_size(point(position).with_y(1.4), Vec3::new(width, 2.8, depth))
}

fn fixture(case: Case) -> Nav {
    let mut zones = vec![Zone {
        min: Vec2::new(-8., -16.),
        max: Vec2::new(8., 11.),
    }];
    let mut portals = Vec::new();
    if case.is_door() {
        zones[0].max.y = 0.0;
        zones.push(Zone {
            min: Vec2::new(-8., 0.),
            max: Vec2::new(8., 11.),
        });
        portals.push(Portal {
            rooms: [0, 1],
            at: Vec2::ZERO,
            width: 15.8,
            height: 3.,
        });
    }
    let solids = match case {
        Case::Side => vec![box_at(Vec2::new(2.70, 3.), 0.55, 6.)],
        Case::Detour => vec![box_at(Vec2::new(0., -0.3), 2.4, 0.54)],
        _ => Vec::new(),
    };
    Nav::new(
        zones,
        portals,
        solids,
        case.is_door().then(|| box_at(Vec2::ZERO, 16., 0.24)),
    )
}

#[derive(Debug, PartialEq)]
struct ResultRow {
    end: Option<End>,
    contacts: u32,
    reacquired: u32,
    searches: bool,
    started: bool,
    vision_ms: Option<u32>,
    noise_ms: Option<u32>,
    noises: u32,
    bad_sweeps: u32,
    metres: f32,
    decisions: u32,
    repeats: u32,
    replans: u32,
    blocks: u32,
    recovered: u32,
    nodes: usize,
    budget_hits: u32,
    bytes: usize,
    updates: u32,
}

fn simulate(case: Case, seed: u32, fps: u32) -> ResultRow {
    let nav = fixture(case);
    let mut brain = Brain::seeded(seed);
    let side = if seed.is_multiple_of(2) { 1.0 } else { -1.0 };
    brain.spawn(Vec2::new(0., -10.), 0.0);
    let initial = case.target(0., 0., side);
    if !case.unarmed() {
        brain.hunt = Hunt::start(
            hunt::Tuning {
                prepare: 1.50,
                ..default()
            },
            initial,
            false,
        );
    }
    let mut result = ResultRow {
        end: None,
        contacts: 0,
        reacquired: 0,
        searches: false,
        started: false,
        vision_ms: None,
        noise_ms: None,
        noises: 0,
        bad_sweeps: 0,
        metres: 0.0,
        decisions: 0,
        repeats: 0,
        replans: 0,
        blocks: 0,
        recovered: 0,
        nodes: 0,
        budget_hits: 0,
        bytes: 0,
        updates: 0,
    };
    let dt = 1.0 / fps as f32;
    let mut run_start = None;
    let mut ordinary_decision = None;
    let mut time_entered = brain.entered;
    let mut closed_for_good = false;
    let mut pulse_sent = false;
    let anchors = [
        Vec2::new(-4.2, -3.4),
        Vec2::new(4.2, 5.4),
        Vec2::new(5.2, -12.5),
    ];
    for tick in 0..(case.duration() * fps as f32) as u32 {
        let now = (tick + 1) as f32 * dt;
        let run = brain.hunt.phase == Phase::Run;
        if run {
            run_start.get_or_insert(now);
        }
        let age = run_start.map_or(0.0, |start| now - start);
        if case == Case::Hide && run && age >= 0.28 {
            closed_for_good = true;
        }
        let closed = closed_for_good || case == Case::Reopen && (0.29..1.49).contains(&age);
        if case == Case::Interrupted && run && age > 0.62 {
            brain.hunt.stop(End::Interrupted);
        }
        if case == Case::Long
            && brain.state == State::Absent
            && brain.clock >= brain.next_manifest
            && tick % fps == 0
        {
            brain.spawn(Vec2::new(side * 2.2, -9.5), 0.0);
        }
        let mut pulse = None;
        if case == Case::Distraction && !pulse_sent && now >= 1.75 {
            pulse_sent = true;
            pulse = Some(Noise {
                at: Vec2::new(1.75, -5.2),
                power: 4.1,
            });
        } else if case == Case::Long && tick % (fps * 9) == 0 {
            pulse = Some(Noise {
                at: anchors[tick as usize / (fps as usize * 9) % 3],
                power: 2.7,
            });
        }
        let frame = Frame {
            nav: &nav,
            door_closed: closed,
            points: &anchors,
            noises: pulse.as_slice(),
            view: View {
                eye: point(case.target(now, age, side)).with_y(1.62),
                facing: Vec3::Z,
                protected: false,
            },
        };
        let from = brain.pos;
        let manifest_before = brain.born;
        brain.step(dt, &frame, Tuning::default(), true);
        result.updates += 1;
        result.started |= brain.hunt.phase == Phase::Run;
        result.searches |= matches!(
            brain.hunt.strategy,
            Strategy::LocalSearch | Strategy::SectorSearch
        ) || matches!(brain.state, State::Investigate | State::Follow);
        if brain.visible && result.vision_ms.is_none() {
            result.vision_ms = Some((brain.clock * 1000.0).round() as u32);
        }
        if case == Case::Distraction
            && result.noise_ms.is_none()
            && brain.state == State::Investigate
        {
            result.noise_ms = Some(((brain.clock - 1.75).max(0.0) * 1000.0).round() as u32);
        }
        if brain.entered != time_entered {
            result.decisions += 1;
            result.repeats += u32::from(ordinary_decision == Some(brain.state));
            ordinary_decision = Some(brain.state);
            time_entered = brain.entered;
        }
        let moved = from.distance(brain.pos);
        if moved > 0.0001 && brain.born == manifest_before {
            result.metres += moved;
            if moved > 5.15 * dt.max(STEP) + 0.015
                || !nav.clear_body(from, brain.pos, HUMAN, closed)
            {
                result.bad_sweeps += 1;
            }
        }
        result.contacts += u32::from(std::mem::take(&mut brain.hunt.pending));
        if brain.hunt.finished {
            break;
        }
    }
    result.end = brain.hunt.end;
    result.reacquired = brain.memory.reacquired;
    result.noises = brain.memory.noises;
    result.replans = brain.memory.replans;
    result.blocks = brain.memory.blocked;
    result.recovered = brain.memory.recovered;
    result.nodes = brain.route.expanded;
    result.budget_hits = brain.route.budget_hits;
    result.bytes =
        size_of::<Brain>() + brain.route.bytes() + brain.path.capacity() * size_of::<Vec2>();
    result
}

fn pass(case: Case, row: &ResultRow) -> bool {
    if row.bad_sweeps != 0 || row.contacts > 1 || row.budget_hits != 0 {
        return false;
    }
    match case {
        Case::Side => row.started && row.contacts == 1 && row.end == Some(End::Contact),
        Case::Reverse | Case::Circle => {
            row.started && row.metres > 3.0 && row.end != Some(End::Route)
        }
        Case::Reopen => row.searches && row.reacquired > 0 && row.contacts == 1,
        Case::Hide => {
            row.searches && row.contacts == 0 && matches!(row.end, Some(End::Lost | End::Route))
        }
        Case::Detour => {
            row.started && row.metres > 5.0 && row.replans > 1 && row.end != Some(End::Route)
        }
        Case::Distraction => {
            row.searches
                && row.contacts == 0
                && row.noises == 1
                && row.vision_ms.is_none()
                && row.noise_ms.is_some_and(|ms| ms <= 210)
        }
        Case::Interrupted => row.started && row.end == Some(End::Interrupted) && row.contacts == 0,
        Case::Long => {
            row.decisions >= 8 && row.repeats < row.decisions / 3 && row.bytes < 160 * 1024
        }
    }
}

pub(in crate::lab) fn matrix(extra_seed: Option<u32>) -> (bool, String) {
    let began = Instant::now();
    let mut out = String::from("Cenários de inteligência da Presença\n");
    let mut total = 0;
    let mut failures = 0;
    let mut updates = 0;
    let mut max_bytes = 0;
    let mut max_nodes = 0;
    let mut checks = [0_u32; 6];
    let mut latest_visual = 0;
    let mut latest_noise = 0;
    let seeds = [
        Some(17_u32),
        Some(42),
        Some(4141),
        Some(20261118),
        extra_seed,
    ];
    for case in Case::ALL {
        let mut endings = [0; 4];
        let mut n = 0;
        for seed in seeds.into_iter().flatten() {
            for fps in [30, 60, 120] {
                let row = simulate(case, seed, fps);
                total += 1;
                n += 1;
                updates += row.updates;
                max_bytes = max_bytes.max(row.bytes);
                max_nodes = max_nodes.max(row.nodes);
                checks[0] += row.replans;
                checks[1] += row.blocks;
                checks[2] += row.recovered;
                checks[3] += row.reacquired;
                checks[4] += row.decisions;
                checks[5] += row.repeats;
                endings[match row.end {
                    Some(End::Contact) => 0,
                    Some(End::Time) => 1,
                    Some(End::Lost | End::Route) => 2,
                    _ => 3,
                }] += 1;
                if matches!(
                    case,
                    Case::Side | Case::Reverse | Case::Circle | Case::Reopen | Case::Hide
                ) {
                    latest_visual = latest_visual.max(row.vision_ms.unwrap_or_default());
                }
                latest_noise = latest_noise.max(row.noise_ms.unwrap_or_default());
                if !pass(case, &row) {
                    failures += 1;
                    let _ = writeln!(out, "FAIL {case:?} seed={seed} fps={fps}: {row:?}");
                }
            }
        }
        let _ = writeln!(
            out,
            "{case:?} {n} rotas · contato, limite, perda/rota, outro {endings:?}"
        );
    }
    let elapsed = began.elapsed().as_secs_f64();
    let _ = writeln!(
        out,
        "Resultado {} · {} de {} cenários\nAtualizações {updates} · relógio total {:.1} ms · {:.2} µs por atualização em média\nPico do cérebro, rota e buffers {max_bytes} bytes · pico acumulado de nós por execução {max_nodes}\nReplanejamentos {} · bloqueios {} · recuperações {} · retomadas {}\nDecisões {} · repetições imediatas {}\nPrimeira confirmação visual até {latest_visual} ms · reação à distração até {latest_noise} ms",
        if failures == 0 { "PASS" } else { "FAIL" },
        total - failures,
        total,
        elapsed * 1000.0,
        elapsed * 1.0e6 / f64::from(updates),
        checks[0],
        checks[1],
        checks[2],
        checks[3],
        checks[4],
        checks[5]
    );
    (failures == 0, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_run_at_three_update_rates_across_seeds() {
        let (ok, report) = matrix(None);
        println!("{report}");
        assert!(ok, "{report}");
        assert_eq!(
            simulate(Case::Long, 27194, 60),
            simulate(Case::Long, 27194, 60)
        );
    }

    #[test]
    fn equipment_and_distraction_reproductions() {
        let detour = simulate(Case::Detour, 42, 60);
        let noise = simulate(Case::Distraction, 42, 60);
        assert!(
            pass(Case::Detour, &detour) && pass(Case::Distraction, &noise),
            "{detour:?}\n{noise:?}"
        );
    }
}
