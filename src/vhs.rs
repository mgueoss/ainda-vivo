use crate::app::{Comfort, MainCamera, Screen, Session};
use bevy::{
    asset::embedded_asset,
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    },
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_graph::{InternedRenderLabel, InternedRenderSubGraph, RenderLabel, RenderSubGraph},
        render_resource::ShaderType,
    },
    shader::ShaderRef,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VhsMode {
    Off,
    #[default]
    Soft,
    Strong,
}

impl VhsMode {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Soft,
            Self::Soft => Self::Strong,
            Self::Strong => Self::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Desligado",
            Self::Soft => "Suave",
            Self::Strong => "Forte",
        }
    }

    pub fn setting(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Soft => "soft",
            Self::Strong => "strong",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "soft" => Some(Self::Soft),
            "strong" => Some(Self::Strong),
            _ => None,
        }
    }

    fn strength(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Soft => 0.45,
            Self::Strong => 1.0,
        }
    }
}

pub struct VhsPlugin;

impl Plugin for VhsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "vhs.wgsl");
        app.add_plugins(FullscreenMaterialPlugin::<VhsImage>::default());
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType, Default)]
pub struct VhsImage {
    parameters: Vec4,
}

impl VhsImage {
    pub fn archive() -> Self {
        Self {
            parameters: Vec4::new(0.54, 37.0, 0.0, 0.0),
        }
    }
}

impl FullscreenMaterial for VhsImage {
    fn fragment_shader() -> ShaderRef {
        "embedded://ainda_vivo/vhs.wgsl".into()
    }

    fn sub_graph() -> Option<InternedRenderSubGraph> {
        Some(Core3d.intern())
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node3d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node3d::EndMainPassPostProcessing.intern(),
        ]
    }
}

pub fn update(
    mut commands: Commands,
    comfort: Res<Comfort>,
    session: Res<Session>,
    mut camera: Query<(Entity, Option<&mut VhsImage>), With<MainCamera>>,
) {
    let Ok((entity, settings)) = camera.single_mut() else {
        return;
    };
    if comfort.vhs == VhsMode::Off || session.screen == Screen::Menu {
        if settings.is_some() {
            commands.entity(entity).remove::<VhsImage>();
        }
        return;
    }
    let parameters = Vec4::new(comfort.vhs.strength(), session.elapsed, 0.0, 0.0);
    if let Some(mut current) = settings {
        if current.parameters != parameters {
            current.parameters = parameters;
        }
    } else {
        commands.entity(entity).insert(VhsImage { parameters });
    }
}
