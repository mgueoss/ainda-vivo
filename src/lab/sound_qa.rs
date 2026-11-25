use super::{Action, Lab, Options, sound_lab::Command};
use crate::app::{Comfort, LocalPlayer, Session, restart};
use crate::atmosphere::sound::presence_audio::{
    Category, Family, MAX_VOICES, Pattern, PresenceEmitter,
};
use bevy::{
    app::AppExit,
    audio::AudioSinkPlayback,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

#[derive(Default, Resource)]
pub(super) struct Check {
    frame: u32,
    closed: Option<(f32, f32, f32)>,
    open: Option<(f32, f32, f32)>,
    hand: bool,
    foot: bool,
    voice: bool,
    walk: bool,
    ghost_hand: bool,
    ghost_foot: bool,
    false_world: bool,
    device: bool,
    peak: usize,
    initial: Option<(u64, u32, f32)>,
    clean: bool,
    unchanged: bool,
    route: Option<(f32, u8, u8)>,
}

pub(super) fn keys(check: Option<Res<Check>>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    if let Some(check) = check {
        if matches!(check.frame, 40 | 980) {
            keys.press(KeyCode::ControlLeft);
            keys.press(KeyCode::KeyI);
        } else if matches!(check.frame, 41 | 981) {
            keys.release(KeyCode::ControlLeft);
            keys.release(KeyCode::KeyI);
        }
    }
}

pub(super) fn verify(
    mut check: ResMut<Check>,
    options: Res<Options>,
    lab: Res<Lab>,
    (mut session, mut player): (ResMut<Session>, ResMut<LocalPlayer>),
    mut comfort: ResMut<Comfort>,
    sources: Query<(&PresenceEmitter, Option<&AudioSink>)>,
    (mut commands, mut actions, mut exit): (
        Commands,
        MessageWriter<Action>,
        MessageWriter<AppExit>,
    ),
) {
    let c = &mut *check;
    c.frame += 1;
    let step = c.frame;
    let stamp = (
        session.attempt,
        session.perception_revision,
        session.psychology.value(),
    );
    if step == 1 {
        comfort.ambience_muted = false;
        comfort.ambience_volume = 0.8;
        comfort.psycho_av = 1.0;
        c.initial = Some(stamp);
    }
    let event = match step {
        55 | 120 => Some(Command::Door),
        63 | 126 => Some(Command::Play),
        135 => Some(Command::Guides),
        185 => Some(Command::Walk),
        340 => Some(Command::Chase),
        620 => Some(Command::Pattern(Pattern::Rush)),
        926 | 927 => Some(Command::Category),
        932 => Some(Command::Family(Family::Throat)),
        _ => None,
    };
    if let Some(event) = event {
        actions.write(Action::Sound(event));
    }
    if step == 137
        && let Some(dir) = &options.captures
        && std::fs::create_dir_all(dir).is_ok()
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(dir.join("01-dev-lab-som.png")));
    }
    let mut count = 0;
    for (source, sink) in &sources {
        if !source.request.lab {
            if step > 1005
                && source.request.category == Category::Illusion
                && let (Some(listener), Some(origin)) = (
                    crate::scene::station::sector_id(player.body.feet),
                    crate::scene::station::sector_id(source.request.at),
                )
                && listener != origin
                && source.audible > 0.01
            {
                c.route.get_or_insert((session.elapsed, listener, origin));
            }
            continue;
        }
        count += 1;
        c.device |= sink.is_some_and(|sink| !sink.empty() && !sink.is_paused());
        let f = source.request.family;
        let level = (source.transmission, source.muffle, source.audible);
        if (66..=84).contains(&step) {
            c.closed = Some(level);
        }
        if (130..=148).contains(&step) {
            c.open = Some(level);
        }
        match source.request.category {
            Category::Physical if (180..339).contains(&step) => {
                c.walk |= matches!(f, Family::Walk | Family::Hurry | Family::Approach)
            }
            Category::Physical if (342..615).contains(&step) => {
                c.hand |= f == Family::Hand;
                c.foot |= f == Family::Foot;
                c.voice |= f.vocal();
            }
            Category::Illusion => {
                c.ghost_hand |= f == Family::Hand;
                c.ghost_foot |= f == Family::Foot;
            }
            Category::FalseReality => c.false_world |= f == Family::Throat,
            _ => {}
        }
    }
    c.peak = c.peak.max(count);
    if step == 1005 {
        c.clean = count == 0 && lab.active.is_none();
        c.unchanged = Some(stamp) == c.initial;
        restart(&mut session, &mut player, &mut comfort);
        player.body.feet = Vec3::new(0., 0.002, -6.);
        player.body.previous_feet = player.body.feet;
        player.yaw = std::f32::consts::PI;
        player.pitch = 0.0;
    }
    if step <= 1005 || c.route.is_none() && step < 8200 {
        return;
    }
    let door = c
        .closed
        .zip(c.open)
        .is_some_and(|(a, b)| a.0 < b.0 && a.1 > b.1 && a.2 < b.2);
    let passed = door
        && c.walk
        && c.hand
        && c.foot
        && c.voice
        && c.ghost_hand
        && c.ghost_foot
        && c.false_world
        && c.device
        && c.peak <= MAX_VOICES
        && c.clean
        && c.unchanged
        && c.route.is_some();
    let report = format!(
        "{} {}\nporta fechada (transmissão,filtro,nível) {:?}\nporta aberta {:?}\ncaminhada={} mão={} pé={} voz={} ilusão_mão={} ilusão_pé={} falsa_realidade={} dispositivo={} pico_de_fontes={} limpo={} campanha_intacta={}\nEstação (tempo,setor ouvinte,setor fonte) {:?}\n",
        crate::app::VERSION,
        if passed { "PASS" } else { "FAIL" },
        c.closed,
        c.open,
        c.walk,
        c.hand,
        c.foot,
        c.voice,
        c.ghost_hand,
        c.ghost_foot,
        c.false_world,
        c.device,
        c.peak,
        c.clean,
        c.unchanged,
        c.route
    );
    info!("{report}");
    if let Some(path) = std::env::args_os()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "--smoke-report")
        .map(|w| w[1].clone())
        && let Err(err) = std::fs::write(path, &report)
    {
        error!("{err}");
    }
    exit.write(if passed {
        AppExit::Success
    } else {
        AppExit::error()
    });
}
