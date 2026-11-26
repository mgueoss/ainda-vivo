//! Physical campaign smoke using the normal menu, motor and action ray.
use crate::{
    app::{Comfort, LocalPlayer, Screen, Session},
    presence::observer::Campaign,
    scene::station::{self, PANEL, PUMP_02_PANEL},
};
use ainda_vivo_core::{reality::Outcome, story::Phase};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{fmt::Write, time::Instant};

#[derive(Clone, Copy)]
enum Step {
    Walk(Vec2),
    Look(&'static str, Vec3, Option<Phase>),
    Wait(f32),
}

use Step::*;
const FUSE: Vec3 = Vec3::new(-5.0, 1.02, 4.0);
const NOTE: Vec3 = Vec3::new(5.2, 0.92, -4.0);
const PUMP: Vec2 = Vec2::new(PUMP_02_PANEL.x - 1.66, PUMP_02_PANEL.z);
const BREAKER: Vec2 = Vec2::new(PANEL.x - 1.61, PANEL.z);
const CORNER: Vec2 = Vec2::new(29.6, -52.5);
const CROSSING: Vec2 = Vec2::new(3.67, -52.5);

pub(crate) fn hz() -> u32 {
    argument("--campaign-hz")
        .and_then(|v| v.parse().ok())
        .filter(|hz| [30, 60, 120].contains(hz))
        .unwrap_or(60)
}

fn argument(name: &str) -> Option<String> {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == name {
            return args.next();
        }
    }
    None
}

#[derive(Resource)]
pub(crate) struct Check {
    profile: String,
    route: Vec<Step>,
    return_route: Vec<Step>,
    step: usize,
    wait: f32,
    attempted: bool,
    frames: u32,
    wall: Instant,
    previous: Instant,
    durations: Vec<f64>,
    notes: String,
    initial_screenshot: bool,
    final_seen: bool,
    began: bool,
    uncapped: bool,
    checkpoints: u8,
    counts: (u32, u32, u32),
    load_ms: f64,
}

impl Default for Check {
    fn default() -> Self {
        let profile = argument("--campaign-profile").unwrap_or("corrida".into());
        let mut route = vec![
            Walk(Vec2::new(0.0, 4.0)),
            Walk(Vec2::new(-3.3, 4.0)),
            Look("fusível", FUSE, Some(Phase::Baseline)),
            Walk(Vec2::new(0.0, 4.0)),
            Walk(Vec2::new(0.0, -4.0)),
            Walk(Vec2::new(3.35, -4.0)),
            Look("ordem 07", NOTE, None),
            Walk(Vec2::new(0.0, -4.0)),
            Walk(Vec2::new(0.0, PUMP_02_PANEL.z)),
            Walk(PUMP),
            Look("repouso", PUMP_02_PANEL, Some(Phase::Electrical)),
        ];
        if profile == "explorador" {
            route.extend([Walk(Vec2::new(0.1, -39.4)), Wait(13.0), Walk(PUMP)]);
        }
        route.extend([
            Walk(Vec2::new(3.67, -50.11)),
            Walk(CROSSING),
            Walk(CORNER),
            Walk(Vec2::new(29.6, PANEL.z)),
            Walk(BREAKER),
            Look("disjuntor", PANEL, Some(Phase::Return)),
        ]);
        if profile == "distracao" {
            route.extend([
                Walk(Vec2::new(32.98, -55.56)),
                Look("relé", station::SERVICE_RELAY, None),
                Wait(3.0),
                Walk(BREAKER),
            ]);
        }
        let return_route = [
            Walk(Vec2::new(29.6, PANEL.z)),
            Walk(CORNER),
            Walk(CROSSING),
            Walk(Vec2::new(3.55, PUMP_02_PANEL.z)),
            Walk(PUMP),
            Look("retorno", PUMP_02_PANEL, Some(Phase::Exit)),
            Walk(Vec2::new(3.44, -44.0)),
            Walk(Vec2::new(1.12, -57.4)),
            Walk(Vec2::new(0.4, -61.65)),
            Walk(Vec2::new(7.85, -61.65)),
            Walk(Vec2::new(7.85, -82.83)),
            Walk(Vec2::new(0.0, -82.83)),
            Walk(Vec2::new(0.0, -96.13)),
        ]
        .to_vec();
        if profile == "repete" {
            route.extend([
                Walk(Vec2::new(29.6, PANEL.z)),
                Walk(CORNER),
                Walk(CROSSING),
                Wait(9.0),
                Walk(CORNER),
                Walk(Vec2::new(29.6, PANEL.z)),
                Walk(BREAKER),
            ]);
        }
        if profile == "muda" {
            route.push(Wait(16.0));
        }
        route.extend(&return_route);
        let now = Instant::now();
        Self {
            profile,
            route,
            return_route,
            step: 0,
            wait: 0.0,
            attempted: false,
            frames: 0,
            wall: now,
            previous: now,
            durations: Vec::with_capacity(12_000),
            notes: String::new(),
            initial_screenshot: false,
            final_seen: false,
            began: false,
            uncapped: std::env::args().any(|arg| arg == "--campaign-uncapped"),
            checkpoints: 0,
            counts: (0, 0, 0),
            load_ms: 0.0,
        }
    }
}

