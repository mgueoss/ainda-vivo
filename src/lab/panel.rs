use super::{
    Action, Knob, Lab, Options, ctrl,
    library::{self, Catalog},
    settings::{Category, Config, Lighting, MAX_INSTANCES, Model, Preset, Tab},
};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::{ButtonState, keyboard::KeyboardInput, mouse::AccumulatedMouseScroll},
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
};

const PANEL: Color = Color::srgb(0.052, 0.073, 0.077);
const HEADER: Color = Color::srgb(0.082, 0.105, 0.105);
const EDGE: Color = Color::srgb(0.18, 0.24, 0.235);
const BASE: Color = Color::srgb(0.103, 0.143, 0.146);
const HOVER: Color = Color::srgb(0.156, 0.218, 0.212);
const ACTIVE: Color = Color::srgb(0.285, 0.217, 0.110);
const INK: Color = Color::srgb(0.88, 0.90, 0.85);
const MUTED: Color = Color::srgb(0.61, 0.69, 0.66);
const ACCENT: Color = Color::srgb(0.99, 0.76, 0.40);

#[derive(Component)]
pub(super) struct Interface;
#[derive(Component)]
pub(super) struct ScrollArea;
#[derive(Component)]
pub(super) struct UsesClipList;
#[derive(Component)]
pub(super) struct Trigger(Action);
#[derive(Component)]
pub(super) enum Showing {
    Tab(Tab),
    Model(Model),
    Instance(usize),
}

#[derive(Component)]
pub(super) enum Label {
    Status,
    View,
    Model(Model),
    Search,
    Category,
    Instance(usize),
    Button(Action),
    Measure(Knob),
    Timeline,
    Technical,
}

pub(super) fn spawn(commands: &mut Commands, camera: Entity, config: &Config) -> Entity {
    commands
        .spawn((
            Interface,
            UiTargetCamera(camera),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                display: if config.hidden {
                    Display::None
                } else {
                    Display::Flex
                },
                flex_direction: FlexDirection::Column,
                ..default()
            },
            FocusPolicy::Pass,
        ))
        .with_children(|ui| {
            ui.spawn((
                Node {
                    width: percent(100),
                    height: px(42),
                    flex_shrink: 0.0,
                    padding: UiRect::axes(px(15), px(3)),
                    border: UiRect::bottom(px(1)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(12),
                    ..default()
                },
                BackgroundColor(HEADER),
                BorderColor::all(EDGE),
            ))
            .with_children(|bar| {
                bar.spawn(text("AINDA VIVO  /  DEV LAB", 17.0, ACCENT));
                bar.spawn((text("", 12.0, MUTED), Label::View));
                bar.spawn(text(
                    format!("v{}  ·  Ctrl+I volta", crate::app::VERSION),
                    12.0,
                    MUTED,
                ));
            });
            ui.spawn((
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_height: px(0),
                    ..default()
                },
                FocusPolicy::Pass,
            ))
            .with_children(|body| {
                sidebar(body, true).with_children(library_panel);
                body.spawn((
                    Button,
                    Trigger(Action::CaptureCursor),
                    BackgroundColor(Color::NONE),
                    Node {
                        flex_grow: 1.0,
                        min_width: px(0),
                        align_items: AlignItems::End,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::bottom(px(18)),
                        ..default()
                    },
                ))
                .with_children(|center| {
                    center.spawn((
                        text("Clique na cena ou use Tab", 12.0, INK),
                        Label::Button(Action::CaptureCursor),
                    ));
                });
                sidebar(body, false).with_children(inspector);
            });
            ui.spawn((
                Node {
                    width: percent(100),
                    min_height: px(29),
                    flex_shrink: 0.0,
                    padding: UiRect::axes(px(12), px(2)),
                    align_items: AlignItems::Center,
                    border: UiRect::top(px(1)),
                    ..default()
                },
                BackgroundColor(HEADER),
                BorderColor::all(EDGE),
            ))
            .with_children(|bar| {
                bar.spawn((text("", 12.0, MUTED), Label::Status));
            });
        })
        .id()
}

