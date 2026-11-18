//! Brief, local sightings at authored positions inside Estação 07.
use super::surfaces;
use crate::{
    app::{CollisionWorld, MainCamera, Screen, Session},
    atmosphere::Atmosphere,
};
use bevy::{light::NotShadowCaster, prelude::*};

const WIDTH: f32 = 1.32;
const HEIGHT: f32 = 2.33;
const FIRST_SIGHTING: f32 = 6.5;
const MIN_DISTANCE: f32 = 12.0;
const MAX_DISTANCE: f32 = 28.0;
const FADE_IN: f32 = 0.68;
const FADE_OUT: f32 = 0.74;
const DIRECT_GAZE: f32 = 0.9925;

struct Site {
    name: &'static str,
    feet: Vec3,
}

const SITES: [Site; 5] = [
    Site {
        name: "Triagem",
        feet: Vec3::new(6.38, 0.016, -25.15),
    },
    Site {
        name: "Carga",
        feet: Vec3::new(-28.55, 0.016, -0.50),
    },
    Site {
        name: "Bombas",
        feet: Vec3::new(4.12, 0.016, -55.60),
    },
    Site {
        name: "Decantação",
        feet: Vec3::new(7.95, 0.016, -78.40),
    },
    Site {
        name: "Inspeção",
        feet: Vec3::new(-30.42, 0.016, -75.82),
    },
];

impl Site {
    fn target(&self) -> Vec3 {
        self.feet + Vec3::Y * 1.68
    }

    fn available(
        &self,
        camera: &Transform,
        lens: &PerspectiveProjection,
        world: &CollisionWorld,
        power: bool,
    ) -> bool {
        let ray = self.target() - camera.translation;
        if !(MIN_DISTANCE + 2.5..=MAX_DISTANCE).contains(&ray.length()) {
            return false;
        }
        // Leave room for the outline even with a narrow window or a smaller FOV.
        let local = camera.rotation.inverse() * ray;
        let half_height = -local.z * (lens.fov * 0.5).tan() * 0.82;
        local.y.abs() < half_height
            && local.x.abs() < half_height * lens.aspect_ratio
            && !world.occludes(camera.translation, self.target(), power)
    }
}

struct Sighting {
    site: usize,
    since: f32,
    gaze_since: Option<f32>,
    fade_at: f32,
    rotation: Quat,
}

#[derive(Resource)]
pub struct Presence {
    attempt: u64,
    seen: [bool; SITES.len()],
    next_at: f32,
    sighting: Option<Sighting>,
}

impl Default for Presence {
    fn default() -> Self {
        Self {
            attempt: 0,
            seen: [false; SITES.len()],
            next_at: FIRST_SIGHTING,
            sighting: None,
        }
    }
}

impl Presence {
    pub fn active(&self) -> bool {
        self.sighting.is_some()
    }

    fn advance(
        &mut self,
        elapsed: f32,
        power: bool,
        quiet: bool,
        camera: &Transform,
        lens: &PerspectiveProjection,
        world: &CollisionWorld,
    ) -> Option<(Transform, f32)> {
        if self.sighting.is_none()
            && elapsed >= self.next_at
            && !quiet
            && let Some((index, site)) = SITES.iter().enumerate().find(|(index, site)| {
                !self.seen[*index] && site.available(camera, lens, world, power)
            })
        {
            let bearing = (camera.translation - site.feet).with_y(0.0);
            // The silhouette is fixed in world space for the entire sighting.
            let rotation = Transform::default().looking_to(-bearing, Vec3::Y).rotation;
            self.seen[index] = true;
            self.sighting = Some(Sighting {
                site: index,
                since: elapsed,
                gaze_since: None,
                fade_at: elapsed + 3.15 + (index % 3) as f32 * 0.28,
                rotation,
            });
            debug!("A Presença: {} em {elapsed:.2}s", site.name);
        }

        let sighting = self.sighting.as_mut()?;
        let site = &SITES[sighting.site];
        let ray = site.target() - camera.translation;
        let distance = camera.translation.xz().distance(site.feet.xz());
        let age = elapsed - sighting.since;
        if ray.normalize_or_zero().dot(*camera.forward()) >= DIRECT_GAZE {
            let since = *sighting.gaze_since.get_or_insert(elapsed);
            if elapsed - since >= 0.82 && age >= 1.05 {
                sighting.fade_at = sighting.fade_at.min(elapsed);
            }
        } else {
            sighting.gaze_since = None;
        }
        if distance <= MIN_DISTANCE
            || distance > MAX_DISTANCE + 3.0
            || elapsed >= sighting.fade_at + FADE_OUT
        {
            self.next_at = elapsed + 23.0 + (sighting.site % 3) as f32 * 4.5;
            self.sighting = None;
            return None;
        }

        let fade =
            smooth(age / FADE_IN) * smooth((sighting.fade_at + FADE_OUT - elapsed) / FADE_OUT);
        let range =
            smooth((distance - MIN_DISTANCE) / 4.0) * smooth((MAX_DISTANCE + 3.0 - distance) / 5.0);
        let transform = Transform::from_translation(site.feet + Vec3::Y * (HEIGHT * 0.5))
            .with_rotation(sighting.rotation);
        Some((transform, 0.40 * fade * range))
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
            base_color: Color::BLACK.with_alpha(0.0),
            base_color_texture: Some(mask),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            fog_enabled: false,
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
            session.progress.power_restored,
            atmosphere.quiet > 0.14,
            camera,
            lens,
            &world,
        )
    {
        *transform = pose;
        *visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(material) {
            material.base_color = Color::BLACK.with_alpha(alpha);
        }
    } else if *visibility != Visibility::Hidden {
        *visibility = Visibility::Hidden;
        presence.sighting = None;
    }
}

