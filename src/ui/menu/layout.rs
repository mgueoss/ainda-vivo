use super::{AMBER, Action, Artwork, EDGE, INK, Label, MUTED, Region, Tone};
use crate::{app::VERSION, preferences::Preferences, ui::UiStyle};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::LineHeight,
    ui::{BackgroundGradient, ColorStop, LinearGradient, widget::NodeImageMode},
};
use zune_jpeg::{
    JpegDecoder,
    zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions},
};

const PAGE_BACKGROUND: Color = Color::srgb(0.018, 0.025, 0.022);
const SIDE: Val = Val::Percent(6.9);

pub(in crate::ui) fn spawn(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    style: &UiStyle,
    bindings: &Preferences,
) {
    let photo = load_art(images);
    commands
        .spawn((
            Name::new("Menu da Estação 07"),
            Region::Deck,
            full(),
            GlobalZIndex(10),
        ))
        .with_children(|deck| {
            deck.spawn((Region::Photo, full(), BackgroundColor(PAGE_BACKGROUND)))
                .with_children(|background| {
                    if let Some((image, size)) = photo {
                        background.spawn((
                            Artwork(size),
                            full(),
                            ImageNode {
                                image,
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                        ));
                    }
                    background.spawn((
                        full(),
                        BackgroundGradient(vec![
                            LinearGradient::to_right(vec![
                                ColorStop::percent(Color::srgba(0.01, 0.013, 0.012, 0.42), 0.0),
                                ColorStop::percent(Color::srgba(0.01, 0.013, 0.012, 0.29), 37.0),
                                ColorStop::percent(Color::NONE, 78.0),
                            ])
                            .into(),
                            LinearGradient::to_top(vec![
                                ColorStop::percent(Color::srgba(0.01, 0.012, 0.011, 0.74), 0.0),
                                ColorStop::percent(Color::NONE, 24.0),
                            ])
                            .into(),
                        ]),
                    ));
                });
            deck.spawn((
                Region::Gameplay,
                full(),
                BackgroundGradient::from(LinearGradient::to_right(vec![
                    ColorStop::percent(Color::srgba(0.01, 0.016, 0.014, 0.96), 0.0),
                    ColorStop::percent(Color::srgba(0.01, 0.016, 0.014, 0.88), 42.0),
                    ColorStop::percent(Color::srgba(0.01, 0.016, 0.014, 0.56), 100.0),
                ])),
            ));
            deck.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(23),
                    right: px(23),
                    top: px(23),
                    bottom: px(23),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.63, 0.61, 0.53, 0.19)),
            ));
            deck.spawn((
                Node {
                    width: percent(100),
                    height: percent(100),
                    padding: UiRect::axes(SIDE, px(54)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    home(style),
                    settings(style),
                    controls(style, bindings),
                    confirmation(style)
                ],
            ));
            deck.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: SIDE,
                    right: SIDE,
                    bottom: px(37),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                    column_gap: px(18),
                    ..default()
                },
                children![
                    (style.mono("", 13.0, MUTED), Label::Hint),
                    style.mono(&format!("v{VERSION}"), 13.0, INK),
                ],
            ));
        });
}

fn full() -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        position_type: PositionType::Absolute,
        ..default()
    }
}

fn column(width: f32, gap: f32) -> Node {
    Node {
        width: px(width),
        max_width: percent(100),
        flex_direction: FlexDirection::Column,
        row_gap: px(gap),
        ..default()
    }
}

fn station_heading(style: &UiStyle) -> impl Bundle + use<> {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(12),
            ..default()
        },
        children![
            (
                Node {
                    width: px(24),
                    height: px(2),
                    ..default()
                },
                BackgroundColor(AMBER)
            ),
            style.mono("ESTAÇÃO DE BOMBEAMENTO 07", 13.0, AMBER),
        ],
    )
}

fn home(style: &UiStyle) -> impl Bundle + use<> {
    (
        Region::Home,
        column(525.0, 28.0),
        children![
            (
                Region::Title,
                column(480.0, 13.0),
                children![
                    station_heading(style),
                    (
                        style.display("AINDA\nVIVO", 93.0, INK),
                        LineHeight::Px(76.0)
                    ),
                    style.body("Ecos do Desaparecimento", 22.0, MUTED),
                ],
            ),
            (
                Region::Record,
                column(480.0, 10.0),
                children![
                    station_heading(style),
                    (style.display("", 56.0, INK), Label::Outcome),
                    (style.body("", 18.0, MUTED), Label::Summary),
                ],
            ),
            (
                column(332.0, 6.0),
                children![
                    button(
                        Action::Main,
                        Tone::Main,
                        56.0,
                        children![
                            (style.body("", 21.0, INK), Label::Main),
                            (Region::MainShortcut, key_hint(style, "ENTER")),
                        ],
                    ),
                    row(style, Action::Settings, "Configurações", "›"),
                    row(style, Action::Controls, "Controles", "›"),
                    (
                        Region::Restart,
                        row(style, Action::Restart, "Reiniciar expedição", "")
                    ),
                    (
                        Region::WakeAction,
                        row(style, Action::Wake, "Despertar", "›")
                    ),
                    row(style, Action::Quit, "Sair", ""),
                ],
            ),
        ],
    )
}