fn sidebar<'a>(parent: &'a mut ChildSpawnerCommands, left: bool) -> EntityCommands<'a> {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: percent(if left { 31 } else { 39 }),
            max_width: px(if left { 233 } else { 307 }),
            flex_shrink: 0.0,
            padding: UiRect::all(px(11)),
            row_gap: px(7),
            border: if left {
                UiRect::right(px(1))
            } else {
                UiRect::left(px(1))
            },
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(EDGE),
        FocusPolicy::Block,
    ))
}

fn library_panel(side: &mut ChildSpawnerCommands) {
    side.spawn(text("Biblioteca", 21.0, INK));
    row(side).with_children(|line| {
        button(line, "Pesquisar", Action::Search);
        button(line, "Categoria", Action::Category);
    });
    side.spawn((text("", 12.0, MUTED), Label::Category));
    side.spawn((text("", 13.0, ACCENT), Label::Search));
    section(side, "ADICIONAR À CENA");
    for model in Model::ALL {
        button(side, model.name(), Action::Add(model)).insert(Showing::Model(model));
    }
    row(side).with_children(|line| {
        button(line, "Comparar", Action::Compare);
        button(line, "Recarregar", Action::Reload);
    });
    section(side, "INSTÂNCIAS");
    scroll(side).with_children(|list| {
        for index in 0..MAX_INSTANCES {
            button(list, "", Action::Select(index)).insert(Showing::Instance(index));
        }
    });
    row(side).with_children(|line| {
        button(line, "Duplicar", Action::Duplicate);
        button(line, "Remover", Action::Remove);
    });
}

