//! An optional frame from the station's own camera, rendered once per inspection.
use super::{DISTANCE_DARK, presence};
use crate::{
    app::{Screen, Session},
    interaction::Interactable,
    vhs::VhsImage,
};
use ainda_vivo_core::{
    collision::Aabb,
    memory::ServiceTrail,
    story::{Action, Phase},
};
use bevy::{
    camera::{Exposure, RenderTarget, visibility::RenderLayers},
    core_pipeline::tonemapping::Tonemapping,
    light::NotShadowCaster,
    prelude::*,
    render::render_resource::TextureFormat,
};

pub const TERMINAL: Vec3 = Vec3::new(17.22, 1.184, -82.105);
const CAMERA: Vec3 = Vec3::new(0.42, 2.48, -43.36);
const WITNESS: Vec3 = Vec3::new(4.73, 0.016, -43.58);
const LAYER: usize = 3;
const CAPTURE_WINDOW: f32 = 0.55;
const SCREEN_OFF: Color = Color::srgb(0.048, 0.092, 0.088);

#[derive(Component)]
pub struct ArchiveCamera;

#[derive(Component)]
pub enum ArchiveMark {
    Figure,
    WorkOrder,
}

#[derive(Resource)]
pub struct Recording {
    attempt: u64,
    started_at: Option<f32>,
    ready: bool,
    displayed: bool,
    showing_echo: bool,
    echo_started_at: Option<f32>,
    frame: Handle<Image>,
    echo_frame: Handle<Image>,
    glass: Handle<StandardMaterial>,
}

impl Recording {
    pub fn displayed(&self) -> bool {
        self.displayed
    }

    pub fn showing_echo(&self) -> bool {
        self.showing_echo
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut target = || {
        images.add(Image::new_target_texture(
            384,
            216,
            TextureFormat::Rgba8Unorm,
            Some(TextureFormat::Rgba8UnormSrgb),
        ))
    };
    let frame = target();
    let echo_frame = target();
    let glass = materials.add(StandardMaterial {
        base_color: SCREEN_OFF,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Name::new("Imagem arquivada no monitor da sala de Controle"),
        Mesh3d(meshes.add(Rectangle::new(0.344, 0.1935))),
        MeshMaterial3d(glass.clone()),
        Transform::from_translation(TERMINAL),
        Interactable {
            bounds: Aabb::from_center_size(TERMINAL, Vec3::new(0.45, 0.285, 0.175)),
            action: Action::ReviewRecording,
        },
        NotShadowCaster,
    ));
    commands.spawn((
        Name::new("Câmera de segurança da nave"),
        ArchiveCamera,
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: DISTANCE_DARK.into(),
            is_active: false,
            ..default()
        },
        RenderTarget::Image(frame.clone().into()),
        RenderLayers::layer(0).with(LAYER),
        Projection::Perspective(PerspectiveProjection {
            fov: 52.0_f32.to_radians(),
            near: 0.08,
            far: 24.0,
            ..default()
        }),
        AmbientLight {
            color: Color::srgb(0.63, 0.86, 0.79),
            brightness: 238.0,
            ..default()
        },
        Exposure { ev100: 8.25 },
        Tonemapping::TonyMcMapface,
        Msaa::Sample4,
        DistanceFog {
            color: DISTANCE_DARK,
            falloff: FogFalloff::Linear {
                start: 5.0,
                end: 17.6,
            },
            ..default()
        },
        VhsImage::archive(),
        Transform::from_translation(CAMERA).looking_at(Vec3::new(6.1, 1.27, -43.23), Vec3::Y),
    ));

    let mask = images.add(presence::mask());
    let bearing = (CAMERA - WITNESS).with_y(0.0);
    commands.spawn((
        Name::new("Figura presente no quadro arquivado"),
        ArchiveMark::Figure,
        RenderLayers::layer(LAYER),
        Mesh3d(meshes.add(Rectangle::new(presence::WIDTH, presence::HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.90),
            base_color_texture: Some(mask),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(WITNESS + Vec3::Y * (presence::HEIGHT * 0.5))
            .looking_to(-bearing, Vec3::Y),
        NotShadowCaster,
    ));
    let print = images.add(super::surfaces::sign(
        "OS 17   03:17",
        [105, 75, 39],
        [230, 202, 151],
    ));
    commands.spawn((
        Name::new("Ficha visível apenas na segunda gravação"),
        ArchiveMark::WorkOrder,
        RenderLayers::layer(LAYER),
        Visibility::Hidden,
        Mesh3d(meshes.add(Rectangle::new(0.66, 0.18))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(print),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(3.91, 0.87, -43.11)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        NotShadowCaster,
    ));
    commands.insert_resource(Recording {
        attempt: 0,
        started_at: None,
        ready: false,
        displayed: false,
        showing_echo: false,
        echo_started_at: None,
        frame,
        echo_frame,
        glass,
    });
}

pub fn update(
    session: Res<Session>,
    mut recording: ResMut<Recording>,
    mut camera: Query<(&mut Camera, &mut RenderTarget), With<ArchiveCamera>>,
    mut marks: Query<(&ArchiveMark, &mut Visibility)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let restarted = recording.attempt != session.attempt;
    if restarted {
        recording.attempt = session.attempt;
        recording.started_at = None;
        recording.ready = false;
        recording.echo_started_at = None;
    }
    let playing = session.screen == Screen::Playing;
    if playing && session.progress().phase() >= Phase::Electrical && recording.started_at.is_none()
    {
        recording.started_at = Some(session.elapsed);
    }
    if let Some(start) = recording.started_at {
        recording.ready = session.elapsed - start >= CAPTURE_WINDOW;
    }
    let start_echo = playing
        && recording.ready
        && recording.echo_started_at.is_none()
        && session.memory().service == ServiceTrail::Reconciled;
    if start_echo {
        recording.echo_started_at = Some(session.elapsed);
    }
    let echo_ready = recording
        .echo_started_at
        .is_some_and(|at| session.elapsed - at >= CAPTURE_WINDOW);
    if let Ok((mut camera, mut target)) = camera.single_mut() {
        if restarted {
            *target = RenderTarget::Image(recording.frame.clone().into());
        }
        if start_echo {
            *target = RenderTarget::Image(recording.echo_frame.clone().into());
        }
        let capturing = playing
            && (recording.started_at.is_some() && !recording.ready
                || recording.echo_started_at.is_some() && !echo_ready);
        if camera.is_active != capturing {
            camera.is_active = capturing;
        }
    }
    for (mark, mut visibility) in &mut marks {
        let echo = recording.echo_started_at.is_some();
        let desired = match mark {
            ArchiveMark::Figure if !echo => Visibility::Visible,
            ArchiveMark::WorkOrder if echo && !echo_ready => Visibility::Visible,
            _ => Visibility::Hidden,
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
    let reviewing = recording.ready && session.progress().recording_reviewed();
    let showing_echo = reviewing && echo_ready;
    if (reviewing != recording.displayed || showing_echo != recording.showing_echo)
        && let Some(glass) = materials.get_mut(&recording.glass)
    {
        glass.base_color_texture = reviewing.then(|| {
            if showing_echo {
                recording.echo_frame.clone()
            } else {
                recording.frame.clone()
            }
        });
        glass.base_color = if reviewing { Color::WHITE } else { SCREEN_OFF };
        recording.displayed = reviewing;
        recording.showing_echo = showing_echo;
    }
}
