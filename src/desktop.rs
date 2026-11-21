use crate::{app::*, smoke::SmokeTest};
use ainda_vivo_core::collision::Aabb;
use ainda_vivo_core::psychology::Motor;
use ainda_vivo_core::{Vec2, movement::SPAWN};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PresentMode, PrimaryWindow},
};

type FlashlightView<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut Visibility),
    (With<Flashlight>, Without<MainCamera>),
>;

#[allow(clippy::too_many_arguments, reason = "Bevy system parameters.")]
pub fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    bindings: Res<crate::preferences::Preferences>,
    windows: Query<&Window, With<PrimaryWindow>>,
    smoke: Res<SmokeTest>,
    mut session: ResMut<Session>,
    mut comfort: ResMut<Comfort>,
    mut player: ResMut<LocalPlayer>,
) {
    if smoke.enabled {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused {
        if session.screen == Screen::Playing {
            session.screen = Screen::Paused;
        }
        player.command = default();
        player.look_armed = false;
        return;
    }
    if keys.just_pressed(KeyCode::F3) {
        comfort.diagnostics = !comfort.diagnostics;
    }
    if keys.just_pressed(KeyCode::F4) {
        comfort.vhs = comfort.vhs.next();
    }
    if session.screen != Screen::Playing {
        player.command = default();
        player.look_armed = false;
        return;
    }
    let axis = |positive, negative| {
        i32::from(keys.pressed(positive)) as f32 - i32::from(keys.pressed(negative)) as f32
    };
    player.command.movement = Vec2::new(
        axis(
            bindings.key("right", KeyCode::KeyD),
            bindings.key("left", KeyCode::KeyA),
        ),
        axis(
            bindings.key("forward", KeyCode::KeyW),
            bindings.key("back", KeyCode::KeyS),
        ),
    );
    player.command.sprint = keys.pressed(bindings.key("sprint", KeyCode::ShiftLeft))
        || keys.pressed(KeyCode::ShiftRight);
    player.command.crouch =
        keys.pressed(bindings.key("crouch", KeyCode::ControlLeft)) || keys.pressed(KeyCode::KeyC);
    // Buffer the edge until a physics tick consumes it (also at high display FPS).
    player.command.jump |= keys.just_pressed(bindings.key("jump", KeyCode::Space));
    // Mouse deltas are already accumulated per frame: NEVER multiply by dt.
    if player.look_armed {
        player.yaw =
            (player.yaw - mouse.delta.x * comfort.sensitivity).rem_euclid(std::f32::consts::TAU);
        player.pitch = (player.pitch - mouse.delta.y * comfort.sensitivity).clamp(-1.45, 1.45);
    }
    player.look_armed = true;
    player.command.yaw = player.yaw;
    if keys.just_pressed(bindings.key("flashlight", KeyCode::KeyF)) {
        comfort.flashlight = !comfort.flashlight;
    }
}

pub fn simulate(
    time: Res<Time<Fixed>>,
    world: Res<CollisionWorld>,
    mut session: ResMut<Session>,
    mut player: ResMut<LocalPlayer>,
    mut solids: Local<Option<[Vec<Aabb>; 2]>>,
) {
    if world.is_changed() || solids.is_none() {
        *solids = Some([world.active(false).collect(), world.active(true).collect()]);
    }
    if session.screen != Screen::Playing {
        return;
    }
    let bounds = &solids.as_ref().expect("collision lists initialized")
        [usize::from(session.progress().exit_unlocked())];
    let dt = time.delta_secs();
    if let Some(age) = &mut session.farewell {
        *age += dt;
        let ended = *age >= crate::psychology::presentation::FAREWELL_SECONDS;
        let command = ainda_vivo_core::movement::PlayerCommand {
            crouch: true,
            ..default()
        };
        player.body.step(command, dt, bounds);
        if ended {
            session.screen = Screen::Complete;
        }
        return;
    }
    let mut command = player.command;
    if command.movement.length_squared() > 0.04 || command.jump {
        session.resting = false;
    }
    player.command.jump = false;
    let motor = session.psychology.motor();
    if motor != Motor::Steady && player.body.grounded {
        command.sprint = false;
        command.movement *= 1.0 - 0.23 * session.psychology.motor_strength();
        if motor == Motor::Kneeling {
            command.crouch = true;
            command.jump = false;
        }
    }
    player.body.step(command, dt, bounds);
    if player.body.feet.y < -8.0 || !player.body.feet.is_finite() {
        player.body = default();
        player.body.feet = SPAWN;
        session.say("Posição recuperada no início do mapa.");
    }
    session.elapsed += time.delta_secs();
    session.message_left = (session.message_left - time.delta_secs()).max(0.0);
    if crate::scene::station::reached_exit(player.body.feet) {
        session.cross_exit();
    }
    session.age_exit_receipt();
    if session.screen == Screen::Playing {
        let recovery = crate::psychology::recovery(&session, &player);
        session.psychology.step(dt, recovery);
        if session.psychology.expired() {
            session.farewell = Some(0.0);
            session.prompt.clear();
            session.false_exit = None;
        } else if session.psychology.critical_remaining().is_some() != session.critical_hint {
            session.critical_hint = !session.critical_hint;
            let message = match (session.critical_hint, session.psychology.has_reserve()) {
                (true, true) => {
                    "O fôlego está falhando. Procure a luz âmbar de um posto ou use a reserva [R]."
                }
                (true, false) => "O fôlego está falhando. Procure a luz âmbar de um posto.",
                (false, _) => "O fôlego começa a voltar.",
            };
            session.say(message);
        }
    }
}

pub fn update_camera(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    session: Res<Session>,
    comfort: Res<Comfort>,
    mut player: ResMut<LocalPlayer>,
    mut camera: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    mut light: FlashlightView,
) {
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    let rate = if session.collapse_active() { 2.6 } else { 14.0 };
    let blend = 1.0 - (-rate * time.delta_secs().min(0.1)).exp();
    let eye_target = if player.body.crouched { 0.93 } else { 1.62 };
    if session.screen == Screen::Playing && (!session.collapse_active() || comfort.psycho_motion) {
        player.eye_height += (eye_target - player.eye_height) * blend;
    }
    let alpha = if session.screen == Screen::Playing {
        fixed.overstep_fraction()
    } else {
        1.0
    };
    let feet = player.body.previous_feet.lerp(player.body.feet, alpha);
    let speed = player.body.velocity.with_y(0.0).length();
    let bob = if comfort.bodycam && player.body.grounded {
        (session.elapsed * 8.2).sin() * 0.005 * (speed / 2.5).min(1.0)
    } else {
        0.0
    };
    transform.translation = feet + Vec3::Y * (player.eye_height + bob);
    transform.rotation = Quat::from_rotation_y(player.yaw) * Quat::from_rotation_x(player.pitch);
    if let Projection::Perspective(perspective) = &mut *projection {
        perspective.fov = comfort.fov_degrees.to_radians();
    }
    if let Ok((mut light_transform, mut visibility)) = light.single_mut() {
        // Share the eye origin to eliminate parallax against nearby surfaces.
        light_transform.translation = transform.translation;
        light_transform.rotation = transform.rotation;
        *visibility = if comfort.flashlight {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn sync_cursor(
    session: Res<Session>,
    comfort: Res<Comfort>,
    smoke: Res<SmokeTest>,
    mut windows: Query<(&mut CursorOptions, &mut Window), With<PrimaryWindow>>,
) {
    let Ok((mut cursor, mut window)) = windows.single_mut() else {
        return;
    };
    let captured = session.screen == Screen::Playing && window.focused && !smoke.enabled;
    let wanted = if captured {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != wanted {
        cursor.grab_mode = wanted;
    }
    if cursor.visible == captured {
        cursor.visible = !captured;
    }
    let mode = if comfort.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    if window.present_mode != mode {
        window.present_mode = mode;
    }
}
