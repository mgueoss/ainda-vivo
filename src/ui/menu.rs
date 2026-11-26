//! Menu navigation lives outside the expedition so opening a panel cannot alter its state.
mod layout;
pub(super) use layout::spawn;

use crate::{app::*, smoke::SmokeTest};
use ainda_vivo_core::{memory::ServiceTrail, reality::Outcome};
use bevy::{
    app::AppExit, camera::visibility::RenderLayers, prelude::*, ui::UiScale, window::PrimaryWindow,
};

const INK: Color = Color::srgb(0.94, 0.91, 0.85);
const MUTED: Color = Color::srgb(0.69, 0.71, 0.66);
const AMBER: Color = Color::srgb(0.86, 0.59, 0.44);
const EDGE: Color = Color::srgb(0.24, 0.28, 0.25);
const FOCUS: Color = Color::srgb(0.87, 0.65, 0.50);

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Action {
    #[default]
    Main,
    Settings,
    Accessibility,
    Controls,
    Back,
    Restart,
    Wake,
    CancelWake,
    ConfirmWake,
    FovDown,
    FovUp,
    SensDown,
    SensUp,
    VolumeDown,
    VolumeUp,
    Mute,
    Bodycam,
    Vhs,
    Vsync,
    PsychologicalEffectsDown,
    PsychologicalEffectsUp,
    CrisisCamera,
    StabilityIndicator,
    Quit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Home,
    Settings,
    Accessibility,
    Controls,
}

#[derive(Resource)]
pub struct Menu {
    pub page: Page,
    pub focus: Action,
    keyboard: bool,
    screen: Screen,
    confirmation: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            page: Page::Home,
            focus: Action::Main,
            keyboard: true,
            screen: Screen::Menu,
            confirmation: false,
        }
    }
}

impl Menu {
    fn sync(&mut self, session: &Session) {
        if self.screen != session.screen {
            *self = Self {
                screen: session.screen,
                ..default()
            };
        }
        if self.confirmation != session.wake_confirmation {
            self.confirmation = session.wake_confirmation;
            self.focus = if self.confirmation {
                Action::CancelWake
            } else {
                Action::Wake
            };
        }
    }

    fn choices(&self, session: &Session) -> &'static [Action] {
        use Action::*;
        if session.wake_confirmation {
            return &[CancelWake, ConfirmWake];
        }
        match self.page {
            Page::Settings => &[
                FovDown,
                FovUp,
                SensDown,
                SensUp,
                Bodycam,
                Vhs,
                Vsync,
                VolumeDown,
                VolumeUp,
                Mute,
                Accessibility,
                Back,
            ],
            Page::Accessibility => &[
                PsychologicalEffectsDown,
                PsychologicalEffectsUp,
                CrisisCamera,
                StabilityIndicator,
                Back,
            ],
            Page::Controls => &[Back],
            Page::Home if session.screen == Screen::Paused && session.collapse_active() => {
                &[Main, Settings, Controls, Restart, Quit]
            }
            Page::Home if session.screen == Screen::Paused => {
                &[Main, Settings, Controls, Restart, Wake, Quit]
            }
            Page::Home if session.expedition.can_resume() => {
                &[Main, Settings, Controls, Restart, Quit]
            }
            Page::Home => &[Main, Settings, Controls, Quit],
        }
    }

    fn shift_focus(&mut self, choices: &[Action], reverse: bool) {
        let index = choices
            .iter()
            .position(|choice| *choice == self.focus)
            .unwrap_or(0);
        let next = if reverse {
            (index + choices.len() - 1) % choices.len()
        } else {
            (index + 1) % choices.len()
        };
        self.focus = choices[next];
        self.keyboard = true;
    }

    fn back(&mut self) {
        (self.page, self.focus) = match self.page {
            Page::Accessibility => (Page::Settings, Action::Accessibility),
            Page::Settings => (Page::Home, Action::Settings),
            Page::Controls => (Page::Home, Action::Controls),
            Page::Home => (Page::Home, Action::Main),
        };
    }
}

pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    (window, smoke): (Query<&Window, With<PrimaryWindow>>, Res<SmokeTest>),
    buttons: Query<(&Action, &Interaction), Changed<Interaction>>,
    mut menu: ResMut<Menu>,
    (mut session, mut player, mut comfort): (ResMut<Session>, ResMut<LocalPlayer>, ResMut<Comfort>),
    mut exit: MessageWriter<AppExit>,
) {
    menu.sync(&session);
    if !smoke.menu
        && !smoke.campaign
        && (smoke.enabled || !window.single().is_ok_and(|window| window.focused))
    {
        return;
    }
    let escape = keys.just_pressed(KeyCode::Escape);
    if session.screen == Screen::Playing {
        if escape {
            session.screen = Screen::Paused;
            player.command = default();
            player.look_armed = false;
        }
        return;
    }
    if escape {
        menu.keyboard = true;
        if session.wake_confirmation {
            session.wake_confirmation = false;
        } else if menu.page != Page::Home {
            menu.back();
        } else if session.screen == Screen::Paused {
            session.screen = Screen::Playing;
        }
        return;
    }

    let choices = menu.choices(&session);
    let reverse = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let move_focus = keys.any_just_pressed([
        KeyCode::Tab,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ]);
    let chosen = if move_focus {
        let backwards = keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowLeft])
            || (keys.just_pressed(KeyCode::Tab) && reverse);
        menu.shift_focus(choices, backwards);
        None
    } else if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        menu.keyboard = true;
        Some(if choices.contains(&menu.focus) {
            menu.focus
        } else {
            choices[0]
        })
    } else {
        let mut pressed = None;
        for (action, interaction) in &buttons {
            if *interaction != Interaction::None && choices.contains(action) {
                menu.focus = *action;
                menu.keyboard = false;
                if *interaction == Interaction::Pressed {
                    pressed = Some(*action);
                }
            }
        }
        pressed
    };

    match chosen {
        Some(Action::Main) if session.screen == Screen::Paused => session.screen = Screen::Playing,
        Some(Action::Main) => continue_or_restart(&mut session, &mut player, &mut comfort),
        Some(Action::Restart) => restart(&mut session, &mut player, &mut comfort),
        Some(Action::Settings) => {
            menu.page = Page::Settings;
            menu.focus = Action::FovDown;
        }
        Some(Action::Accessibility) => {
            menu.page = Page::Accessibility;
            menu.focus = Action::PsychologicalEffectsDown;
        }
        Some(Action::Controls) => {
            menu.page = Page::Controls;
            menu.focus = Action::Back;
        }
        Some(Action::Back) => menu.back(),
        Some(Action::Wake) => super::request_awakening(&mut session),
        Some(Action::CancelWake) => session.wake_confirmation = false,
        Some(Action::ConfirmWake) => awaken(&mut session, &mut player),
        Some(Action::FovDown) => comfort.fov_degrees = (comfort.fov_degrees - 5.0).max(55.0),
        Some(Action::FovUp) => comfort.fov_degrees = (comfort.fov_degrees + 5.0).min(80.0),
        Some(Action::SensDown) => comfort.sensitivity = (comfort.sensitivity - 0.0002).max(0.0004),
        Some(Action::SensUp) => comfort.sensitivity = (comfort.sensitivity + 0.0002).min(0.004),
        Some(Action::VolumeDown) => {
            comfort.ambience_volume = (comfort.ambience_volume - 0.10).max(0.0)
        }
        Some(Action::VolumeUp) => {
            comfort.ambience_volume = (comfort.ambience_volume + 0.10).min(1.0);
            comfort.ambience_muted = false;
        }
        Some(Action::Mute) => comfort.ambience_muted = !comfort.ambience_muted,
        Some(Action::Bodycam) => comfort.bodycam = !comfort.bodycam,
        Some(Action::Vhs) => comfort.vhs = comfort.vhs.next(),
        Some(Action::Vsync) => comfort.vsync = !comfort.vsync,
        Some(Action::PsychologicalEffectsDown) => {
            comfort.psycho_av = (comfort.psycho_av - 0.25).max(0.0);
        }
        Some(Action::PsychologicalEffectsUp) => {
            comfort.psycho_av = (comfort.psycho_av + 0.25).min(1.0);
        }
        Some(Action::CrisisCamera) => comfort.psycho_motion = !comfort.psycho_motion,
        Some(Action::StabilityIndicator) => comfort.psycho_indicator = !comfort.psycho_indicator,
        Some(Action::Quit) => {
            exit.write(AppExit::Success);
        }
        None => (),
    }
}