fn inspector(side: &mut ChildSpawnerCommands) {
    side.spawn(text("Inspeção", 21.0, INK));
    row(side).with_children(|line| {
        button(line, "Objeto", Action::SetTab(Tab::Object));
        button(line, "Luz", Action::SetTab(Tab::Lighting));
        button(line, "Poses", Action::SetTab(Tab::Animation));
        button(line, "Ajuda", Action::SetTab(Tab::Help));
    });
    scroll(side).with_children(|list| {
        group(list, Tab::Object).with_children(|view| {
            section(view, "TRANSFORMAÇÕES");
            for (axis, tag) in ["X", "Y", "Z"].into_iter().enumerate() {
                adjust(view, &format!("Posição {tag}"), Knob::Translation(axis), 0.10);
            }
            for (axis, tag) in ["X", "Y", "Z"].into_iter().enumerate() {
                adjust(view, &format!("Rotação {tag}"), Knob::Angles(axis), 15.0);
            }
            adjust(view, "Escala", Knob::Scale, 0.10);
            row(view).with_children(|line| {
                button(line, "Centralizar", Action::CenterModel);
                button(line, "Restaurar", Action::ResetModel);
                button(line, "Visível", Action::Visible);
            });
            row(view).with_children(|line| {
                button(line, "Sombra", Action::ModelShadow);
                button(line, "Colisor", Action::ModelBounds);
            });
            button(view, "Material", Action::Material);
            section(view, "CÂMERA");
            row(view).with_children(|line| {
                button(line, "Livre / 1ª pessoa", Action::FreeCamera);
                button(line, "Enquadrar", Action::Framing);
            });
            row(view).with_children(|line| {
                button(line, "Órbita", Action::Orbit);
                button(line, "Rosto", Action::Face);
                button(line, "Mãos", Action::Hands);
            });
            adjust(view, "Campo de visão", Knob::Fov, 5.0);
            adjust(view, "Velocidade livre", Knob::Speed, 0.5);
            section(view, "DADOS DO RECURSO");
            view.spawn((text("", 12.0, MUTED), Label::Technical));
        });
        group(list, Tab::Lighting).with_children(|view| {
            section(view, "PREDEFINIÇÕES");
            for preset in Preset::ALL {
                button(view, preset.label(), Action::Preset(preset));
            }
            section(view, "AJUSTES");
            adjust(view, "Luz principal", Knob::Key, 200.0);
            adjust(view, "Ambiente", Knob::Fill, 12.5);
            adjust(view, "Azimute", Knob::Azimuth, 15.0);
            adjust(view, "Elevação", Knob::Elevation, 7.5);
            adjust(view, "Exposição", Knob::Exposure, 0.5);
            button(view, "Cor", Action::Color);
            row(view).with_children(|line| {
                button(line, "Lanterna", Action::Torch);
                button(line, "Sombras", Action::Shadows);
                button(line, "Névoa", Action::Fog);
            });
            row(view).with_children(|line| {
                button(line, "VHS", Action::Tape);
                button(line, "Grade", Action::Grid);
            });
        });
        group(list, Tab::Animation).with_children(|view| {
            section(view, "POSES E CLIPES DO GLB");
            view.spawn((column(3), UsesClipList));
            row(view).with_children(|line| {
                button(line, "Ossos", Action::Skeleton);
                button(line, "Wireframe", Action::Wireframe);
            });
            section(view, "REPRODUÇÃO");
            row(view).with_children(|line| {
                button(line, "Reproduzir", Action::Play);
                button(line, "Reiniciar", Action::Rewind);
                button(line, "Loop", Action::Repeat);
            });
            adjust(view, "Velocidade", Knob::Playback, 0.25);
            view.spawn((text("", 13.0, ACCENT), Label::Timeline));
            row(view).with_children(|line| {
                button(line, "−0,1 s", Action::Seek(-0.10));
                for frame in [0., 0.25, 0.5, 0.75, 1.] {
                    button(line, &format!("{:.0}", frame * 100.0), Action::SeekFraction(frame));
                }
                button(line, "+0,1 s", Action::Seek(0.10));
            });
        });
        group(list, Tab::Help).with_children(|view| {
            section(view, "ATALHOS");
            view.spawn(text(
                "Ctrl+I  Entrar ou retornar\nCtrl+H  Mostrar interface\nCtrl+G  Alternar grade\nCtrl+L  Próxima luz\nCtrl+B  Esqueleto\nCtrl+W  Wireframe\nCtrl+P  Pausar ou reproduzir\nCtrl+R  Restaurar cena\nCtrl+F  Câmera livre\nCtrl+Shift+S  Captura PNG",
                13.0, INK,
            ));
            section(view, "MOVIMENTO");
            view.spawn(text(
                "Clique na cena ou use Tab para mover. Esc ou Tab libera o cursor. Use WASD e mouse. Espaço sobe na câmera livre e Ctrl desce. Shift acelera. Na órbita, use o scroll para aproximar. F liga a lanterna.",
                13.0, MUTED,
            ));
            section(view, "PREFERÊNCIAS LOCAIS");
            button(view, "Salvar laboratório", Action::Save);
            button(view, "Restaurar padrões", Action::Reset);
            button(view, "Capturar cena", Action::Screenshot);
        });
    });
    row(side).with_children(|line| {
        button(line, "Salvar cena", Action::Save);
        button(line, "Ocultar painel", Action::HideUi);
    });
}

fn text(content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

fn column(gap: i32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        width: percent(100),
        row_gap: px(gap),
        flex_shrink: 0.0,
        ..default()
    }
}

fn section(parent: &mut ChildSpawnerCommands, name: &str) {
    parent.spawn((
        text(name, 11.0, MUTED),
        Node {
            margin: UiRect::top(px(4)),
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

fn group<'a>(parent: &'a mut ChildSpawnerCommands, tab: Tab) -> EntityCommands<'a> {
    parent.spawn((column(6), Showing::Tab(tab)))
}

fn row<'a>(parent: &'a mut ChildSpawnerCommands) -> EntityCommands<'a> {
    parent.spawn(Node {
        column_gap: px(4),
        align_items: AlignItems::Center,
        width: percent(100),
        flex_shrink: 0.0,
        ..default()
    })
}

