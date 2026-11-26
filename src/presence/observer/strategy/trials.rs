use super::*;
use crate::presence::observer::{
    brain::{Brain, Frame, Noise, STEP, Tuning, View},
    nav::{Portal, Zone},
};
use ainda_vivo_core::collision::Aabb;
use std::fmt::Write;

const HINTS: [Vec2; 2] = [Vec2::new(6.1, -4.0), Vec2::new(3.0, -5.0)];
const ACTOR: Vec2 = Vec2::new(0.0, -5.0);

fn solid(x: f32, z: f32, width: f32, depth: f32) -> Aabb {
    Aabb::from_center_size(Vec3::new(x, 1.3, z), Vec3::new(width, 2.6, depth))
}

fn arena(solids: Vec<Aabb>) -> Nav {
    Nav::new(
        [(-8.0, 8.0), (8.0, 20.0), (-20.0, -8.0)]
            .map(|(a, b)| Zone {
                min: Vec2::new(a, -8.0),
                max: Vec2::new(b, 8.0),
            })
            .to_vec(),
        [8.0, -8.0]
            .into_iter()
            .enumerate()
            .map(|(i, x)| Portal {
                rooms: [0, i + 1],
                at: Vec2::new(x, 0.0),
                width: 3.6,
                height: 3.0,
            })
            .collect(),
        solids,
        None,
    )
}

fn clue(at: Vec2, when: f32, sector: usize) -> Memory {
    Memory {
        at,
        when,
        expires: when + 9.0,
        certainty: 0.94,
        error: 0.22,
        velocity: Vec2::ZERO,
        sector: Some(sector),
    }
}

fn input<'a>(nav: &'a Nav, book: &'a MemoryBook, now: f32, own: Vec2, clue: Memory) -> Inputs<'a> {
    Inputs {
        now,
        own,
        state: State::Watch,
        knowledge: Knowledge::Confirmed,
        clue: Some(clue),
        observation: Some(clue),
        noticed: false,
        protected: false,
        blocked: 0,
        nav,
        memory: book,
        closed: false,
        visited: &[None; 4],
        hints: &HINTS,
    }
}

fn habit_and_failures(out: &mut String) -> bool {
    let nav = arena(vec![]);
    let mut book = MemoryBook::default();
    let own = Vec2::new(5.4, -0.2);
    let observed = clue(Vec2::new(0., -1.2), 10.0, 0);
    let mind = Mind::default();
    let empty = mind
        .choose(&input(&nav, &book, 10.0, own, observed), observed, None)
        .map(|p| p.kind);
    for sample in 0..4 {
        let t = 4.0 + sample as f32;
        book.observe(
            Some(clue(Vec2::new(7.4, 0.0), t, 0)),
            clue(Vec2::new(8.7, 0.0), t + 0.12, 1),
            &nav,
            false,
        );
    }
    let i = input(&nav, &book, 10.0, own, observed);
    let informed = mind
        .choose(&i, observed, None)
        .map(|p| (p.kind, p.crossing));
    let mut failed = Mind {
        enabled: true,
        ..default()
    };
    let base = failed.scores(&i, observed)[Kind::Interception as usize];
    for k in 0..2 {
        let t = 10.0 + k as f32 * 7.0;
        let observation = clue(observed.at, t, 0);
        let f = input(&nav, &book, t, own, observation);
        failed.plan = build(Kind::Interception, &f, observation);
        let mut blocked = input(&nav, &book, t + 0.50, own, observation);
        blocked.blocked = 1;
        failed.step(blocked);
    }
    let after = failed.scores(
        &input(&nav, &book, 18.0, own, clue(observed.at, 18.0, 0)),
        clue(observed.at, 18.0, 0),
    )[Kind::Interception as usize];
    let old_plan = build(Kind::Interception, &i, observed).and_then(|p| p.crossing);
    for k in 0..4 {
        let t = 20.0 + k as f32;
        book.observe(
            Some(clue(Vec2::new(-7.4, 0.0), t, 0)),
            clue(Vec2::new(-8.7, 0.0), t + 0.12, 2),
            &nav,
            false,
        );
    }
    book.failed([0, 1], 23.7);
    book.failed([0, 1], 24.1);
    let west = clue(observed.at, 25.0, 0);
    let changed = build(
        Kind::Interception,
        &input(&nav, &book, 25.0, -own, west),
        west,
    )
    .and_then(|p| p.crossing);
    let _ = writeln!(
        out,
        "A memória vazia={empty:?} observações=4 plano={informed:?}"
    );
    let _ = writeln!(
        out,
        "B bloqueios={} prioridade={base:.3}->{after:.3}\n{}",
        failed.finished,
        failed.report(18.0)
    );
    let _ = writeln!(
        out,
        "C passagem antes={old_plan:?} após={changed:?} peso_leste={:.3} oeste={:.3} leste_tardio={:.3}",
        book.rate(0, 1, 25.0),
        book.rate(0, 2, 25.0),
        book.rate(0, 1, 140.0)
    );
    empty != Some(Kind::Interception)
        && informed == Some((Kind::Interception, Some(1)))
        && failed.finished == 2
        && after < base - 0.20
        && old_plan == Some(1)
        && changed == Some(2)
}

