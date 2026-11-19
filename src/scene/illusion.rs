//! Observer-local dressing. These impressions never own a collider or a mission interaction.
use crate::{
    app::{CollisionWorld, ExitPresentation, MainCamera, Screen, Session},
    atmosphere::director::{Cue, Director},
};
use ainda_vivo_core::story::Phase;
use bevy::{light::NotShadowCaster, prelude::*};

pub const FILE: Vec3 = Vec3::new(-6.76, 0.952, -16.50);
pub const LENS: Vec3 = Vec3::new(23.37, 1.52, -85.63);
pub const CLOCK: Vec3 = Vec3::new(22.50, 2.37, -85.62);
pub const RECEIPT_DISPLAY: Vec3 = Vec3::new(0.0, 1.85, -98.48);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpressionId {
    File,
    IsolatedCircuit,
    Clock,
    ShiftTag,
}

#[derive(Clone)]
struct Variant {
    pose: Transform,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Impression {
    pub id: ImpressionId,
    phase: Phase,
    normal: Variant,
    changed: Variant,
    second: Option<Variant>,
    noticed: bool,
    stage: u8,
    last_change: f32,
}

impl Impression {
    pub fn is_applied(&self) -> bool {
        self.stage > 0
    }

    pub fn stage(&self) -> u8 {
        self.stage
    }
}

#[derive(Component)]
pub struct ReceiptDisplay {
    live: Handle<StandardMaterial>,
    receipt: Handle<StandardMaterial>,
    pending: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct Illusions {
    pub attempt: u64,
    revision: u32,
    pub altered: u8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Attention {
    Inspecting,
    Away,
    Between,
}

fn attention(facing: f32, clear_ray: bool) -> Attention {
    if facing > 0.89 && clear_ray {
        Attention::Inspecting
    } else if facing < -0.12 {
        Attention::Away
    } else {
        Attention::Between
    }
}

type ColorBytes = ([u8; 3], [u8; 3]);

fn plate(
    word: &str,
    palette: ColorBytes,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color_texture: Some(images.add(super::surfaces::sign(word, palette.0, palette.1))),
        unlit: true,
        ..default()
    })
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let file_color = materials.add(StandardMaterial {
        base_color: Color::srgb(0.33, 0.12, 0.09),
        perceptual_roughness: 0.91,
        ..default()
    });
    let lens_off = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.20, 0.17),
        perceptual_roughness: 0.47,
        ..default()
    });
    let lens_on = materials.add(StandardMaterial {
        base_color: Color::srgb(0.33, 0.54, 0.41),
        emissive: LinearRgba::rgb(0.28, 0.56, 0.37),
        ..default()
    });
    let normal_file = Transform::from_translation(FILE);
    let changed_file = Transform::from_translation(FILE + Vec3::new(0.26, 0.0, 0.12))
        .with_rotation(Quat::from_rotation_y(0.43));
    let lens = Transform::from_translation(LENS);
    let clock = Transform::from_translation(CLOCK);
    let tag = Transform::from_xyz(3.43, 1.67, -57.68);
    let sign_mesh = meshes.add(Rectangle::new(0.82, 0.22));
    let clock_skin = ([29, 43, 41], [146, 182, 160]);
    let tag_skin = ([51, 43, 29], [172, 153, 110]);
    let clock_normal = plate("02:48", clock_skin, &mut materials, &mut images);
    let clock_later = plate("03:12", clock_skin, &mut materials, &mut images);
    let variant = |pose, material| Variant { pose, material };

    for (id, phase, mesh, normal, changed, second) in [
        (
            ImpressionId::File,
            Phase::Fuse,
            meshes.add(Cuboid::new(0.23, 0.026, 0.32)),
            variant(normal_file, file_color.clone()),
            variant(changed_file, file_color.clone()),
            Some(variant(
                normal_file.with_rotation(Quat::from_rotation_y(-0.61)),
                file_color,
            )),
        ),
        (
            ImpressionId::IsolatedCircuit,
            Phase::Return,
            meshes.add(Cuboid::new(0.16, 0.07, 0.045)),
            variant(lens, lens_off),
            variant(lens, lens_on),
            None,
        ),
        (
            ImpressionId::Clock,
            Phase::Return,
            sign_mesh.clone(),
            variant(clock, clock_normal.clone()),
            variant(clock, clock_later),
            Some(variant(clock, clock_normal)),
        ),
        (
            ImpressionId::ShiftTag,
            Phase::Baseline,
            sign_mesh,
            variant(
                tag,
                plate("TURNO 07", tag_skin, &mut materials, &mut images),
            ),
            variant(
                tag,
                plate("TURNO 02", tag_skin, &mut materials, &mut images),
            ),
            None,
        ),
    ] {
        commands.spawn((
            Name::new(format!("{id:?}")),
            Mesh3d(mesh),
            MeshMaterial3d(normal.material.clone()),
            normal.pose,
            Impression {
                id,
                phase,
                normal,
                changed,
                second,
                noticed: false,
                stage: 0,
                last_change: 0.0,
            },
            NotShadowCaster,
        ));
    }

    let display_skin = ([19, 35, 32], [129, 171, 147]);
    let live = plate("PORTARIA 07", display_skin, &mut materials, &mut images);
    commands.spawn((
        Name::new("Leitor de baixa da portaria"),
        Mesh3d(meshes.add(Rectangle::new(2.06, 0.37))),
        MeshMaterial3d(live.clone()),
        Transform::from_translation(RECEIPT_DISPLAY),
        ReceiptDisplay {
            live,
            receipt: plate("BAIXA RECEBIDA", display_skin, &mut materials, &mut images),
            pending: plate(
                "ENTRADA PENDENTE",
                display_skin,
                &mut materials,
                &mut images,
            ),
        },
        NotShadowCaster,
    ));
}