fn scroll<'a>(parent: &'a mut ChildSpawnerCommands) -> EntityCommands<'a> {
    parent.spawn((
        ScrollArea,
        ScrollPosition::default(),
        RelativeCursorPosition::default(),
        Node {
            flex_grow: 1.0,
            min_height: px(0),
            overflow: Overflow::scroll_y(),
            ..column(4)
        },
    ))
}

fn button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    label: &str,
    action: Action,
) -> EntityCommands<'a> {
    let mut button = parent.spawn((
        Button,
        Trigger(action),
        Node {
            min_width: px(0),
            min_height: px(25),
            flex_grow: 1.0,
            flex_shrink: 0.0,
            border: UiRect::all(px(1)),
            padding: UiRect::axes(px(5), px(2)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BASE),
        BorderColor::all(EDGE),
    ));
    button.with_children(|contents| {
        contents.spawn((
            text(label, 12.0, INK),
            match action {
                Action::Select(index) => Label::Instance(index),
                Action::Add(model) => Label::Model(model),
                _ => Label::Button(action),
            },
        ));
    });
    button
}

fn adjust(parent: &mut ChildSpawnerCommands, name: &str, knob: Knob, delta: f32) {
    row(parent).with_children(|line| {
        line.spawn((
            text(name, 12.0, MUTED),
            Node {
                flex_grow: 1.0,
                ..default()
            },
        ));
        button(line, "−", Action::Adjust(knob, -delta)).insert(Node {
            width: px(29),
            height: px(23),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        });
        line.spawn((
            text("", 12.0, INK),
            Label::Measure(knob),
            Node {
                min_width: px(52),
                ..default()
            },
        ));
        button(line, "+", Action::Adjust(knob, delta)).insert(Node {
            width: px(29),
            height: px(23),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        });
    });
}