fn reaction(out: &mut String, kind: Kind, expected: ResultKind) -> bool {
    let nav = arena(vec![solid(1.65, -2.7, 0.7, 1.3)]);
    let book = MemoryBook::default();
    let first = clue(Vec2::ZERO, 10.0, 0);
    let inputs = input(&nav, &book, 10.0, ACTOR, first);
    let proposal = build(kind, &inputs, first);
    let Some(plan) = proposal else {
        let _ = writeln!(out, "D {kind:?} sem precondição");
        return false;
    };
    let mut mind = Mind {
        enabled: true,
        plan: Some(plan),
        ..default()
    };
    mind.step(input(&nav, &book, 10.0, ACTOR, first));
    let Some(order) = mind.request() else {
        return false;
    };
    mind.feedback(order.id, Delivery::Reserved, 10.1);
    mind.feedback(order.id, Delivery::Played, 10.4);
    let sector = match expected {
        ResultKind::Success => order.sector,
        ResultKind::Unexpected => {
            if order.sector == 1 {
                2
            } else {
                1
            }
        }
        _ => 0,
    };
    let now = 11.0;
    let observed = clue(
        match sector {
            1 => Vec2::new(8.8, 0.0),
            2 => Vec2::new(-8.8, 0.0),
            _ => Vec2::ZERO,
        },
        now,
        sector,
    );
    let mut own = ACTOR;
    for j in 0..=50 {
        let t = now + j as f32 * 0.18;
        let witnessing = clue(observed.at, t, sector);
        let result = mind.step(input(&nav, &book, t, own, witnessing));
        if let Some(Directive::Begin(_, Some(goal))) = result {
            own = goal;
        }
        if !mind.active() {
            break;
        }
    }
    let actual = mind.record[0].map(|r| r.result);
    let _ = writeln!(
        out,
        "D {kind:?} pedido={:?} setor={} ator={} testemunhado={sector} esperado={expected:?} observado={actual:?}\n{}",
        order.kind,
        order.sector,
        order.actor_sector,
        mind.report(22.0)
    );
    actual == Some(expected) && order.sector != order.actor_sector
}