impl Check {
    fn progress(&mut self) {
        self.step += 1;
        self.wait = 0.0;
        self.attempted = false;
    }

    fn report(&mut self, session: &Session, mode: &Campaign, passed: bool) -> String {
        let end = session.expedition.observer().outcome();
        self.durations.sort_unstable_by(f64::total_cmp);
        let n = self.durations.len();
        let mean = self.durations.iter().sum::<f64>() / n.max(1) as f64;
        let tail = &self.durations[n.saturating_sub(n.div_ceil(100))..];
        let low = tail.iter().sum::<f64>() / tail.len().max(1) as f64;
        let p95 = self
            .durations
            .get(n.saturating_sub(1) * 95 / 100)
            .copied()
            .unwrap_or_default();
        format!(
            "AINDA VIVO v{}\nPhysical campaign: {}\nprofile={} hz={} vsync={} frames={} game={:.3}s end={end:?} checkpoints={:04b} step={}/{}\nwall_frame_1280x720 n={n} fps={:.2} low_1pct={:.2} median_ms={:.3} p95_ms={p95:.3} menu_ms={:.1}\n{}{}",
            crate::app::VERSION,
            if passed { "PASS" } else { "FAIL" },
            self.profile,
            hz(),
            !self.uncapped,
            self.frames,
            session.elapsed,
            self.checkpoints,
            self.step,
            self.route.len(),
            1000.0 / mean,
            1000.0 / low,
            self.durations.get(n / 2).copied().unwrap_or_default(),
            self.load_ms,
            self.notes,
            mode.report(),
        )
    }
}

fn aim(player: &mut LocalPlayer, target: Vec3) {
    let delta = target - player.body.feet - Vec3::Y * player.eye_height;
    player.yaw = (-delta.x).atan2(-delta.z);
    player.pitch = delta.y.atan2(delta.xz().length());
}

