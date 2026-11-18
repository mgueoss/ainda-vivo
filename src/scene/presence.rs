//! Brief, local sightings at authored positions inside Estação 07.
use super::surfaces;
use crate::{
    app::{CollisionWorld, MainCamera, Screen, Session},
    atmosphere::Atmosphere,
};
use ainda_vivo_core::story::{Phase, Progress};
use bevy::{light::NotShadowCaster, prelude::*};

pub(super) const WIDTH: f32 = 1.56;
pub(super) const HEIGHT: f32 = 2.48;
const MIN_DISTANCE: f32 = 6.4;
const MAX_DISTANCE: f32 = 14.3;
const FADE_IN: f32 = 0.44;
const FADE_OUT: f32 = 0.83;

pub const FIRST_MARK: Vec3 = Vec3::new(2.55, 0.016, -21.36);
pub const FIRST_WITNESS: Vec3 = Vec3::new(0.0, 1.62, -8.8);

struct Site {
    name: &'static str,
    feet: Vec3,
    witness: Vec3,
    from: Phase,
    through: Phase,
    motion: Motion,
}

#[derive(Clone, Copy)]
enum Motion {
    Watch,
    Lean(f32),
    Cross(f32),
}

impl Motion {
    fn hold(self) -> f32 {
        match self {
            Self::Watch => 4.65,
            Self::Lean(_) => 3.48,
            Self::Cross(_) => 2.55,
        }
    }

    fn pose(self, age: f32, facing: Quat) -> (Vec3, Quat) {
        match self {
            Self::Watch => (facing * Vec3::X * (0.28 * smooth(age / 1.03)), facing),
            Self::Lean(distance) => {
                let lean = smooth(age / 1.15);
                (
                    facing * Vec3::X * (distance * lean),
                    facing * Quat::from_rotation_z(-0.097 * lean * distance.signum()),
                )
            }
            Self::Cross(distance) => {
                let crossing = smooth((age - 0.52) / 2.55);
                (
                    Vec3::X * (distance * crossing),
                    facing * Quat::from_rotation_y(0.33 * crossing * distance.signum()),
                )
            }
        }
    }
}

const SITES: [Site; 10] = [
    Site {
        name: "Triagem",
        feet: FIRST_MARK,
        witness: FIRST_WITNESS,
        from: Phase::Fuse,
        through: Phase::Baseline,
        motion: Motion::Watch,
    },
    Site {
        name: "Entre as bombas",
        feet: Vec3::new(4.13, 0.016, -43.65),
        witness: Vec3::new(-0.9, 1.62, -32.0),
        from: Phase::Baseline,
        through: Phase::Electrical,
        motion: Motion::Lean(0.66),
    },
    Site {
        name: "Acesso da Elétrica",
        feet: Vec3::new(10.32, 0.016, -51.35),
        witness: Vec3::new(-0.8, 1.62, -46.8),
        from: Phase::Electrical,
        through: Phase::Electrical,
        motion: Motion::Lean(0.73),
    },
    Site {
        name: "Do outro lado da porta da Elétrica",
        feet: Vec3::new(9.30, 0.016, -51.47),
        witness: Vec3::new(21.2, 1.62, -52.50),
        from: Phase::Return,
        through: Phase::Return,
        motion: Motion::Lean(-0.78),
    },
    Site {
        name: "Vão oeste da nave",
        feet: Vec3::new(-3.62, 0.016, -54.10),
        witness: Vec3::new(1.6, 1.62, -44.10),
        from: Phase::Return,
        through: Phase::Exit,
        motion: Motion::Lean(-0.61),
    },
    Site {
        name: "Passarela leste",
        feet: Vec3::new(7.16, 0.016, -80.25),
        witness: Vec3::new(7.92, 1.62, -69.05),
        from: Phase::Exit,
        through: Phase::Exit,
        motion: Motion::Cross(-2.34),
    },
    Site {
        name: "Passarela oeste",
        feet: Vec3::new(-7.15, 0.016, -80.36),
        witness: Vec3::new(-7.70, 1.62, -69.64),
        from: Phase::Exit,
        through: Phase::Exit,
        motion: Motion::Cross(2.25),
    },
    Site {
        name: "Atrás do guarda-corpo",
        feet: Vec3::new(-5.29, 0.016, -75.45),
        witness: Vec3::new(6.42, 1.62, -69.1),
        from: Phase::Exit,
        through: Phase::Exit,
        motion: Motion::Lean(0.67),
    },
    Site {
        name: "Carga",
        feet: Vec3::new(-21.85, 0.016, -1.89),
        witness: Vec3::new(-11.25, 1.62, 4.2),
        from: Phase::Fuse,
        through: Phase::Exit,
        motion: Motion::Watch,
    },
    Site {
        name: "Inspeção",
        feet: Vec3::new(-25.0, 0.016, -76.15),
        witness: Vec3::new(-13.2, 1.62, -74.0),
        from: Phase::Electrical,
        through: Phase::Exit,
        motion: Motion::Lean(-0.68),
    },
];

