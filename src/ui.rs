//! Presentation for the title, menus and the in-game view.
mod menu;
pub use menu::{
    Action as MenuAction, Menu, Page as MenuPage, input as menu_input, refresh as refresh_menu,
};

use crate::{app::*, diagnostics::Performance, preferences::Preferences, psychology::Effects};
use bevy::{prelude::*, text::FontWeight};

const INK: Color = Color::srgb(0.84, 0.87, 0.81);
const MUTED: Color = Color::srgb(0.50, 0.60, 0.56);

#[derive(Component)]
pub enum HudLabel {
    Recording,
    Objective,
    Prompt,
    Message,
    Performance,
    Psychology,
}

#[derive(Component)]
pub struct Hud;

struct UiStyle {
    body: Handle<Font>,
    display: Handle<Font>,
    mono: Handle<Font>,
}

impl UiStyle {
    fn installed(fonts: &mut Assets<Font>) -> Self {
        // Use fonts already installed on the player's PC. The portable build carries only art.
        let windows = std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into());
        let directory = std::path::PathBuf::from(windows).join("Fonts");
        let mut load = |names: &[&str]| {
            names
                .iter()
                .find_map(|name| {
                    Font::try_from_bytes(std::fs::read(directory.join(name)).ok()?).ok()
                })
                .map(|font| fonts.add(font))
                .unwrap_or_default()
        };
        Self {
            body: load(&["segoeui.ttf", "consola.ttf"]),
            display: load(&["bahnschrift.ttf", "segoeuib.ttf", "consolab.ttf"]),
            mono: load(&["consola.ttf", "segoeui.ttf"]),
        }
    }

    fn body(&self, content: &str, size: f32, color: Color) -> impl Bundle + use<> {
        (
            Text::new(content),
            TextFont::from_font_size(size).with_font(self.body.clone()),
            TextColor(color),
        )
    }

    fn display(&self, content: &str, size: f32, color: Color) -> impl Bundle + use<> {
        (
            Text::new(content),
            TextFont {
                font: self.display.clone(),
                font_size: size,
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(color),
        )
    }

    fn mono(&self, content: &str, size: f32, color: Color) -> impl Bundle + use<> {
        (
            Text::new(content),
            TextFont::from_font_size(size).with_font(self.mono.clone()),
            TextColor(color),
        )
    }
}

pub fn setup(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    mut images: ResMut<Assets<Image>>,
    bindings: Res<Preferences>,
) {
    let style = UiStyle::installed(&mut fonts);
    menu::spawn(&mut commands, &mut images, &style, &bindings);
    commands.spawn((
        Hud,
        Node {
            width: percent(100),
            height: percent(100),
            position_type: PositionType::Absolute,
            display: Display::None,
            ..default()
        },
        children![
            (
                style.mono("", 16.0, INK),
                HudLabel::Recording,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(22),
                    left: px(26),
                    ..default()
                }
            ),
            (
                style.mono("", 14.0, MUTED),
                HudLabel::Performance,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(22),
                    right: px(26),
                    ..default()
                }
            ),
            (
                style.body("", 16.0, Color::srgb(0.93, 0.77, 0.53)),
                HudLabel::Psychology,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(51),
                    left: px(26),
                    max_width: percent(60),
                    ..default()
                }
            ),
            (
                style.mono("·", 24.0, Color::srgba(0.9, 0.94, 0.9, 0.75)),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    top: percent(49),
                    ..default()
                }
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(26),
                    right: px(26),
                    bottom: px(24),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                children![
                    (style.mono("", 16.0, INK), HudLabel::Objective),
                    (style.mono("", 16.0, MUTED), HudLabel::Message),
                    style.mono("E  INTERAGIR     F  LANTERNA     ESC  PAUSA", 12.0, MUTED),
                ],
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    top: percent(70),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    style.mono("", 17.0, Color::srgb(0.93, 0.77, 0.43)),
                    HudLabel::Prompt,
                )],
            ),
        ],
    ));
}

pub fn request_awakening(session: &mut Session) {
    if session.screen == Screen::Paused && !session.collapse_active() {
        session.wake_confirmation = true;
    }
}

pub fn refresh_hud(
    session: Res<Session>,
    comfort: Res<Comfort>,
    performance: Res<Performance>,
    effects: Res<Effects>,
    mut root: Query<&mut Node, With<Hud>>,
    mut labels: Query<(&HudLabel, &mut Text)>,
) {
    let visible = session.screen == Screen::Playing && effects.farewell.is_none();
    if let Ok(mut root) = root.single_mut() {
        menu::show(&mut root, visible);
    }
    if !visible {
        return;
    }
    for (role, mut label) in &mut labels {
        let value = match role {
            HudLabel::Recording => format!(
                "● REC   {:02}:{:02}   /   AINDA VIVO",
                session.elapsed as u32 / 60,
                session.elapsed as u32 % 60
            ),
            HudLabel::Objective => session.objective().into(),
            HudLabel::Prompt => session.prompt.clone(),
            HudLabel::Message if session.message_left > 0.0 => session.message.into(),
            HudLabel::Message => String::new(),
            HudLabel::Performance if comfort.diagnostics => performance.text.clone(),
            HudLabel::Performance => String::new(),
            HudLabel::Psychology if comfort.psycho_indicator => {
                instability_label(effects.stage, effects.remaining.map(|_| effects.critical)).into()
            }
            HudLabel::Psychology => String::new(),
        };
        if label.0 != value {
            label.0 = value;
        }
    }
}

fn instability_label(stage: u8, critical_progress: Option<f32>) -> &'static str {
    if let Some(progress) = critical_progress {
        return match progress {
            value if value < 0.25 => "Colapso · Procure um posto iluminado",
            value if value < 0.50 => "Colapso · Procure abrigo agora",
            value if value < 0.75 => "Colapso · Recuperação urgente",
            _ => "Colapso · Recuperação imediata",
        };
    }
    match stage {
        0 => "Estável",
        1 => "Inquietação",
        2 => "Paranoia",
        3 => "Desorientação",
        4 => "Pré-colapso",
        _ => "Colapso",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_indicator_reports_urgency_instead_of_a_precise_timer() {
        assert_eq!(instability_label(4, None), "Pré-colapso");
        assert_eq!(
            instability_label(5, Some(0.20)),
            "Colapso · Procure um posto iluminado"
        );
        assert_eq!(
            instability_label(5, Some(0.50)),
            "Colapso · Recuperação urgente"
        );
        assert_eq!(
            instability_label(5, Some(0.99)),
            "Colapso · Recuperação imediata"
        );
    }
}