fn physical_ambush(out: &mut String) -> bool {
    let nav = arena(vec![solid(4.0, -1.95, 0.81, 1.05)]);
    let origin = Vec2::new(2.9, -3.55);
    let post = HINTS[0];
    let book = MemoryBook::default();
    let clue = clue(Vec2::new(2.1, 0.0), 10.0, 0);
    let mut i = input(&nav, &book, 10.0, origin, clue);
    i.hints = &HINTS[..1];
    let Some(plan) = build(Kind::Ambush, &i, clue) else {
        let _ = writeln!(out, "E emboscada indisponível");
        return false;
    };
    let mut actor = Brain::campaign(20261118);
    actor.spawn(origin, std::f32::consts::FRAC_PI_2);
    actor.clock = 10.0;
    actor.mind.plan = Some(plan);
    let target = View {
        eye: Vec3::new(-15.0, 1.62, 7.6),
        facing: Vec3::Z,
        protected: false,
    };
    let mut reached = false;
    let mut bad_sweep = false;
    for _ in 0..(60 * 18) {
        let last = actor.pos;
        actor.step(
            STEP,
            &Frame {
                nav: &nav,
                door_closed: false,
                view: target,
                noises: &[],
                points: &HINTS,
            },
            Tuning::default(),
            true,
        );
        bad_sweep |=
            last.distance(actor.pos) > 0.02 || !nav.clear_body(last, actor.pos, HUMAN, false);
        reached |=
            actor.pos.distance(post) < 0.34 && matches!(actor.state, State::Watch | State::Follow);
        if !actor.mind.active() && actor.mind.finished > 0 {
            break;
        }
    }
    let result = actor.mind.record[0].map(|r| r.result);
    let _ = writeln!(
        out,
        "E emboscada real origem={origin:?} posto={post:?} chegou={reached} varredura_ruim={bad_sweep} resultado={result:?}\n{}",
        actor.mind.report(actor.clock)
    );
    reached && !bad_sweep && result == Some(ResultKind::NoEvidence)
}

fn hidden_and_noisy(out: &mut String) -> bool {
    let nav = arena(vec![solid(0.0, -2.0, 13.0, 0.22)]);
    let run = |eye: Vec3, noisy: bool| {
        let mut actor = Brain::campaign(42);
        actor.spawn(ACTOR, 0.0);
        let mut states = Vec::new();
        for tick in 0..260 {
            let pulse = (noisy && tick == 95).then_some(Noise {
                at: Vec2::new(3.3, -5.2),
                power: 4.1,
            });
            actor.step(
                STEP,
                &Frame {
                    nav: &nav,
                    door_closed: false,
                    noises: pulse.as_slice(),
                    view: View {
                        eye,
                        facing: Vec3::Z,
                        protected: false,
                    },
                    points: &HINTS,
                },
                Tuning::default(),
                true,
            );
            states.push((actor.state, actor.goal));
        }
        (states, actor.seen.map(|m| m.at), actor.heard.map(|m| m.at))
    };
    let a = Vec3::new(0., 1.62, 4.);
    let b = Vec3::new(-4.0, 1.62, 2.7);
    let (quiet, lost, _) = run(a, false);
    let (other, hidden, _) = run(b, false);
    let (noisy, sight, heard) = run(a, true);
    let investigated = noisy
        .iter()
        .any(|&(s, goal)| s == State::Investigate && goal.is_some());
    let _ = writeln!(
        out,
        "F distração física ruído=(3.3,-5.2) pista={heard:?} investigou={investigated} trajetos_diferentes={}\nG posições ocultas distintas, decisões idênticas={} visão={lost:?}/{hidden:?}/{sight:?}",
        quiet != noisy,
        quiet == other
    );
    investigated
        && quiet != noisy
        && quiet == other
        && lost.is_none()
        && hidden.is_none()
        && sight.is_none()
        && heard.is_some()
}

pub(crate) fn report() -> (bool, String) {
    let mut out = String::new();
    let a = habit_and_failures(&mut out);
    let b = [
        ResultKind::Success,
        ResultKind::Ignored,
        ResultKind::Unexpected,
    ]
    .into_iter()
    .map(|reaction_kind| reaction(&mut out, Kind::Diversion, reaction_kind))
    .fold(true, |a, b| a & b);
    let c = reaction(&mut out, Kind::FalseChase, ResultKind::Ignored);
    let d = physical_ambush(&mut out);
    let e = hidden_and_noisy(&mut out);
    for kind in [Kind::FeignedRetreat, Kind::Investigation] {
        let nav = arena(vec![]);
        let book = MemoryBook::default();
        let observed = clue(Vec2::ZERO, 15., 0);
        let plan = build(kind, &input(&nav, &book, 15., ACTOR, observed), observed);
        let _ = writeln!(out, "Plano {kind:?} {:?}", plan.map(|p| p.actions));
    }
    let _ = writeln!(
        out,
        "Resultado {} · aprendizado={a} desvio={b} ameaça={c} emboscada={d} percepção={e}",
        if a && b && c && d && e {
            "PASS"
        } else {
            "FAIL"
        }
    );
    (a && b && c && d && e, out)
}

