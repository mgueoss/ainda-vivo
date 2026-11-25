use super::{
    Brain, Footstep, Frame, HUMAN, Nav, Noise, Portal, Pose, State, Tuning, View, Zone, forward,
    point, presentation,
};
use super::{hunt, hunt_lab::HuntLab};
use crate::lab::{Active, Lab, library::Catalog, room, settings::Config};
use ainda_vivo_core::collision::Aabb;
use bevy::{
    camera::visibility::{NoFrustumCulling, RenderLayers},
    prelude::*,
};

const HOME: Vec2 = Vec2::new(4.55, -12.55);
const TARGETS: [Vec2; 6] = [
    Vec2::new(4.63, -6.55),
    Vec2::new(4.75, -11.4),
    Vec2::new(6.88, -4.9),
    Vec2::new(-3.4, -1.8),
    Vec2::new(-6.6, -10.0),
    Vec2::new(7.15, -16.0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Knob {
    Fov,
    Range,
    Hearing,
    Speed,
    Watch,
    Investigate,
    Approach,
}

impl Knob {
    pub const ALL: [(Self, &'static str, f32); 7] = [
        (Self::Fov, "Campo de visão", 10.0),
        (Self::Range, "Alcance visual", 2.0),
        (Self::Hearing, "Audição", 0.2),
        (Self::Speed, "Movimento", 0.2),
        (Self::Watch, "Observar", 1.0),
        (Self::Investigate, "Investigar", 1.0),
        (Self::Approach, "Distância mínima", 0.5),
    ];

    fn setting(self, tuning: &mut Tuning) -> (&mut f32, f32, f32, &'static str) {
        match self {
            Self::Fov => (&mut tuning.fov, 35., 165., "°"),
            Self::Range => (&mut tuning.range, 4., 32., " m"),
            Self::Hearing => (&mut tuning.hearing, 0., 2.5, "×"),
            Self::Speed => (&mut tuning.speed, 0.2, 2.5, "×"),
            Self::Watch => (&mut tuning.watching, 1.0, 20.0, " s"),
            Self::Investigate => (&mut tuning.investigation, 2.0, 22.0, " s"),
            Self::Approach => (&mut tuning.approach, 3.0, 12.0, " m"),
        }
    }

    pub fn value(self, mut current: Tuning) -> String {
        let (v, _, _, unit) = self.setting(&mut current);
        format!("{v:.1}{unit}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Enable,
    Pause,
    Reset,
    Automatic,
    Door,
    Guides,
    Target,
    Noise,
    Select(State),
    Adjust(Knob, f32),
}

#[derive(Default)]
pub struct TestLab {
    pub enabled: bool,
    pub paused: bool,
    pub automatic: bool,
    pub closed: bool,
    pub guides: bool,
    pub target: usize,
    pub brain: Brain,
    pub tuning: Tuning,
    pub hunt: HuntLab,
    stage: Option<Stage>,
    nav: Option<Nav>,
    pulse: Option<Noise>,
    footstep: Footstep,
    blocked_door: bool,
}

struct Stage {
    root: Entity,
    actor: Entity,
    gate: Entity,
    guides: Entity,
    lines: [Handle<Mesh>; 2],
    revision: u32,
}

fn gate() -> Aabb {
    Aabb::from_center_size(Vec3::new(4.60, 1.275, -9.0), Vec3::new(2.50, 2.55, 0.22))
}

fn reset_brain(brain: &mut Brain) {
    *brain = Brain::seeded(70146);
    brain.spawn(HOME, 0.0);
}

impl TestLab {
    pub fn dispose(&mut self, commands: &mut Commands) {
        if let Some(stage) = self.stage.take() {
            commands.entity(stage.root).despawn();
        }
    }

    pub fn solid(&self) -> Option<Aabb> {
        (self.enabled && self.closed).then(gate)
    }

    pub fn diagnostics(&self) -> String {
        let b = &self.brain;
        if !self.enabled {
            return "Ative o cenário no corredor leste.".into();
        }
        let describe =
            |at: Option<Vec2>| at.map_or("—".into(), |p| format!("{:.1}, {:.1}", p.x, p.y));
        let memory = |item: Option<super::brain::Memory>| {
            item.map_or("—".into(), |m| {
                format!(
                    "{} · {:.1}s",
                    describe(Some(m.at)),
                    (m.expires - b.clock).max(0.0)
                )
            })
        };
        format!(
            "{} · {}\n{}\nDestino {} · rota {}\nÚlt. visão {}\nÚlt. ruído {}\nSetores {:?} · encontro {:?}\nAlvo {} · porta {}{}",
            b.state.label(),
            if self.paused {
                "pausada"
            } else if self.automatic {
                "automática"
            } else {
                "manual"
            },
            b.reason,
            describe(b.goal),
            b.path.len().saturating_sub(b.waypoint),
            memory(b.seen),
            memory(b.heard),
            b.visited,
            b.hit_sector,
            describe(Some(TARGETS[self.target])),
            if self.closed { "fechada" } else { "aberta" },
            if self.blocked_door {
                " · área ocupada"
            } else {
                ""
            }
        )
    }
}

pub fn command(active: &mut Active, config: &mut Config, action: Command) {
    if action == Command::Enable && !active.observer.enabled {
        config.tab = crate::lab::settings::Tab::Observer;
        config.lens.free = false;
        config.lens.position = [4.60, 1.622, -4.55];
        config.lens.yaw = 0.0;
        config.lens.pitch = -0.045;
        active.reset_view(&config.lens);
        active.playback.cancel_run();
    }
    let lab = &mut active.observer;
    match action {
        Command::Enable => {
            lab.enabled = !lab.enabled;
            if lab.enabled {
                lab.automatic = true;
                lab.paused = false;
                lab.guides = true;
                reset_brain(&mut lab.brain);
            }
        }
        Command::Pause => lab.paused = !lab.paused,
        Command::Reset => {
            reset_brain(&mut lab.brain);
            lab.footstep = Footstep::default();
            lab.pulse = None;
            lab.paused = false;
            lab.hunt.reset();
        }
        Command::Automatic => {
            lab.automatic = !lab.automatic;
            if lab.automatic && lab.brain.state == State::Absent {
                reset_brain(&mut lab.brain);
            }
        }
        Command::Door => {
            let occupied = gate()
                .expanded(if lab.brain.hunt.low() {
                    hunt::half_at(lab.brain.yaw)
                } else {
                    HUMAN
                })
                .ray(point(lab.brain.pos).with_y(1.14), Vec3::X, 0.0)
                .is_some()
                || gate()
                    .expanded(Vec3::splat(0.29))
                    .ray(active.walker.feet + Vec3::Y * 0.8, Vec3::X, 0.0)
                    .is_some();
            lab.blocked_door = !lab.closed && occupied;
            if !lab.blocked_door {
                lab.closed = !lab.closed;
            }
        }
        Command::Guides => lab.guides = !lab.guides,
        Command::Target => lab.target = (lab.target + 1) % TARGETS.len(),
        Command::Noise => {
            lab.pulse = Some(Noise {
                at: TARGETS[lab.target],
                power: 3.1,
            })
        }
        Command::Select(state) => {
            lab.automatic = false;
            if lab.brain.state == State::Absent && state != State::Absent {
                reset_brain(&mut lab.brain);
            }
            let mut target = TARGETS[lab.target];
            if state == State::Approach {
                let offset = lab.brain.pos - target;
                target += offset.normalize_or_zero() * lab.tuning.approach.min(offset.length());
            }
            if state == State::Threaten
                && lab.nav.as_ref().is_some_and(|nav| {
                    !nav.clear_body(
                        lab.brain.pos,
                        lab.brain.pos,
                        Vec3::new(1.24, 1.12, 1.24),
                        lab.closed,
                    )
                })
            {
                lab.brain.reason = "sem área para a ruptura";
            } else {
                lab.brain.direct(state, target, lab.tuning);
            }
        }
        Command::Adjust(what, delta) => {
            let (value, min, max, _) = what.setting(&mut lab.tuning);
            *value = (*value + delta).clamp(min, max);
        }
    }
}

fn spawn(
    active: &mut Active,
    catalog: &Catalog,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let root = commands
        .spawn((
            ChildOf(active.root),
            Name::new("Corredor de teste da IA"),
            Transform::default(),
            Visibility::Inherited,
        ))
        .id();
    let Some(actor) = catalog.spawn_observer(
        commands,
        Pose::new(true),
        Transform::from_translation(point(HOME)),
        Some(root),
    ) else {
        commands.entity(root).despawn();
        return;
    };
    let gate = commands
        .spawn((
            ChildOf(root),
            Name::new("Porta de ensaio"),
            Mesh3d(meshes.add(Cuboid::new(2.50, 2.55, 0.22))),
            MeshMaterial3d(materials.add(Color::srgb(0.28, 0.22, 0.11))),
            Transform::from_xyz(4.60, 1.275, -9.0),
            Visibility::Hidden,
            RenderLayers::layer(room::LAYER),
        ))
        .id();
    let guides = commands
        .spawn((ChildOf(root), Transform::default(), Visibility::Inherited))
        .id();
    let lines = [Color::srgb(0.33, 0.96, 0.70), Color::srgb(1.0, 0.65, 0.26)].map(|base_color| {
        let mesh = meshes.add(room::line_mesh(vec![[0.; 3]; 2]));
        let material = materials.add(StandardMaterial {
            base_color,
            unlit: true,
            ..default()
        });
        commands.spawn((
            ChildOf(guides),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            NoFrustumCulling,
            RenderLayers::layer(room::LAYER),
            Transform::default(),
        ));
        mesh
    });
    active.observer.stage = Some(Stage {
        root,
        actor,
        gate,
        guides,
        lines,
        revision: catalog.revision(),
    });
}

fn lab_nav(solids: &[Aabb]) -> Nav {
    let boundary = -9.0;
    Nav::new(
        vec![
            Zone {
                min: Vec2::new(-8.22, boundary),
                max: Vec2::new(8.22, 6.30),
            },
            Zone {
                min: Vec2::new(-8.22, -17.28),
                max: Vec2::new(8.22, boundary),
            },
        ],
        [(-5.8, 4.8), (0.0, 5.0), (4.60, 2.55), (7.1, 2.10)]
            .map(|(x, width)| Portal {
                rooms: [0, 1],
                at: Vec2::new(x, boundary),
                width,
                height: 2.65,
            })
            .into(),
        solids.to_vec(),
        Some(gate()),
    )
}

pub fn update_lab(
    mut commands: Commands,
    mut lab: ResMut<Lab>,
    config: Res<Config>,
    catalog: Res<Catalog>,
    time: Res<Time>,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    mut signals: MessageWriter<hunt::Signal>,
) {
    let Some(active) = &mut lab.active else {
        return;
    };
    if !active.observer.enabled
        || active
            .observer
            .stage
            .as_ref()
            .is_some_and(|s| s.revision != catalog.revision())
    {
        active.observer.dispose(&mut commands);
    }
    if !active.observer.enabled || catalog.number(12).is_none() {
        return;
    }
    if active.observer.stage.is_none() {
        spawn(active, &catalog, &mut commands, &mut meshes, &mut materials);
    }
    let eye = active.camera_transform(&config);
    let view = View {
        eye: eye.translation,
        facing: *eye.forward(),
        protected: false,
    };
    let lab = &mut active.observer;
    let Some(stage) = &lab.stage else { return };
    let nav = lab.nav.get_or_insert_with(|| lab_nav(&active.room.solids));
    if !lab.paused {
        lab.hunt.session.elapsed += time.delta_secs();
        lab.hunt.session.psychology.step(
            time.delta_secs(),
            ainda_vivo_core::psychology::Recovery::None,
        );
        let footstep = lab.footstep.sample(
            active.walker.feet.xz(),
            active.walker.velocity.xz().length(),
            active.walker.crouched,
        );
        let noises = lab.pulse.take().or(footstep);
        let frame = Frame {
            nav,
            door_closed: lab.closed,
            view,
            noises: noises.as_slice(),
            points: &TARGETS,
        };
        if lab.automatic
            && lab.brain.state == State::Absent
            && lab.brain.clock >= lab.brain.next_manifest
        {
            lab.brain.spawn(HOME, 0.);
        }
        lab.brain
            .step(time.delta_secs(), &frame, lab.tuning, lab.automatic);
        if let Some(cue) = lab.brain.hunt.cue.take() {
            signals.write(hunt::Signal {
                cue,
                at: point(lab.brain.pos).with_y(0.45),
                lab: true,
            });
        }
        if std::mem::take(&mut lab.brain.hunt.pending) {
            lab.brain.hunt.result = Some(lab.hunt.contact());
        }
    }
    lab.hunt.distance = lab.brain.pos.distance(view.eye.xz());
    let b = &lab.brain;
    let visibility = |v| {
        if v {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    commands.entity(stage.gate).insert(visibility(lab.closed));
    commands.entity(stage.guides).insert(visibility(lab.guides));
    commands.entity(stage.actor).insert((
        presentation(b, true, lab.paused),
        Transform::from_translation(point(b.pos).with_y(0.022))
            .with_rotation(Quat::from_rotation_y(b.yaw)),
        visibility(b.state != State::Absent && !b.hunt.hidden()),
    ));
    if lab.guides {
        draw(lab, &mut meshes);
    }
}

fn draw(lab: &TestLab, meshes: &mut Assets<Mesh>) {
    let stage = lab.stage.as_ref().unwrap();
    let b = &lab.brain;
    let mut planned = Vec::with_capacity(2 * b.path.len() + 24);
    let mut sight = Vec::with_capacity(40);
    let mut line = |a: Vec3, b: Vec3| planned.extend([a.to_array(), b.to_array()]);
    let base = |p| point(p).with_y(0.05);
    let route = b.path.get(b.waypoint..).unwrap_or_default();
    let mut previous = base(b.pos);
    for &p in route {
        let next = base(p);
        line(previous, next);
        previous = next;
    }
    for p in [b.goal, b.seen.map(|m| m.at), Some(TARGETS[lab.target])]
        .into_iter()
        .flatten()
    {
        let p = base(p);
        line(p - Vec3::X * 0.24, p + Vec3::X * 0.24);
        line(p - Vec3::Z * 0.24, p + Vec3::Z * 0.24);
        line(p, p + Vec3::Y * 0.47);
    }
    let mut guide = |a: Vec3, b: Vec3| sight.extend([a.to_array(), b.to_array()]);
    if b.state != State::Absent {
        let eye = b.eye();
        let angle = b.yaw + b.head;
        let half = lab.tuning.fov.to_radians() * 0.5;
        let edge = |i: f32| eye + point(forward(angle + i * half)) * lab.tuning.range;
        guide(eye, edge(-1.0));
        guide(eye, edge(1.0));
        for n in 0..8 {
            guide(edge(n as f32 / 4.0 - 1.0), edge((n + 1) as f32 / 4.0 - 1.0));
        }
        if let Some((a, b, clear)) = b.ray {
            if clear {
                line(a, b);
            } else {
                guide(a, b);
            }
        }
        let half = if b.low() { super::nav::LOW } else { HUMAN };
        let rotation = Quat::from_rotation_y(b.yaw);
        let corners = std::array::from_fn::<_, 8, _>(|i| {
            let signs = Vec3::new(
                if i & 1 == 0 { -1. } else { 1. },
                if i & 2 == 0 { -1. } else { 1. },
                if i & 4 == 0 { -1. } else { 1. },
            );
            point(b.pos) + Vec3::Y * (half.y + 0.022) + rotation * (half * signs)
        });
        for i in 0..8 {
            for bit in [1, 2, 4] {
                if i & bit == 0 {
                    guide(corners[i], corners[i | bit]);
                }
            }
        }
    }
    for (id, vertices) in stage.lines.iter().zip([planned, sight]) {
        if let Some(mesh) = meshes.get_mut(id) {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                if vertices.is_empty() {
                    vec![[0.; 3]; 2]
                } else {
                    vertices
                },
            );
        }
    }
}
