use super::{
    Brain, State,
    hunt::{self, Contact, End, Hunt, Phase},
    test_lab::{Command, TestLab, command},
};
use crate::{
    app::{Screen, Session},
    lab::{
        Active,
        settings::{Config, Tab},
    },
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HuntKnob {
    Speed,
    Acceleration,
    Start,
    Duration,
    SearchLimit,
    Prepare,
    Reach,
}

impl HuntKnob {
    pub const ALL: [(Self, &'static str, f32); 7] = [
        (Self::Speed, "Velocidade", 0.25),
        (Self::Acceleration, "Aceleração", 0.5),
        (Self::Start, "Distância inicial", 0.5),
        (Self::Duration, "Esforço", 0.5),
        (Self::SearchLimit, "Corrida e busca", 0.5),
        (Self::Prepare, "Preparação", 0.2),
        (Self::Reach, "Alcance de contato", 0.1),
    ];
    fn setting(self, t: &mut hunt::Tuning) -> (&mut f32, f32, f32, &'static str) {
        match self {
            Self::Speed => (&mut t.speed, 1.0, 6.0, " m/s"),
            Self::Acceleration => (&mut t.acceleration, 1.0, 9.0, " m/s²"),
            Self::Start => (&mut t.start, 5.0, 13.0, " m"),
            Self::Duration => (&mut t.duration, 2.0, 16.0, " s"),
            Self::SearchLimit => (&mut t.limit, 4.0, 25.0, " s"),
            Self::Prepare => (&mut t.prepare, 1.0, 5.0, " s"),
            Self::Reach => (&mut t.reach, 0.9, 1.8, " m"),
        }
    }
    pub fn value(self, mut t: hunt::Tuning) -> String {
        let (v, _, _, unit) = self.setting(&mut t);
        format!("{v:.2}{unit}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HuntCommand {
    Start,
    Bluff,
    DoorTrial,
    Stop,
    Rehearse,
    Result(Option<Contact>),
    Psychology,
    Adjust(HuntKnob, f32),
    Profile(HuntProfile),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HuntProfile {
    Base,
    Moderate,
    Intense,
}

impl HuntProfile {
    pub fn tuning(self) -> hunt::Tuning {
        let mut tuning = hunt::Tuning::default();
        let (speed, acceleration, duration, limit) = match self {
            Self::Base => (4.95, 4.2, 7.5, 12.0),
            Self::Moderate => (5.5, 5.0, 12.0, 18.0),
            Self::Intense => (6.0, 6.0, 16.0, 25.0),
        };
        tuning.speed = speed;
        tuning.acceleration = acceleration;
        tuning.duration = duration;
        tuning.limit = limit;
        tuning
    }
}

pub struct HuntLab {
    pub tuning: hunt::Tuning,
    pub selected: Option<Contact>,
    pub psychological: bool,
    pub session: Session,
    pub last: Option<Contact>,
    pub distance: f32,
}

impl Default for HuntLab {
    fn default() -> Self {
        Self {
            tuning: default(),
            selected: None,
            psychological: true,
            session: Session::default(),
            last: None,
            distance: 0.,
        }
    }
}

impl HuntLab {
    pub fn reset(&mut self) {
        self.session = Session {
            screen: Screen::Playing,
            ..default()
        };
        let post = crate::scene::station::RETURN_POST;
        use ainda_vivo_core::story::Action::*;
        for action in [TakeFuse, InspectPump, RestorePower] {
            self.session.expedition.act(action, post);
        }
        self.last = None;
    }

    pub fn contact(&mut self) -> Contact {
        let effect = self.selected.unwrap_or_else(|| {
            hunt::select_contact(
                0.35,
                self.session.expedition.can_capture(),
                self.session.psychology.value(),
            )
        });
        let effect = hunt::apply(&mut self.session, effect, self.psychological);
        self.last = Some(effect);
        effect
    }
}

pub fn diagnostics(lab: &TestLab) -> String {
    let b = &lab.brain;
    let h = &b.hunt;
    let phase = match h.phase {
        Phase::Idle => b.state.label(),
        Phase::Prepare => "Preparação",
        Phase::Run => h.strategy.label(),
        Phase::Brake => "Freada",
        Phase::Rise => "Retorno em pé",
        Phase::Flinch => "Intimidação no contato",
    };
    format!(
        "{phase} · {:.2} m/s\nDistância {:.1} m · {} pontos\nEncerramento · {}\nContato · {}\nInstabilidade {:.1} · {:?}",
        b.moving,
        lab.hunt.distance,
        b.path.len().saturating_sub(b.waypoint),
        h.end.map_or("—", End::label),
        lab.hunt.last.map_or("—", Contact::label),
        lab.hunt.session.psychology.value(),
        lab.hunt.session.expedition.observer().reality()
    )
}

pub fn hunt_command(active: &mut Active, config: &mut Config, action: HuntCommand) {
    match action {
        HuntCommand::Profile(profile) => active.observer.hunt.tuning = profile.tuning(),
        HuntCommand::Start | HuntCommand::Bluff | HuntCommand::DoorTrial => {
            if !active.observer.enabled {
                command(active, config, Command::Enable);
            }
            config.tab = Tab::Hunt;
            let corridor = action == HuntCommand::DoorTrial;
            let camera = if corridor {
                Vec2::new(4.60, -4.55)
            } else {
                Vec2::new(-6.38, -1.62)
            };
            config.lens.free = false;
            config.lens.position = [camera.x, 1.622, camera.y];
            config.lens.yaw = 0.0;
            config.lens.pitch = -0.10;
            active.reset_view(&config.lens);
            let lab = &mut active.observer;
            lab.bot = None;
            lab.automatic = false;
            lab.closed = false;
            lab.paused = false;
            lab.hunt.reset();
            let start = if corridor {
                lab.hunt.tuning.start.min(8.2)
            } else {
                lab.hunt.tuning.start
            };
            lab.brain = Brain::seeded(lab.seed.unwrap_or(4141));
            lab.brain.spawn(camera - Vec2::Y * start, 0.0);
            lab.brain.change(
                State::Threaten,
                "ensaio da caçada",
                lab.hunt.tuning.prepare,
                None,
            );
            lab.brain.hunt = Hunt::start(lab.hunt.tuning, camera, action == HuntCommand::Bluff);
            lab.brain.hunt.guided = true;
        }
        HuntCommand::Stop => active.observer.brain.hunt.stop(End::Interrupted),
        HuntCommand::Rehearse => {
            if !active.observer.enabled {
                command(active, config, Command::Enable);
            }
            config.tab = Tab::Hunt;
            config.lens.free = false;
            config.lens.position = [-6.38, 1.622, -1.62];
            config.lens.yaw = 0.;
            config.lens.pitch = -0.10;
            active.reset_view(&config.lens);
            let lab = &mut active.observer;
            lab.bot = None;
            lab.automatic = false;
            lab.paused = false;
            lab.hunt.reset();
            let effect = lab.hunt.contact();
            lab.brain = Brain::seeded(lab.seed.unwrap_or(4141));
            lab.brain.spawn(Vec2::new(-6.38, -3.40), 0.0);
            lab.brain.hunt = Hunt::preview(effect);
        }
        HuntCommand::Result(result) => active.observer.hunt.selected = result,
        HuntCommand::Psychology => {
            active.observer.hunt.psychological = !active.observer.hunt.psychological;
        }
        HuntCommand::Adjust(knob, delta) => {
            let (v, min, max, _) = knob.setting(&mut active.observer.hunt.tuning);
            *v = (*v + delta).clamp(min, max);
        }
    }
}