/// Verify one reachable sightline for each authored position against the actual station.
pub fn audit_sites(world: &CollisionWorld) -> Result<usize, &'static str> {
    let witnesses = [
        Vec3::new(0.00, 1.62, -8.55),
        Vec3::new(-11.25, 1.62, 4.2),
        Vec3::new(-1.20, 1.62, -36.50),
        Vec3::new(-5.10, 1.62, -61.75),
        Vec3::new(-13.2, 1.62, -74.0),
    ];
    for (site, witness) in SITES.iter().zip(witnesses) {
        let camera = Transform::from_translation(witness).looking_at(site.target(), Vec3::Y);
        if !site.available(&camera, &PerspectiveProjection::default(), world, false) {
            return Err(site.name);
        }
    }
    Ok(SITES.len())
}

fn mask() -> Image {
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
            let edge = ellipse(Vec2::new(0.072, 2.047), Vec2::new(0.145, 0.204))
                .min(limb(
                    Vec2::new(0.064, 1.97),
                    Vec2::new(-0.013, 1.65),
                    0.079,
                    0.141,
                ))
                .min(ellipse(Vec2::new(-0.037, 1.49), Vec2::new(0.283, 0.367)))
                .min(ellipse(Vec2::new(-0.022, 1.00), Vec2::new(0.217, 0.62)))
                .min(limb(
                    Vec2::new(-0.245, 1.60),
                    Vec2::new(-0.375, 0.56),
                    0.113,
                    0.041,
                ))
                .min(limb(
                    Vec2::new(0.166, 1.65),
                    Vec2::new(0.325, 0.76),
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
            let fringe = smooth((0.058 - edge + (grain - 0.5) * 0.056) / 0.106);
            let density = (0.73 + 0.25 * wisps) * smooth((p.y - 0.032) / 0.75);
            pixels.extend_from_slice(&[255, 255, 255, (255.0 * fringe * density) as u8]);
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
        let camera = Transform::from_translation(site.target() + Vec3::Z * 19.0)
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
        let close = camera.with_translation(site.target() + Vec3::Z * 8.0);
        assert!(!site.available(&close, &lens, &world, true));
        let away = camera.looking_to(Vec3::Z, Vec3::Y);
        assert!(!site.available(&away, &lens, &world, true));
    }

    #[test]
    fn a_centered_gaze_dissolves_the_figure_and_the_site_stays_spent() {
        let site = &SITES[0];
        let mut presence = Presence::default();
        let camera = Transform::from_translation(site.target() + Vec3::Z * 18.0)
            .looking_at(site.target(), Vec3::Y);
        let lens = PerspectiveProjection::default();
        let world = CollisionWorld::default();
        let mut advance = |time, view| presence.advance(time, false, false, view, &lens, &world);
        assert!(advance(FIRST_SIGHTING - 0.1, &camera).is_none());
        let (first, start) = advance(FIRST_SIGHTING, &camera).unwrap();
        assert_eq!(start, 0.0);
        let (later, peak) = advance(FIRST_SIGHTING + 1.06, &camera).unwrap();
        assert_eq!(first, later);
        assert!(peak > 0.35);
        assert!(advance(FIRST_SIGHTING + 2.0, &camera).is_none());
        assert!(advance(FIRST_SIGHTING + 100.0, &camera).is_none());
    }
}