impl Site {
    fn target(&self) -> Vec3 {
        self.feet + Vec3::Y * 1.74
    }

    fn available(
        &self,
        camera: &Transform,
        lens: &PerspectiveProjection,
        world: &CollisionWorld,
        gate_open: bool,
    ) -> bool {
        let range = self.target().distance(camera.translation);
        (MIN_DISTANCE + 2.0..=MAX_DISTANCE).contains(&range)
            && self.in_view(Vec3::ZERO, camera, lens, world, gate_open)
    }

    fn in_view(
        &self,
        offset: Vec3,
        camera: &Transform,
        lens: &PerspectiveProjection,
        world: &CollisionWorld,
        gate_open: bool,
    ) -> bool {
        let target = self.target() + offset;
        // Leave room for the outline even with a narrow window or a smaller FOV.
        let local = camera.rotation.inverse() * (target - camera.translation);
        let half_height = -local.z * (lens.fov * 0.5).tan() * 0.82;
        local.y.abs() < half_height
            && local.x.abs() < half_height * lens.aspect_ratio
            && !world.occludes(camera.translation, target, gate_open)
    }
}

struct Sighting {
    site: usize,
    since: f32,
    checked_at: f32,
    framed_for: f32,
    rotation: Quat,
}

#[derive(Resource, Default)]
pub struct Presence {
    attempt: u64,
    seen: [bool; SITES.len()],
    next_at: f32,
    last_phase: Phase,
    sighting: Option<Sighting>,
}

impl Presence {
    pub fn active(&self) -> bool {
        self.sighting.is_some()
    }

    pub fn observed_count(&self) -> usize {
        self.seen.iter().filter(|seen| **seen).count()
    }

    pub fn crossing(&self, elapsed: f32) -> bool {
        self.sighting.as_ref().is_some_and(|sighting| {
            matches!(SITES[sighting.site].motion, Motion::Cross(_))
                && (1.0..1.9).contains(&(elapsed - sighting.since))
        })
    }

    fn advance(
        &mut self,
        elapsed: f32,
        progress: &Progress,
        camera: &Transform,
        lens: &PerspectiveProjection,
        world: &CollisionWorld,
        quiet_beat: bool,
    ) -> Option<(Transform, f32)> {
        let phase = progress.phase();
        let gate_open = progress.exit_unlocked();
        if self.last_phase != phase {
            self.last_phase = phase;
            self.next_at = self.next_at.min(elapsed + 1.8);
        }
        if self.sighting.is_none()
            && elapsed >= self.next_at
            && !quiet_beat
            && let Some((index, site)) = SITES.iter().enumerate().find(|(index, site)| {
                !self.seen[*index]
                    && (site.from..=site.through).contains(&phase)
                    && site.available(camera, lens, world, gate_open)
            })
        {
            let bearing = (camera.translation - site.feet).with_y(0.0);
            // Fix the silhouette's orientation at the start of the sighting.
            let rotation = Transform::default().looking_to(-bearing, Vec3::Y).rotation;
            self.sighting = Some(Sighting {
                site: index,
                since: elapsed,
                checked_at: elapsed,
                framed_for: 0.0,
                rotation,
            });
            debug!("A Presença: {} em {elapsed:.2}s", site.name);
        }

        let sighting = self.sighting.as_mut()?;
        let site = &SITES[sighting.site];
        let age = elapsed - sighting.since;
        let (offset, motion) = site.motion.pose(age, sighting.rotation);
        let distance = camera.translation.xz().distance((site.feet + offset).xz());
        let duration = site.motion.hold() + FADE_IN + FADE_OUT;
        if distance <= MIN_DISTANCE || distance > MAX_DISTANCE + 2.2 || age >= duration {
            self.next_at = elapsed + if self.seen[sighting.site] { 4.5 } else { 2.7 };
            self.sighting = None;
            return None;
        }

        let fade = smooth(age / FADE_IN) * smooth((duration - age) / FADE_OUT);
        let range =
            smooth((distance - MIN_DISTANCE) / 2.3) * smooth((MAX_DISTANCE + 2.2 - distance) / 2.2);
        let alpha = 0.86 * fade * range;
        // A glimpse that happened while rounding a corner can try again on return.
        if !self.seen[sighting.site]
            && alpha >= 0.48
            && site.in_view(offset, camera, lens, world, gate_open)
        {
            sighting.framed_for += (elapsed - sighting.checked_at).min(0.075);
            self.seen[sighting.site] = sighting.framed_for >= 0.46;
        }
        sighting.checked_at = elapsed;
        let transform = Transform::from_translation(site.feet + offset + Vec3::Y * (HEIGHT * 0.5))
            .with_rotation(motion);
        Some((transform, alpha))
    }
}