#[derive(Component)]
pub enum Region {
    Deck,
    Photo,
    Gameplay,
    Home,
    Title,
    Record,
    MainShortcut,
    Settings,
    Accessibility,
    Controls,
    Restart,
    WakeAction,
    Confirmation,
}

#[derive(Component)]
pub struct Artwork(Vec2);

#[derive(Component, Clone, Copy)]
pub enum Label {
    Outcome,
    Summary,
    Main,
    Hint,
    Fov,
    Sensitivity,
    Volume,
    Mute,
    Bodycam,
    Vhs,
    Vsync,
    PsychologicalEffects,
    CrisisCamera,
    StabilityIndicator,
}

#[derive(Component, Clone, Copy)]
pub enum Tone {
    Main,
    Row,
    Control,
    Warning,
}

impl Tone {
    fn palette(self, highlighted: bool, pressed: bool) -> (Color, Color) {
        let (background, border) = match self {
            Self::Main if pressed => (Color::srgb(0.60, 0.30, 0.22), FOCUS),
            Self::Main if highlighted => (Color::srgb(0.54, 0.28, 0.21), FOCUS),
            Self::Main => (Color::srgb(0.46, 0.23, 0.18), Color::srgb(0.71, 0.42, 0.33)),
            Self::Row if highlighted => (Color::srgba(0.16, 0.18, 0.16, 0.96), FOCUS),
            Self::Row => (Color::srgba(0.065, 0.08, 0.071, 0.84), EDGE),
            Self::Control if highlighted => (Color::srgb(0.22, 0.23, 0.19), FOCUS),
            Self::Control => (Color::srgb(0.105, 0.125, 0.11), EDGE),
            Self::Warning if highlighted => (Color::srgb(0.26, 0.16, 0.13), FOCUS),
            Self::Warning => (
                Color::srgb(0.13, 0.09, 0.075),
                Color::srgb(0.34, 0.21, 0.18),
            ),
        };
        (background, border)
    }
}

pub(super) fn show(node: &mut Node, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}

fn cover(view: Vec2, image: Vec2) -> Rect {
    let size = view * (image / view).min_element();
    let start = (image - size).max(Vec2::ZERO) * Vec2::new(0.84, 0.5);
    Rect::from_corners(start, (start + size).min(image))
}

