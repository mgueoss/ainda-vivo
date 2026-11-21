//! Local presentation of instability. None of these surfaces emits a stimulus.
mod body_audio;
mod figure;

pub use body_audio::{BodySound, update as update_audio};
pub use figure::update as update_figure;

use super::Effects;
use crate::app::{Comfort, Flashlight, Screen, Session};
use bevy::{
    asset::RenderAssetUsages,
    ecs::system::SystemParam,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    ui::widget::NodeImageMode,
};

pub const FAREWELL_SECONDS: f32 = 8.5;

#[derive(Component)]
struct PeripheralShade;

#[derive(Component)]
struct FarewellFade;

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut voices: ResMut<Assets<crate::atmosphere::sound::Voice>>,
) {
    let image = images.add(shade());
    commands.spawn((
        Name::new("Sombra periférica"),
        PeripheralShade,
        screen_node(),
        ImageNode {
            image,
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        Visibility::Hidden,
        GlobalZIndex(-2),
    ));
    commands.spawn((
        Name::new("Escurecimento da gravação"),
        FarewellFade,
        screen_node(),
        BackgroundColor(Color::NONE),
        Visibility::Hidden,
        GlobalZIndex(-1),
    ));
    figure::setup(&mut commands, &mut meshes, &mut materials, &mut images);
    body_audio::setup(&mut commands, &mut voices);
}

fn screen_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

fn shade() -> Image {
    const SIZE: u32 = 64;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for col in 0..SIZE {
            let point = Vec2::new(col as f32, row as f32) / (SIZE - 1) as f32 * 2.0 - Vec2::ONE;
            let dark = smooth((point.length() - 0.57) / 0.68);
            pixels.extend_from_slice(&[2, 5, 4, (255.0 * dark).round() as u8]);
        }
    }
    rgba_image(SIZE, SIZE, pixels)
}

fn rgba_image(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn smooth(value: f32) -> f32 {
    let x = value.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[derive(SystemParam)]
#[allow(
    clippy::type_complexity,
    reason = "Disjoint Bevy camera, lamp and overlay queries."
)]
pub struct ViewEffects<'w, 's> {
    light: Query<'w, 's, (&'static mut SpotLight, &'static mut Transform), With<Flashlight>>,
    shaded: Query<
        'w,
        's,
        (&'static mut ImageNode, &'static mut Visibility),
        (With<PeripheralShade>, Without<FarewellFade>),
    >,
    fade:
        Query<'w, 's, (&'static mut BackgroundColor, &'static mut Visibility), With<FarewellFade>>,
}

pub fn update(
    session: Res<Session>,
    comfort: Res<Comfort>,
    effects: Res<Effects>,
    mut view: ViewEffects,
    mut lamp_intensity: Local<Option<f32>>,
) {
    let in_run = matches!(session.screen, Screen::Playing | Screen::Paused);
    let playing = session.screen == Screen::Playing;
    let farewell = in_run.then_some(effects.farewell).flatten();
    let severity = smooth((effects.intensity - 0.38) / 0.62);
    let av = comfort.psycho_av.clamp(0.0, 1.0);
    let edge = av * (0.22 * severity + 0.13 * smooth((effects.critical - 0.45) / 0.55));
    let immersion = playing && farewell.is_none();

    if let Ok((mut image, mut visibility)) = view.shaded.single_mut() {
        set_visible(&mut visibility, immersion && edge > 0.004);
        let tint = Color::srgb(1.0, 1.0, 1.0).with_alpha(edge);
        if image.color != tint {
            image.color = tint;
        }
    }
    if let Ok((mut color, mut visibility)) = view.fade.single_mut() {
        let alpha = farewell.map_or(0.0, |t| smooth((t - 6.15) / 2.0));
        set_visible(&mut visibility, alpha > 0.002);
        let shade = Color::BLACK.with_alpha(alpha);
        if color.0 != shade {
            color.0 = shade;
        }
    }
    if let Ok((mut light, mut pose)) = view.light.single_mut() {
        if let Some(age) = farewell {
            let normal = *lamp_intensity.get_or_insert(light.intensity);
            light.intensity = normal * (1.0 - smooth((age - 0.45) / 1.52));
        } else {
            if let Some(normal) = lamp_intensity.take() {
                light.intensity = normal;
            }
            if playing && comfort.psycho_motion && av > 0.0 && effects.episode > 0.0 {
                // Brief uneven grip, isolated from the camera and its aiming ray.
                let drift = 0.006 * av * effects.episode;
                let phase = session.elapsed * 10.0;
                pose.rotation *= Quat::from_rotation_y(phase.sin() * drift)
                    * Quat::from_rotation_x((phase * 0.67).cos() * drift * 0.66);
            }
        }
    }
}

fn set_visible(value: &mut Visibility, visible: bool) {
    let target = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *value != target {
        *value = target;
    }
}

/// Return ambient gain and low-pass strength for the existing stereo mixer.
pub fn ambience_mix(effects: &Effects, comfort: &Comfort) -> (f32, f32) {
    if let Some(age) = effects.farewell {
        let gone = smooth((age - 0.25) / 4.60);
        return (1.0 - gone, 0.48 * gone * comfort.psycho_av);
    }
    let pressure = smooth((effects.critical - 0.48) / 0.44);
    let stressed = smooth((effects.intensity - 0.40) / 0.60);
    (
        1.0 - comfort.psycho_av * (0.075 * stressed + 0.26 * pressure),
        comfort.psycho_av * (0.075 * stressed + 0.38 * pressure),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_respects_safe_levels_and_suppression() {
        let comfort = Comfort::default();
        let mut effects = Effects::default();
        assert_eq!(ambience_mix(&effects, &comfort), (1.0, 0.0));
        effects.intensity = 1.0;
        effects.critical = 1.0;
        let (gain, muffle) = ambience_mix(&effects, &comfort);
        assert!(gain >= 0.65);
        assert!(muffle <= 0.46);
        let reduced = Comfort {
            psycho_av: 0.0,
            ..comfort
        };
        assert_eq!(ambience_mix(&effects, &reduced), (1.0, 0.0));
        effects.farewell = Some(FAREWELL_SECONDS);
        assert_eq!(ambience_mix(&effects, &reduced), (0.0, 0.0));
    }
}