#[allow(
    clippy::too_many_arguments,
    reason = "Existing input and renderer exercised by the smoke route."
)]
pub(crate) fn drive(
    mut check: ResMut<Check>,
    smoke: Res<super::SmokeTest>,
    mode: Res<Campaign>,
    time: Res<Time>,
    mut session: ResMut<Session>,
    mut player: ResMut<LocalPlayer>,
    mut comfort: ResMut<Comfort>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    check.frames += 1;
    let now = Instant::now();
    let frame_ms = now.duration_since(check.previous).as_secs_f64() * 1000.0;
    check.previous = now;
    if session.screen == Screen::Playing && session.elapsed >= 4.0 {
        check.durations.push(frame_ms);
    }
    if check.uncapped {
        comfort.vsync = false;
    }
    for key in [KeyCode::Enter, KeyCode::KeyE] {
        keys.release(key);
    }
    if mode.physical() && !check.began && check.frames > 15 {
        check.began = true;
        check.load_ms = check.wall.elapsed().as_secs_f64() * 1000.0;
        keys.press(KeyCode::Enter);
    }
    let counts = mode.qa_counts();
    if counts != check.counts {
        check.counts = counts;
        let _ = writeln!(
            check.notes,
            "{:.2}s ator: manifestações={} corridas={} planos encerrados={}",
            session.elapsed, counts.0, counts.1, counts.2
        );
    }
    let photo = |tag: &str, commands: &mut Commands| {
        if let Some(path) = &smoke.capture_dir
            && std::fs::create_dir_all(path).is_ok()
        {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path.join(format!("{tag}.png"))));
        }
    };
    if session.screen == Screen::Complete && !check.final_seen {
        photo("06-saida", &mut commands);
        check.final_seen = true;
        return;
    }
    if check.frames >= 192 * hz() || session.screen == Screen::Complete && check.final_seen {
        let passed = mode.physical()
            && check.checkpoints == 15
            && session.expedition.observer().outcome() == Some(Outcome::Escaped);
        let output = check.report(&session, &mode, passed);
        info!("{output}");
        let written = smoke
            .report
            .as_ref()
            .is_none_or(|p| std::fs::write(p, output).is_ok());
        exit.write(if passed && written {
            AppExit::Success
        } else {
            AppExit::error()
        });
        return;
    }
    if session.screen != Screen::Playing {
        return;
    }
    if session.elapsed > 1.0 && !check.initial_screenshot {
        photo("01-acesso", &mut commands);
        check.initial_screenshot = true;
    }
    player.command = default();
    if session.expedition.observer().outcome() == Some(Outcome::PerceivedExit) {
        if matches!(
            session.false_exit,
            Some(crate::app::ExitPresentation::Contradiction)
        ) {
            session.screen = Screen::Paused;
            crate::ui::request_awakening(&mut session);
            crate::app::awaken(&mut session, &mut player);
            let _ = writeln!(
                check.notes,
                "{:.2}s Despertar confirmou retomada do posto",
                session.elapsed
            );
            check.route = check.return_route.clone();
            check.step = 0;
            check.wait = 0.0;
            check.attempted = false;
        }
        return;
    }
    let Some(step) = check.route.get(check.step).copied() else {
        return;
    };
    let dt = time.delta_secs();
    match step {
        Walk(at) => {
            let direction = at - player.body.feet.xz();
            if direction.length_squared() < 0.24_f32.powi(2) {
                check.progress();
                return;
            }
            player.yaw = (-direction.x).atan2(-direction.y);
            player.pitch = 0.0;
            player.command.yaw = player.yaw;
            player.command.movement = Vec2::Y;
            let sprint = match check.profile.as_str() {
                "cauteloso" | "explorador" => false,
                "muda" => session.elapsed < 35.0 || session.elapsed.rem_euclid(12.0) < 4.0,
                _ => true,
            };
            player.command.sprint = sprint && direction.length_squared() > 3.2;
            player.command.crouch =
                check.profile == "muda" && (55.0..61.0).contains(&session.elapsed);
        }
        Look(label, at, expected) => {
            aim(&mut player, at);
            check.wait += dt;
            if check.wait > 0.26 && !check.attempted && !session.prompt.is_empty() {
                check.attempted = true;
                check.wait = 0.0;
                keys.press(KeyCode::KeyE);
            }
            if check.attempted
                && check.wait > dt * 1.75
                && expected.is_none_or(|phase| session.progress().phase() == phase)
            {
                let _ = writeln!(
                    check.notes,
                    "{:.3}s E {label} setor={:?}: {}",
                    session.elapsed,
                    station::sector_id(player.body.feet),
                    session.message
                );
                if let Some(phase) = expected {
                    check.checkpoints |= 1 << (phase as u8 - 1);
                    photo(&format!("etapa-{}", phase as u8), &mut commands);
                }
                check.progress();
            } else if check.attempted && check.wait > 0.7 {
                check.attempted = false;
            }
        }
        Wait(seconds) => {
            player.yaw = (session.elapsed * 0.22).sin() * 1.5;
            player.pitch = 0.0;
            check.wait += dt;
            if check.wait >= seconds {
                check.progress();
            }
        }
    }
}