type Props<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Impression,
        &'static mut Transform,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
    Without<MainCamera>,
>;

pub fn update(
    session: Res<Session>,
    world: Res<CollisionWorld>,
    camera: Query<&Transform, With<MainCamera>>,
    mut director: ResMut<Director>,
    mut state: ResMut<Illusions>,
    mut props: Props,
    mut displays: Query<
        (&ReceiptDisplay, &mut MeshMaterial3d<StandardMaterial>),
        Without<Impression>,
    >,
) {
    let restarted =
        (state.attempt, state.revision) != (session.attempt, session.perception_revision);
    if restarted {
        *state = Illusions {
            attempt: session.attempt,
            revision: session.perception_revision,
            ..default()
        };
    }
    for (display, mut material) in &mut displays {
        let desired = match session.false_exit {
            None => &display.live,
            Some(ExitPresentation::Receipt { .. }) => &display.receipt,
            Some(ExitPresentation::Contradiction) => &display.pending,
        };
        if &material.0 != desired {
            material.0 = desired.clone();
        }
    }
    let Ok(camera) = camera.single() else { return };
    for (mut impression, mut pose, mut material) in &mut props {
        if restarted {
            impression.noticed = false;
            impression.stage = 0;
            impression.last_change = 0.0;
            *pose = impression.normal.pose;
            material.0 = impression.normal.material.clone();
        }
        if session.screen != Screen::Playing || session.progress().phase() < impression.phase {
            continue;
        }
        let now = session.elapsed;
        let next = match impression.stage {
            0 => &impression.changed,
            1 if session.expedition.observer().distortion_ready(now)
                && director.revisit
                && now - impression.last_change > 14.0 =>
            {
                let Some(second) = &impression.second else {
                    continue;
                };
                second
            }
            _ => continue,
        };
        // The renderer reads the same small set of entities in every reality.
        // Only cast an occlusion ray when the player is actually inspecting one.
        let offset = pose.translation + Vec3::Y * 0.032 - camera.translation;
        if offset.length_squared() > 5.5 * 5.5 {
            continue;
        }
        let facing = camera.forward().dot(offset.normalize_or_zero());
        let clear_ray = facing > 0.89
            && !world.occludes(
                camera.translation,
                camera.translation + offset,
                session.progress().exit_unlocked(),
            );
        match attention(facing, clear_ray) {
            Attention::Inspecting => impression.noticed = true,
            Attention::Away if impression.noticed && director.allows(Cue::Illusion, now) => {
                *pose = next.pose;
                material.0 = next.material.clone();
                impression.stage += 1;
                impression.noticed = false;
                impression.last_change = now;
                state.altered += 1;
                director.claim(Cue::Illusion, now);
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_requires_an_initial_clear_look_and_changes_behind_the_observer() {
        assert_eq!(attention(1.0, true), Attention::Inspecting);
        assert_eq!(attention(1.0, false), Attention::Between);
        assert_eq!(attention(0.0, true), Attention::Between);
        assert_eq!(attention(-1.0, true), Attention::Away);
    }
}