#[test]
fn strategy_matrix_reproduces_observation_adaptation_and_counterplay() {
    let (ok, text) = report();
    assert!(ok, "{text}");
}

#[test]
fn the_interception_swaps_to_a_new_habit_from_the_same_pose() {
    let nav = Nav::new(
        [(-4.5, 4.5), (4.5, 20.0), (-20.0, -4.5)]
            .map(|(a, b)| Zone {
                min: Vec2::new(a, -8.0),
                max: Vec2::new(b, 8.0),
            })
            .to_vec(),
        [4.5, -4.5]
            .into_iter()
            .enumerate()
            .map(|(index, x)| Portal {
                rooms: [0, index + 1],
                at: Vec2::new(x, 0.0),
                width: 3.6,
                height: 3.0,
            })
            .collect(),
        vec![],
        None,
    );
    let own = Vec2::ZERO;
    let observe = |book: &mut MemoryBook, side: f32, to: usize, start: f32| {
        for index in 0..4 {
            let when = start + index as f32;
            book.observe(
                Some(clue(Vec2::new(side * 4.2, 0.0), when, 0)),
                clue(Vec2::new(side * 4.85, 0.0), when + 0.15, to),
                &nav,
                false,
            );
        }
    };
    let observed = clue(Vec2::new(0.0, -6.4), 40.0, 0);
    let select = |book: &MemoryBook| {
        build(
            Kind::Interception,
            &input(&nav, book, 40.0, own, observed),
            observed,
        )
        .and_then(|p| p.crossing)
    };
    let mut book = MemoryBook::default();
    observe(&mut book, 1.0, 1, 4.0);
    let east = select(&book);
    observe(&mut book, -1.0, 2, 34.0);
    book.failed([0, 1], 38.2);
    book.failed([0, 1], 39.1);
    let result = (east, select(&book));
    println!(
        "C controlado posição={own:?} pista={:?} passagens={result:?}",
        observed.at
    );
    assert_eq!(result, (Some(1), Some(2)));
}

#[test]
fn retreat_and_investigation_walk_their_route_then_release_an_empty_lead() {
    let nav = arena(vec![]);
    let book = MemoryBook::default();
    let clue = clue(Vec2::new(0.0, -0.6), 10.0, 0);
    for kind in [Kind::FeignedRetreat, Kind::Investigation] {
        let mut actor = Brain::campaign(42);
        actor.spawn(ACTOR, 0.0);
        actor.clock = 10.0;
        let plan = build(kind, &input(&nav, &book, 10.0, ACTOR, clue), clue).unwrap();
        let goal = match plan.actions[0].unwrap() {
            Action::Move(goal, _) => goal,
            action => panic!("primeira ação inesperada: {action:?}"),
        };
        actor.mind.plan = Some(plan);
        let mut walked = false;
        for _ in 0..1_220 {
            let from = actor.pos;
            actor.step(
                STEP,
                &Frame {
                    nav: &nav,
                    door_closed: false,
                    view: View {
                        eye: Vec3::new(-19.0, 1.62, 7.6),
                        facing: Vec3::Z,
                        protected: false,
                    },
                    points: &[],
                    noises: &[],
                },
                Tuning::default(),
                true,
            );
            assert!(nav.clear_body(from, actor.pos, HUMAN, false));
            walked |= actor.pos.distance(goal) < 0.34;
            if actor.mind.finished > 0 {
                break;
            }
        }
        let result = actor.mind.record[0].map(|r| r.result);
        println!("{kind:?} origem={ACTOR:?} destino={goal:?} chegou={walked} resultado={result:?}");
        assert!(walked && result == Some(ResultKind::NoEvidence));
    }
}
