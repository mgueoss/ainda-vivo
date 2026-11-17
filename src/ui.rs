use crate::{app::*, diagnostics::Performance};
use bevy::{app::AppExit, prelude::*};

const INK: Color = Color::srgb(0.84, 0.87, 0.81);
const MUTED: Color = Color::srgb(0.50, 0.60, 0.56);
const BUTTON: Color = Color::srgb(0.10, 0.15, 0.14);

#[derive(Component, Clone, Copy)]
pub enum MenuAction {
    Main,
    Restart,
    Bodycam,
    Vsync,
    FovDown,
    FovUp,
    SensDown,
    SensUp,
    Quit,
}

#[derive(Component)]
pub enum TextRole {
    Heading,
    Subtitle,
    Button(MenuAction),
    Settings,
    Recording,
    Objective,
    Prompt,
    Message,
    Performance,
}

#[derive(Component)]
pub enum Root {
    Menu,
    Hud,
}

struct UiStyle(Handle<Font>);

impl UiStyle {
    fn text(&self, content: &str, size: f32, color: Color) -> impl Bundle + use<> {
        (
            Text::new(content),
            TextFont {
                font: self.0.clone(),
                font_size: size,
                ..default()
            },
            TextColor(color),
        )
    }

    fn button(&self, action: MenuAction, label: &str, small: bool) -> impl Bundle + use<> {
        (
            Button,
            action,
            Node {
                min_height: px(if small { 32.0 } else { 38.0 }),
                padding: UiRect::axes(px(16), px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 1.0,
                ..default()
            },
            BackgroundColor(BUTTON),
            children![(self.text(label, 15.0, INK), TextRole::Button(action))],
        )
    }
}

pub fn setup(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    // Read an installed font at runtime. No proprietary font is bundled or copied.
    let windows = std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into());
    let directory = std::path::PathBuf::from(windows).join("Fonts");
    let font = ["consola.ttf", "segoeui.ttf"].into_iter().find_map(|name| {
        std::fs::read(directory.join(name))
            .ok()
            .and_then(|bytes| Font::try_from_bytes(bytes).ok())
    });
    let handle = if let Some(font) = font {
        fonts.add(font)
    } else {
        warn!("Fonte local indisponível; usando a fonte limitada do motor.");
        Handle::default()
    };
    let style = UiStyle(handle);
    commands.spawn((Root::Menu, Node {
        width: percent(100), height: percent(100), position_type: PositionType::Absolute,
        justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()
    }, GlobalZIndex(10), BackgroundColor(Color::srgba(0.014, 0.027, 0.027, 0.97)), children![(
        Node { width: percent(92), max_width: px(660), padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column, row_gap: px(8), ..default() },
        children![
            style.text("●  REGISTRO EXPERIMENTAL  /  v0.0.2", 13.0, Color::srgb(0.70, 0.31, 0.25)),
            (style.text("AINDA VIVO", 52.0, INK), TextRole::Heading),
            (style.text("Não siga a sombra.", 17.0, MUTED), TextRole::Subtitle),
            style.text("WASD  mover    MOUSE  olhar    SHIFT  correr    CTRL/C  agachar\nE  interagir    F  lanterna    ESPAÇO  pular    ESC  pausar    F3  diagnóstico", 14.0, MUTED),
            style.button(MenuAction::Main, "INICIAR EXPEDIÇÃO   [ENTER]", false),
            style.button(MenuAction::Restart, "REINICIAR EXPEDIÇÃO", true),
            (Node { column_gap: px(8), ..default() }, children![
                style.button(MenuAction::Bodycam, "BALANÇO: DESLIGADO", true),
                style.button(MenuAction::Vsync, "VSYNC: LIGADO", true),
            ]),
            (style.text("", 14.0, MUTED), TextRole::Settings),
            (Node { column_gap: px(8), ..default() }, children![
                style.button(MenuAction::FovDown, "VISÃO −", true),
                style.button(MenuAction::FovUp, "VISÃO +", true),
                style.button(MenuAction::SensDown, "MOUSE −", true),
                style.button(MenuAction::SensUp, "MOUSE +", true),
            ]),
            style.button(MenuAction::Quit, "SAIR", true),
            style.text("Um corredor. Um fusível. Uma saída.\nProtótipo solo • sem áudio e sem sustos bruscos • conforto primeiro", 13.0, MUTED),
        ]
    )]));
    commands.spawn((
        Root::Hud,
        Node {
            width: percent(100),
            height: percent(100),
            position_type: PositionType::Absolute,
            display: Display::None,
            ..default()
        },
        children![
            (
                style.text("", 16.0, INK),
                TextRole::Recording,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(22),
                    left: px(26),
                    ..default()
                }
            ),
            (
                style.text("", 14.0, MUTED),
                TextRole::Performance,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(22),
                    right: px(26),
                    ..default()
                }
            ),
            (
                style.text("·", 24.0, Color::srgba(0.9, 0.94, 0.9, 0.75)),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    top: percent(49),
                    ..default()
                }
            ),
            (
                style.text("", 16.0, INK),
                TextRole::Objective,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(26),
                    bottom: px(85),
                    max_width: percent(85),
                    ..default()
                }
            ),
            (
                style.text("", 18.0, Color::srgb(0.93, 0.77, 0.43)),
                TextRole::Prompt,
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(36),
                    top: percent(62),
                    ..default()
                }
            ),
            (
                style.text("", 16.0, INK),
                TextRole::Message,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(26),
                    right: px(26),
                    bottom: px(52),
                    ..default()
                }
            ),
            (
                style.text("E  INTERAGIR     F  LANTERNA     ESC  PAUSA", 12.0, MUTED),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(26),
                    bottom: px(24),
                    ..default()
                }
            ),
        ],
    ));
}