#[derive(Component)]
pub struct Silhouette;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mask = images.add(mask());
    commands.spawn((
        Name::new("A Presença"),
        Silhouette,
        Mesh3d(meshes.add(Rectangle::new(WIDTH, HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.0),
            base_color_texture: Some(mask),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
    ));
}

type SilhouetteView<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Visibility,
        &'static MeshMaterial3d<StandardMaterial>,
    ),
    (With<Silhouette>, Without<MainCamera>),
>;

pub fn update(
    session: Res<Session>,
    world: Res<CollisionWorld>,
    atmosphere: Res<Atmosphere>,
    cameras: Query<(&Transform, &Projection), With<MainCamera>>,
    mut presence: ResMut<Presence>,
    mut figure: SilhouetteView,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if presence.attempt != session.attempt {
        *presence = Presence {
            attempt: session.attempt,
            ..default()
        };
    }
    if session.screen == Screen::Paused {
        return;
    }
    let Ok((mut transform, mut visibility, material)) = figure.single_mut() else {
        return;
    };
    if session.screen == Screen::Playing
        && let Ok((camera, Projection::Perspective(lens))) = cameras.single()
        && let Some((pose, alpha)) = presence.advance(
            session.elapsed,
            &session.progress,
            camera,
            lens,
            &world,
            atmosphere.episode.pump_hush > 0.26
                || atmosphere
                    .episode
                    .trace
                    .is_some_and(|trace| trace.age < 1.24),
        )
    {
        *transform = pose;
        *visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(material) {
            material.base_color = Color::WHITE.with_alpha(alpha);
        }
    } else if *visibility != Visibility::Hidden {
        *visibility = Visibility::Hidden;
        presence.sighting = None;
    }
}

/// Verify one reachable sightline for each authored position against the actual station.
pub fn audit_sites(world: &CollisionWorld) -> Result<usize, &'static str> {
    for site in &SITES {
        let camera = Transform::from_translation(site.witness).looking_at(site.target(), Vec3::Y);
        if !site.available(&camera, &PerspectiveProjection::default(), world, false) {
            return Err(site.name);
        }
    }
    Ok(SITES.len())
}

pub(super) fn mask() -> Image {
    const SIZE: UVec2 = UVec2::new(128, 224);
    let mut pixels = Vec::with_capacity((SIZE.x * SIZE.y * 4) as usize);
    for y in 0..SIZE.y {
        for x in 0..SIZE.x {
            let uv = (Vec2::new(x as f32, y as f32) + 0.5) / SIZE.as_vec2();
            let p = Vec2::new((uv.x - 0.5) * WIDTH, (1.0 - uv.y) * HEIGHT);
            let ellipse = |center: Vec2, radius: Vec2| {
                (((p - center) / radius).length() - 1.0) * radius.min_element()
            };
            let limb = |a: Vec2, b: Vec2, start: f32, end: f32| {
                let axis = b - a;
                let t = ((p - a).dot(axis) / axis.length_squared()).clamp(0.0, 1.0);
                p.distance(a + axis * t) - (start + (end - start) * t)
            };
            let edge = ellipse(Vec2::new(0.072, 2.16), Vec2::new(0.17, 0.221))
                .min(limb(
                    Vec2::new(0.064, 2.08),
                    Vec2::new(-0.013, 1.65),
                    0.079,
                    0.141,
                ))
                .min(ellipse(Vec2::new(-0.037, 1.54), Vec2::new(0.33, 0.391)))
                .min(ellipse(Vec2::new(-0.022, 1.00), Vec2::new(0.217, 0.62)))
                .min(limb(
                    Vec2::new(-0.267, 1.68),
                    Vec2::new(-0.433, 0.56),
                    0.113,
                    0.041,
                ))
                .min(limb(
                    Vec2::new(0.236, 1.74),
                    Vec2::new(0.391, 0.76),
                    0.093,
                    0.049,
                ))
                .min(limb(
                    Vec2::new(-0.12, 0.88),
                    Vec2::new(-0.21, 0.04),
                    0.133,
                    0.066,
                ))
                .min(limb(
                    Vec2::new(0.08, 0.87),
                    Vec2::new(0.138, 0.13),
                    0.139,
                    0.067,
                ));
            let grain = surfaces::noise(uv, IVec2::new(23, 34), 71);
            let wisps = surfaces::noise(uv, IVec2::new(9, 11), 29);
            let fringe = smooth((0.048 - edge + (grain - 0.5) * 0.037) / 0.083);
            let density = (0.88 + 0.12 * wisps) * smooth((p.y - 0.032) / 0.58);
            let rim = (1.0 - edge.abs() / 0.072).max(0.0) * (p.x + 0.06).max(0.0);
            pixels.extend_from_slice(&[
                (2.0 + 23.0 * rim) as u8,
                (3.0 + 28.0 * rim) as u8,
                (4.0 + 33.0 * rim) as u8,
                (255.0 * fringe * density) as u8,
            ]);
        }
    }
    surfaces::texture(SIZE.x, SIZE.y, pixels, false)
}

