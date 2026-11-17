use crate::app::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

#[derive(Resource, Default)]
pub struct Performance {
    pub text: String,
    since: f32,
}

pub fn update(
    time: Res<Time>,
    store: Res<DiagnosticsStore>,
    player: Res<LocalPlayer>,
    mut performance: ResMut<Performance>,
) {
    performance.since += time.delta_secs();
    if performance.since < 0.25 {
        return;
    }
    performance.since = 0.0;
    let fps = store
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let ms = store
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let p = player.body.feet;
    performance.text = format!(
        "{fps:.0} FPS  |  {ms:.1} ms\nFísica 60 Hz  |  contatos {}\n{:.1}, {:.2}, {:.1}  |  {}\nF3 ocultar",
        player.body.contacts,
        p.x,
        p.y,
        p.z,
        if player.body.crouched {
            "AGACHADO"
        } else if player.body.grounded {
            "NO CHÃO"
        } else {
            "NO AR"
        }
    );
}