pub fn buttons(
    mut query: Query<(&Interaction, &MenuAction, &mut BackgroundColor), Changed<Interaction>>,
    mut session: ResMut<Session>,
    mut player: ResMut<LocalPlayer>,
    mut comfort: ResMut<Comfort>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action, mut color) in &mut query {
        *color = BackgroundColor(match interaction {
            Interaction::Hovered => Color::srgb(0.18, 0.25, 0.22),
            Interaction::Pressed => Color::srgb(0.28, 0.36, 0.29),
            Interaction::None => BUTTON,
        });
        if *interaction != Interaction::Pressed || session.screen == Screen::Playing {
            continue;
        }
        match action {
            MenuAction::Main if session.screen == Screen::Paused => {
                session.screen = Screen::Playing
            }
            MenuAction::Main | MenuAction::Restart => {
                restart(&mut session, &mut player, &mut comfort)
            }
            MenuAction::Bodycam => comfort.bodycam = !comfort.bodycam,
            MenuAction::Vsync => comfort.vsync = !comfort.vsync,
            MenuAction::FovDown => comfort.fov_degrees = (comfort.fov_degrees - 5.0).max(55.0),
            MenuAction::FovUp => comfort.fov_degrees = (comfort.fov_degrees + 5.0).min(80.0),
            MenuAction::SensDown => {
                comfort.sensitivity = (comfort.sensitivity - 0.0002).max(0.0004)
            }
            MenuAction::SensUp => comfort.sensitivity = (comfort.sensitivity + 0.0002).min(0.004),
            MenuAction::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

pub fn refresh(
    session: Res<Session>,
    comfort: Res<Comfort>,
    performance: Res<Performance>,
    mut roots: Query<(&Root, &mut Node)>,
    mut labels: Query<(&TextRole, &mut Text)>,
) {
    for (root, mut node) in &mut roots {
        let shown = match root {
            Root::Menu => session.screen != Screen::Playing,
            Root::Hud => session.screen == Screen::Playing,
        };
        let display = if shown { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
    for (role, mut label) in &mut labels {
        let value = match role {
            TextRole::Heading => match session.screen {
                Screen::Paused => "REGISTRO PAUSADO".into(),
                Screen::Complete => "AINDA VIVO".into(),
                _ => "AINDA VIVO".into(),
            },
            TextRole::Subtitle => match session.screen {
                Screen::Complete => format!(
                    "Saída alcançada em {:.0}s. O registro continua na próxima versão.",
                    session.elapsed
                ),
                Screen::Paused => "Respire. O corredor pode esperar.".into(),
                _ => "Não siga a sombra.".into(),
            },
            TextRole::Button(action) => match action {
                MenuAction::Main => if session.screen == Screen::Paused {
                    "CONTINUAR   [ENTER / ESC]"
                } else {
                    "INICIAR EXPEDIÇÃO   [ENTER]"
                }
                .into(),
                MenuAction::Bodycam => format!(
                    "BALANÇO: {}",
                    if comfort.bodycam {
                        "SUAVE"
                    } else {
                        "DESLIGADO"
                    }
                ),
                MenuAction::Vsync => format!(
                    "VSYNC: {}",
                    if comfort.vsync { "LIGADO" } else { "DESLIGADO" }
                ),
                _ => continue,
            },
            TextRole::Settings => format!(
                "Campo de visão vertical: {:.0}°    |    Sensibilidade: {:.1}    |    Balanço máximo: 6 mm",
                comfort.fov_degrees,
                comfort.sensitivity * 1000.0
            ),
            TextRole::Recording => format!(
                "● REC   {:02}:{:02}   /   AINDA VIVO",
                session.elapsed as u32 / 60,
                session.elapsed as u32 % 60
            ),
            TextRole::Objective => session.progress.objective().into(),
            TextRole::Prompt => session.prompt.clone(),
            TextRole::Message => {
                if session.message_left > 0.0 {
                    session.message.into()
                } else {
                    String::new()
                }
            }
            TextRole::Performance => {
                if comfort.diagnostics {
                    performance.text.clone()
                } else {
                    String::new()
                }
            }
        };
        if label.0 != value {
            label.0 = value;
        }
    }
}