fn button<B: Bundle>(action: Action, tone: Tone, height: f32, content: B) -> impl Bundle + use<B> {
    (
        Button,
        action,
        tone,
        Node {
            min_height: px(height),
            width: percent(100),
            border: UiRect::all(px(1)),
            padding: UiRect::axes(px(16), px(6)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(10),
            ..default()
        },
        BackgroundColor(PAGE_BACKGROUND),
        BorderColor::all(EDGE),
        content,
    )
}

fn row(style: &UiStyle, action: Action, title: &str, marker: &str) -> impl Bundle + use<> {
    button(
        action,
        Tone::Row,
        39.0,
        children![
            style.body(title, 17.0, INK),
            style.display(marker, 22.0, MUTED)
        ],
    )
}

fn settings(style: &UiStyle) -> impl Bundle + use<> {
    (
        Region::Settings,
        column(500.0, 16.0),
        children![
            page_heading(style, "Configurações"),
            (
                column(500.0, 0.0),
                children![
                    stepper(
                        style,
                        "Campo de visão vertical",
                        Label::Fov,
                        Action::FovDown,
                        Action::FovUp
                    ),
                    stepper(
                        style,
                        "Sensibilidade do mouse",
                        Label::Sensitivity,
                        Action::SensDown,
                        Action::SensUp
                    ),
                    toggle(style, "Balanço da câmera", Label::Bodycam, Action::Bodycam),
                    toggle(style, "Efeito VHS", Label::Vhs, Action::Vhs),
                    toggle(style, "Sincronização vertical", Label::Vsync, Action::Vsync),
                    stepper(
                        style,
                        "Volume do ambiente",
                        Label::Volume,
                        Action::VolumeDown,
                        Action::VolumeUp
                    ),
                    toggle(style, "Som do ambiente", Label::Mute, Action::Mute),
                ],
            ),
            back(style),
        ],
    )
}

fn page_heading(style: &UiStyle, name: &str) -> impl Bundle + use<> {
    (
        column(500.0, 9.0),
        children![
            (
                Node {
                    width: px(30),
                    height: px(2),
                    ..default()
                },
                BackgroundColor(AMBER)
            ),
            style.display(name, 47.0, INK),
        ],
    )
}

fn setting_line() -> Node {
    Node {
        width: percent(100),
        min_height: px(49),
        padding: UiRect::vertical(px(6)),
        border: UiRect::bottom(px(1)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(12),
        ..default()
    }
}

fn stepper(
    style: &UiStyle,
    title: &str,
    value: Label,
    lower: Action,
    raise: Action,
) -> impl Bundle + use<> {
    (
        setting_line(),
        BorderColor::all(EDGE),
        children![
            style.body(title, 17.0, INK),
            (
                Node {
                    column_gap: px(2),
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                children![
                    adjust(style, lower, "−"),
                    (
                        Node {
                            width: px(66),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        children![(style.mono("", 15.0, INK), value)],
                    ),
                    adjust(style, raise, "+"),
                ],
            ),
        ],
    )
}

fn adjust(style: &UiStyle, action: Action, icon: &str) -> impl Bundle + use<> {
    (
        Button,
        action,
        Tone::Control,
        Node {
            width: px(37),
            height: px(32),
            border: UiRect::all(px(1)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(PAGE_BACKGROUND),
        BorderColor::all(EDGE),
        children![style.body(icon, 19.0, INK)],
    )
}

fn toggle(style: &UiStyle, title: &str, value: Label, action: Action) -> impl Bundle + use<> {
    (
        setting_line(),
        BorderColor::all(EDGE),
        children![
            style.body(title, 17.0, INK),
            (
                Button,
                action,
                Tone::Control,
                Node {
                    width: px(144),
                    min_height: px(32),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    flex_shrink: 0.0,
                    border: UiRect::all(px(1)),
                    padding: UiRect::horizontal(px(10)),
                    ..default()
                },
                BackgroundColor(PAGE_BACKGROUND),
                BorderColor::all(EDGE),
                children![
                    (style.mono("", 14.0, INK), value),
                    style.display("›", 19.0, AMBER)
                ],
            ),
        ],
    )
}

fn back(style: &UiStyle) -> impl Bundle + use<> {
    (
        column(188.0, 0.0),
        children![button(
            Action::Back,
            Tone::Row,
            39.0,
            children![style.body("Voltar", 17.0, INK), key_hint(style, "ESC")],
        )],
    )
}

fn key_hint(style: &UiStyle, key: &str) -> impl Bundle + use<> {
    (
        Node {
            border: UiRect::all(px(1)),
            padding: UiRect::axes(px(7), px(4)),
            ..default()
        },
        BorderColor::all(Color::srgba(1.0, 0.91, 0.82, 0.40)),
        children![style.mono(key, 11.0, INK)],
    )
}

fn controls(style: &UiStyle, bindings: &Preferences) -> impl Bundle + use<> {
    let key = |name, fallback| key_name(bindings.key(name, fallback));
    let movement = [
        key("forward", KeyCode::KeyW),
        key("left", KeyCode::KeyA),
        key("back", KeyCode::KeyS),
        key("right", KeyCode::KeyD),
    ]
    .join("  ");
    let sprint = key("sprint", KeyCode::ShiftLeft);
    let sprint = if sprint == "Shift" {
        sprint
    } else {
        format!("{sprint} / Shift dir.")
    };
    let crouch = key("crouch", KeyCode::ControlLeft);
    let crouch = if crouch == "C" {
        crouch
    } else {
        format!("{crouch} / C")
    };
    (
        Region::Controls,
        column(500.0, 16.0),
        children![
            page_heading(style, "Controles"),
            (
                column(500.0, 0.0),
                children![
                    control_line(style, "Mover", &movement),
                    control_line(style, "Olhar", "Mouse"),
                    control_line(style, "Correr", &sprint),
                    control_line(style, "Agachar", &crouch),
                    control_line(style, "Pular", &key("jump", KeyCode::Space)),
                    control_line(style, "Interagir", &key("interact", KeyCode::KeyE)),
                    control_line(style, "Lanterna", &key("flashlight", KeyCode::KeyF)),
                    control_line(style, "Pausar", "Esc"),
                    control_line(style, "Diagnóstico", "F3"),
                    control_line(style, "Alternar VHS", "F4"),
                ],
            ),
            back(style),
        ],
    )
}

fn key_name(key: KeyCode) -> String {
    match key {
        KeyCode::Space => "Espaço".into(),
        KeyCode::ShiftLeft => "Shift".into(),
        KeyCode::ControlLeft => "Ctrl".into(),
        KeyCode::ArrowUp => "↑".into(),
        KeyCode::ArrowDown => "↓".into(),
        KeyCode::ArrowLeft => "←".into(),
        KeyCode::ArrowRight => "→".into(),
        _ => format!("{key:?}").trim_start_matches("Key").into(),
    }
}

fn control_line(style: &UiStyle, name: &str, key: &str) -> impl Bundle + use<> {
    (
        Node {
            min_height: px(33),
            padding: UiRect::vertical(px(4)),
            ..setting_line()
        },
        BorderColor::all(EDGE),
        children![style.body(name, 17.0, INK), style.mono(key, 15.0, INK)],
    )
}

fn confirmation(style: &UiStyle) -> impl Bundle + use<> {
    (
        Region::Confirmation,
        Node {
            padding: UiRect::all(px(26)),
            border: UiRect::all(px(1)),
            ..column(500.0, 16.0)
        },
        BackgroundColor(Color::srgb(0.049, 0.061, 0.051)),
        BorderColor::all(EDGE),
        children![
            page_heading(style, "Despertar"),
            style.body(
                "Interromper o registro e tentar voltar ao posto? A vistoria pode ser encerrada.",
                18.0,
                INK
            ),
            (
                Node {
                    column_gap: px(8),
                    ..default()
                },
                children![
                    button(
                        Action::CancelWake,
                        Tone::Row,
                        42.0,
                        children![style.body("Voltar", 17.0, INK)]
                    ),
                    button(
                        Action::ConfirmWake,
                        Tone::Warning,
                        42.0,
                        children![style.body("Confirmar", 17.0, INK)]
                    ),
                ],
            ),
        ],
    )
}

fn load_art(images: &mut Assets<Image>) -> Option<(Handle<Image>, Vec2)> {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/menu/estacao-07.jpg"
    ));
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes.as_slice()), options);
    let pixels = match decoder.decode() {
        Ok(pixels) => pixels,
        Err(error) => {
            warn!("Arte da tela inicial indisponível: {error}");
            return None;
        }
    };
    let info = decoder.info()?;
    let texture = Image::new(
        Extent3d {
            width: u32::from(info.width),
            height: u32::from(info.height),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    let size = texture.size_f32();
    Some((images.add(texture), size))
}