type MenuButtons<'w, 's> = Query<
    'w,
    's,
    (
        &'static Action,
        &'static Tone,
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
>;

type Viewport<'w, 's> = (
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    ResMut<'w, UiScale>,
    Query<'w, 's, &'static mut RenderLayers, With<MainCamera>>,
);

pub fn refresh(
    (session, comfort, mut menu): (Res<Session>, Res<Comfort>, ResMut<Menu>),
    mut regions: Query<(&Region, &mut Node)>,
    mut labels: Query<(&Label, &mut Text)>,
    mut buttons: MenuButtons,
    mut artwork: Query<(&Artwork, &mut ImageNode)>,
    (window, mut scale, mut camera): Viewport,
) {
    menu.sync(&session);
    let showing = session.screen != Screen::Playing;
    let title = session.screen == Screen::Menu;
    let showing_page = showing && !session.wake_confirmation;
    if let Ok(mut layers) = camera.single_mut() {
        let desired = if title {
            RenderLayers::none()
        } else {
            RenderLayers::default()
        };
        if *layers != desired {
            *layers = desired;
        }
    }
    if let Ok(window) = window.single() {
        let view = Vec2::new(window.width(), window.height());
        if view.min_element() > 0.0 {
            let factor = if showing {
                (view / Vec2::new(860.0, 720.0))
                    .min_element()
                    .clamp(0.70, 1.18)
            } else {
                1.0
            };
            if scale.0 != factor {
                scale.0 = factor;
            }
            if title {
                for (art, mut image) in &mut artwork {
                    let crop = Some(cover(view, art.0));
                    if image.rect != crop {
                        image.rect = crop;
                    }
                }
            }
        }
    }
    for (region, mut node) in &mut regions {
        show(
            &mut node,
            match region {
                Region::Deck => showing,
                Region::Photo | Region::Title => title,
                Region::Gameplay | Region::Record => !title,
                Region::Home => showing_page && menu.page == Page::Home,
                Region::MainShortcut => menu.focus == Action::Main,
                Region::Settings => showing_page && menu.page == Page::Settings,
                Region::Accessibility => showing_page && menu.page == Page::Accessibility,
                Region::Controls => showing_page && menu.page == Page::Controls,
                Region::Restart => {
                    session.screen == Screen::Paused || session.expedition.can_resume()
                }
                Region::WakeAction => {
                    session.screen == Screen::Paused && !session.collapse_active()
                }
                Region::Confirmation => showing && session.wake_confirmation,
            },
        );
    }
    if !showing {
        return;
    }
    for (action, tone, interaction, mut color, mut border) in &mut buttons {
        let pressed = *interaction == Interaction::Pressed;
        let highlighted = pressed
            || if menu.keyboard {
                menu.focus == *action
            } else {
                *interaction == Interaction::Hovered
            };
        let (background, edge) = tone.palette(highlighted, pressed);
        if color.0 != background {
            color.0 = background;
        }
        if *border != BorderColor::all(edge) {
            *border = BorderColor::all(edge);
        }
    }
    for (role, mut text) in &mut labels {
        let value = match role {
            Label::Outcome => if session.screen == Screen::Paused {
                "Pausa"
            } else if session.collapse_finished() {
                "Registro perdido"
            } else {
                "Registro encerrado"
            }
            .into(),
            Label::Summary => record_summary(&session).into(),
            Label::Main => match session.screen {
                Screen::Paused => "Continuar",
                Screen::Complete if session.expedition.can_resume() => "Retomar do posto",
                Screen::Complete => "Nova expedição",
                _ => "Iniciar expedição",
            }
            .to_owned(),
            Label::Hint => if menu.page == Page::Controls {
                "Enter ou Esc  Voltar"
            } else if session.wake_confirmation || menu.page != Page::Home {
                "↑ ↓  Navegar     Enter  Selecionar     Esc  Voltar"
            } else if session.screen == Screen::Paused {
                "↑ ↓  Navegar     Enter  Selecionar     Esc  Continuar"
            } else {
                "↑ ↓  Navegar     Enter  Selecionar"
            }
            .into(),
            Label::Fov => format!("{:.0}°", comfort.fov_degrees),
            Label::Sensitivity => format!("{:.1}", comfort.sensitivity * 1000.0).replace('.', ","),
            Label::Volume => format!("{:.0}%", comfort.ambience_volume * 100.0),
            Label::Mute => if comfort.ambience_muted {
                "Mudo"
            } else {
                "Ligado"
            }
            .into(),
            Label::Bodycam => if comfort.bodycam {
                "Suave"
            } else {
                "Desligado"
            }
            .into(),
            Label::Vhs => comfort.vhs.label().into(),
            Label::Vsync => if comfort.vsync { "Ligado" } else { "Desligado" }.into(),
            Label::PsychologicalEffects => format!("{:.0}%", comfort.psycho_av * 100.0),
            Label::CrisisCamera => if comfort.psycho_motion {
                "Ativa"
            } else {
                "Reduzida"
            }
            .into(),
            Label::StabilityIndicator => if comfort.psycho_indicator {
                "Visível"
            } else {
                "Oculto"
            }
            .into(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

fn record_summary(session: &Session) -> &'static str {
    match session.screen {
        Screen::Paused => "",
        Screen::Complete if session.collapse_finished() => "A chamada do posto ficou sem resposta.",
        Screen::Complete if session.expedition.can_resume() => {
            "Registro interrompido. O posto conserva a última leitura."
        }
        Screen::Complete
            if session.expedition.observer().outcome() == Some(Outcome::Interrupted) =>
        {
            "Registro interrompido. Nenhuma baixa foi assinada."
        }
        Screen::Complete if session.memory().service == ServiceTrail::Reconciled => {
            "Vistoria arquivada. A via 17 segue assinada após o cancelamento."
        }
        Screen::Complete => "Vistoria arquivada. A câmera da nave continua gravando.",
        _ => "",
    }
}