#[allow(
    clippy::too_many_arguments,
    reason = "Independent Bevy input channels and UI queries."
)]
pub(super) fn input(
    mut lab: ResMut<Lab>,
    mut config: ResMut<Config>,
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard: MessageReader<KeyboardInput>,
    clicks: Query<(&Interaction, &Trigger), Changed<Interaction>>,
    wheel: Res<AccumulatedMouseScroll>,
    mut scrollers: Query<(&RelativeCursorPosition, &mut ScrollPosition), With<ScrollArea>>,
    mut queue: MessageWriter<Action>,
    options: Res<Options>,
) {
    if options.smoke {
        return;
    }
    let can_click = !lab.active.as_ref().is_some_and(|current| current.captured);
    if can_click && wheel.delta.y != 0.0 && !config.hidden {
        for (cursor, mut position) in &mut scrollers {
            if cursor.cursor_over() {
                position.y = (position.y - wheel.delta.y * 31.0).max(0.0);
            }
        }
    }
    if can_click {
        for (hit, command) in &clicks {
            if *hit == Interaction::Pressed {
                lab.search_focus = command.0 == Action::Search;
                queue.write(command.0);
            }
        }
    }
    for event in keyboard.read() {
        if !lab.search_focus || event.state != ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Escape | KeyCode::Enter | KeyCode::NumpadEnter => {
                lab.search_focus = false;
            }
            KeyCode::Backspace => {
                config.search.pop();
            }
            KeyCode::KeyA if ctrl(&keys) => config.search.clear(),
            _ if !ctrl(&keys) => {
                if let Some(typed) = &event.text {
                    let room = 48usize.saturating_sub(config.search.chars().count());
                    config
                        .search
                        .extend(typed.chars().filter(|c| !c.is_control()).take(room));
                }
            }
            _ => {}
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Independent Bevy UI and asset queries."
)]
pub(super) fn refresh(
    mut commands: Commands,
    lab: Res<Lab>,
    catalog: Res<Catalog>,
    options: Res<Options>,
    config: Res<Config>,
    metrics: Res<DiagnosticsStore>,
    mut containers: Query<&mut Node, With<Interface>>,
    mut visibility: Query<(&Showing, &mut Node), Without<Interface>>,
    mut buttons: Query<(&Interaction, &Trigger, &mut BackgroundColor)>,
    mut labels: Query<(&Label, &mut Text)>,
    clips: Query<(Entity, Option<&Children>), With<UsesClipList>>,
    animation: (
        Query<(&AnimationPlayer, &library::Player)>,
        Query<&library::Viewer>,
    ),
    mut previous_count: Local<Option<(Entity, u32, bool)>>,
) {
    let Some(active) = &lab.active else { return };
    for mut node in &mut containers {
        let display = if config.hidden {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (show, mut node) in &mut visibility {
        let is_visible = match show {
            Showing::Tab(tab) => config.tab == *tab,
            Showing::Model(model) => model.matches(config.category, &config.search),
            Showing::Instance(index) => *index < config.instances.len(),
        };
        let display = if is_visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let selected = config.selected();
    if let Ok((id, children)) = clips.single() {
        let want_clips = selected.is_some_and(|model| model.model == Model::Presence);
        let key = (id, catalog.revision(), want_clips);
        if *previous_count != Some(key) {
            *previous_count = Some(key);
            if let Some(children) = children {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
            commands.entity(id).with_children(|parent| {
                if want_clips {
                    for (i, clip) in catalog.clips().iter().enumerate() {
                        button(parent, &clip.label, Action::Clip(i));
                    }
                }
                if !want_clips || catalog.clips().is_empty() {
                    parent.spawn(text("Sem clipes disponíveis para a seleção.", 12.0, MUTED));
                }
            });
        }
    }
    for (interaction, Trigger(action), mut background) in &mut buttons {
        if *action == Action::CaptureCursor {
            continue;
        }
        let toggled = match *action {
            Action::SetTab(tab) => config.tab == tab,
            Action::Select(index) => config.selected == index,
            Action::Preset(which) => config.lighting.preset == which,
            Action::Clip(i) => selected.is_some_and(|spec| spec.clip == i),
            Action::Search => lab.search_focus,
            Action::Grid => config.grid,
            Action::Skeleton => config.skeleton,
            Action::Wireframe => config.wireframe,
            Action::Play => active.playback.running,
            Action::Repeat => active.playback.repeat,
            Action::Fog => config.lighting.fog,
            Action::Shadows => config.lighting.shadow,
            Action::Torch => config.lighting.torch,
            Action::Orbit => active.orbit.is_some(),
            _ => false,
        };
        let next = if toggled || *interaction == Interaction::Pressed {
            ACTIVE
        } else if *interaction == Interaction::Hovered {
            HOVER
        } else {
            BASE
        };
        if background.0 != next {
            background.0 = next;
        }
    }
    for (label, mut value) in &mut labels {
        let edited = match label {
            Label::Status => catalog.error.as_ref().unwrap_or(&lab.status).clone(),
            Label::View => format!(
                "{}  ·  {}",
                if config.lens.free {
                    "Câmera livre"
                } else {
                    "Primeira pessoa"
                },
                config.lighting.preset.label()
            ),
            Label::Search => {
                if lab.search_focus {
                    format!("{}▏", config.search)
                } else if config.search.is_empty() {
                    "Todos os nomes".into()
                } else {
                    config.search.clone()
                }
            }
            Label::Category => format!("Categoria · {}", Category::ALL[config.category as usize].1),
            Label::Model(model) => model.name().into(),
            Label::Instance(i) => config.instances.get(*i).map_or(String::new(), |s| {
                format!("{:02}  {}", i + 1, s.model.name())
            }),
            Label::Measure(knob) => measure(*knob, &config, active.playback.speed),
            Label::Technical => {
                let sample = |id: &bevy::diagnostic::DiagnosticPath| {
                    metrics
                        .get(id)
                        .and_then(|values| values.smoothed())
                        .map_or(String::from("—"), |v| format!("{v:.1}"))
                };
                let fps = sample(&FrameTimeDiagnosticsPlugin::FPS);
                let ms = sample(&FrameTimeDiagnosticsPlugin::FRAME_TIME);
                let header = format!(
                    "{fps} FPS  ·  {ms} ms  ·  {} instâncias",
                    config.instances.len()
                );
                if let Some(spec) = selected
                    && let Some(meta) = catalog.stats(spec.model)
                {
                    let source = match spec.model {
                        Model::Presence => options.path(),
                        Model::Legacy => "src/scene/presence.rs",
                        Model::Human => "Referência cinemática",
                        _ => "src/scene/station/machines.rs",
                    };
                    let dimensions = meta.meters.map_or("—".into(), |extent| {
                        let d = extent * spec.scale;
                        format!("{:.2} × {:.2} × {:.2} m", d.x, d.y, d.z)
                    });
                    let clips = if spec.model == Model::Presence {
                        catalog.clips().len()
                    } else {
                        0
                    };
                    format!(
                        "{}\n{source}\n{dimensions}  (X × Y × Z)\n{} triângulos  ·  {} vértices\n{} materiais  ·  {} texturas\n{} ossos  ·  {clips} clipes  ·  {} colisores\n{header}",
                        spec.model.name(),
                        meta.triangles,
                        meta.vertices,
                        meta.materials,
                        meta.textures,
                        meta.bones,
                        meta.colliders
                    )
                } else {
                    header
                }
            }
            Label::Timeline => {
                library::timeline(config.selected, &catalog, &animation.0, &animation.1)
                    .map_or_else(
                        || "Linha do tempo indisponível".into(),
                        |(time, total)| format!("{time:.2} / {total:.2} s"),
                    )
            }
            Label::Button(action) => match action {
                Action::CaptureCursor => if active.captured {
                    "Tab ou Esc para liberar"
                } else {
                    "Clique na cena ou use Tab"
                }
                .into(),
                Action::Material => {
                    format!("Material · {}", selected.map_or("—", |s| s.surface.label()))
                }
                Action::Color => format!("Cor · {}", Lighting::COLORS[config.lighting.color].0),
                Action::Tape => format!(
                    "VHS · {}",
                    ["Off", "Suave", "Forte"][config.lighting.vhs as usize]
                ),
                Action::Visible => if selected.is_some_and(|s| s.visible) {
                    "Visível"
                } else {
                    "Oculto"
                }
                .into(),
                Action::ModelShadow => if selected.is_some_and(|s| s.shadow) {
                    "Sombra sim"
                } else {
                    "Sombra não"
                }
                .into(),
                Action::ModelBounds => if selected.is_some_and(|s| s.bounds) {
                    "Colisor sim"
                } else {
                    "Colisor não"
                }
                .into(),
                Action::Play => if active.playback.running {
                    "Pausar"
                } else {
                    "Reproduzir"
                }
                .into(),
                _ => continue,
            },
        };
        if **value != edited {
            **value = edited;
        }
    }
}

fn measure(knob: Knob, config: &Config, playback: f32) -> String {
    let selected = config.selected();
    match knob {
        Knob::Translation(axis) => {
            selected.map_or("—".into(), |spec| format!("{:.2} m", spec.position[axis]))
        }
        Knob::Angles(axis) => {
            selected.map_or("—".into(), |spec| format!("{:+.0}°", spec.angles[axis]))
        }
        Knob::Scale => selected.map_or("—".into(), |spec| format!("{:.2}×", spec.scale)),
        Knob::Key => format!("{:.0} lux", config.lighting.key),
        Knob::Fill => format!("{:.0}", config.lighting.fill),
        Knob::Azimuth => format!("{:.0}°", config.lighting.azimuth),
        Knob::Elevation => format!("{:.0}°", config.lighting.elevation),
        Knob::Exposure => format!("{:.1} EV", config.lighting.exposure),
        Knob::Fov => format!("{:.0}°", config.lens.fov),
        Knob::Speed => format!("{:.1} m/s", config.lens.speed),
        Knob::Playback => format!("{playback:.2}×"),
    }
}