fn smooth(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appearance_requires_distance_framing_and_a_clear_ray() {
        let site = &SITES[0];
        let camera = Transform::from_translation(site.target() + Vec3::Z * 12.0)
            .looking_at(site.target(), Vec3::Y);
        let lens = PerspectiveProjection::default();
        let mut world = CollisionWorld::default();
        assert!(site.available(&camera, &lens, &world, false));
        world.0.push(crate::app::Solid {
            bounds: ainda_vivo_core::collision::Aabb::from_center_size(
                site.target() + Vec3::Z * 8.0,
                Vec3::new(3.7, 4.0, 0.2),
            ),
            gate: true,
        });
        assert!(!site.available(&camera, &lens, &world, false));
        assert!(site.available(&camera, &lens, &world, true));
        let close = camera.with_translation(site.target() + Vec3::Z * 7.0);
        assert!(!site.available(&close, &lens, &world, true));
        let far = camera.with_translation(site.target() + Vec3::Z * 17.0);
        assert!(!site.available(&far, &lens, &world, true));
        let away = camera.looking_to(Vec3::Z, Vec3::Y);
        assert!(!site.available(&away, &lens, &world, true));
    }

    #[test]
    fn a_centered_gaze_gives_time_to_read_the_figure() {
        let site = &SITES[0];
        let mut presence = Presence::default();
        let camera = Transform::from_translation(site.target() + Vec3::Z * 12.0)
            .looking_at(site.target(), Vec3::Y);
        let lens = PerspectiveProjection::default();
        let world = CollisionWorld::default();
        let progress = Progress::default();
        let mut advance =
            |time, view| presence.advance(time, &progress, view, &lens, &world, false);
        let (first, start) = advance(0.0, &camera).unwrap();
        assert_eq!(start, 0.0);
        for frame in 1..=180 {
            let dt = frame as f32 / 60.0;
            let (later, alpha) = advance(dt, &camera).unwrap();
            assert!(first.translation.distance(later.translation) < 0.35);
            if dt >= FADE_IN {
                assert!(alpha > 0.72);
            }
        }
        assert!(advance(6.3, &camera).is_none());
        assert!(advance(100.0, &camera).is_none());
    }

    #[test]
    fn if_the_player_turns_away_the_sighting_can_retry() {
        let site = &SITES[0];
        let mut presence = Presence::default();
        let camera = Transform::from_translation(site.target() + Vec3::Z * 12.0)
            .looking_at(site.target(), Vec3::Y);
        let away = camera.looking_to(Vec3::Z, Vec3::Y);
        let lens = PerspectiveProjection::default();
        let world = CollisionWorld::default();
        let progress = Progress::default();
        assert!(
            presence
                .advance(0.0, &progress, &camera, &lens, &world, false)
                .is_some()
        );
        assert!(
            presence
                .advance(6.3, &progress, &away, &lens, &world, false)
                .is_none()
        );
        assert!(
            presence
                .advance(7.0, &progress, &camera, &lens, &world, false)
                .is_none()
        );
        assert!(
            presence
                .advance(9.2, &progress, &camera, &lens, &world, false)
                .is_some()
        );
    }
}
