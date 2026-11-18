//! Observer-local dressing. Illusions never own a collider or a mission interaction.
use crate::{
    app::{CollisionWorld, MainCamera, Screen, Session},
    atmosphere::director::{Cue, Director},
};
use ainda_vivo_core::story::Phase;
use bevy::{light::NotShadowCaster, prelude::*};

pub const FILE: Vec3 = Vec3::new(-6.76, 0.952, -16.50);
pub const LENS: Vec3 = Vec3::new(23.37, 1.52, -85.63);

#[derive(Component)]
pub struct Impression {
    normal: Transform,
    altered: Transform,
    phase: Phase,
    dormant: Handle<StandardMaterial>,
    changed: Handle<StandardMaterial>,
    noticed: bool,
    applied: bool,
}

impl Impression {
    pub fn is_applied(&self) -> bool {
        self.applied
    }
}

#[derive(Resource, Default)]
pub struct Illusions {
    pub attempt: u64,
    pub altered: u8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Attention {
    Inspecting,
    Away,
    Between,
}

fn attention(camera: &Transform, position: Vec3, clear_ray: bool) -> Attention {
    let offset = position - camera.translation;
    let facing = camera.forward().dot(offset.normalize_or_zero());
    if facing > 0.89 && clear_ray {
        Attention::Inspecting
    } else if facing < -0.12 {
        Attention::Away
    } else {
        Attention::Between
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    for (name, size, impression) in [
        (
            "Ficha de turno sem rubrica",
            Vec3::new(0.23, 0.026, 0.32),
            Impression {
                normal: normal_file,
                altered: changed_file,
                phase: Phase::Fuse,
                dormant: file_color.clone(),
                changed: file_color,
                noticed: false,
                applied: false,
            },
        ),
        (
            "Indicador do ramal isolado",
            Vec3::new(0.16, 0.07, 0.045),
            Impression {
                normal: lens,
                altered: lens,
                phase: Phase::Return,
                dormant: lens_off,
                changed: lens_on,
                noticed: false,
                applied: false,
            },
        ),
    ] {
        commands.spawn((
            Name::new(name),
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(impression.dormant.clone()),
            impression.normal,
            impression,
            NotShadowCaster,
        ));
    }
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
) {
    let restarted = state.attempt != session.attempt;
    if restarted {
        *state = Illusions {
            attempt: session.attempt,
            ..default()
        };
    }
    if session.screen != Screen::Playing {
        return;
    }
    let Ok(camera) = camera.single() else { return };
    for (mut impression, mut pose, mut material) in &mut props {
        if restarted {
            impression.noticed = false;
            impression.applied = false;
            *pose = impression.normal;
            material.0 = impression.dormant.clone();
        }
        if impression.applied || session.progress.phase() < impression.phase {
            continue;
        }
        let position = impression.normal.translation + Vec3::Y * 0.032;
        if camera.translation.distance_squared(position) > 5.5 * 5.5 {
            continue;
        }
        let clear_ray = !world.occludes(
            camera.translation,
            position,
            session.progress.exit_unlocked(),
        );
        match attention(camera, position, clear_ray) {
            Attention::Inspecting => impression.noticed = true,
            Attention::Away
                if impression.noticed && director.allows(Cue::Illusion, session.elapsed) =>
            {
                impression.applied = true;
                *pose = impression.altered;
                material.0 = impression.changed.clone();
                state.altered += 1;
                director.claim(Cue::Illusion, session.elapsed);
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
        let camera = Transform::IDENTITY;
        assert_eq!(attention(&camera, Vec3::NEG_Z, true), Attention::Inspecting);
        assert_eq!(attention(&camera, Vec3::NEG_Z, false), Attention::Between);
        assert_eq!(attention(&camera, Vec3::X, true), Attention::Between);
        assert_eq!(attention(&camera, Vec3::Z, true), Attention::Away);
    }
}
