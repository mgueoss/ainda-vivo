use crate::Vec3;

const EPS: f32 = 1.0e-6;
pub const SKIN: f32 = 0.001;

/// Axis-aligned, static geometry in world metres. Visible geometry must match it.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        Self {
            min: center - size * 0.5,
            max: center + size * 0.5,
        }
    }

    pub fn expanded(self, half: Vec3) -> Self {
        Self {
            min: self.min - half,
            max: self.max + half,
        }
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.min.cmplt(other.max - Vec3::splat(EPS)).all()
            && self.max.cmpgt(other.min + Vec3::splat(EPS)).all()
    }

    /// Slab ray cast. Returns distance in metres for a unit direction.
    pub fn ray(self, origin: Vec3, direction: Vec3, limit: f32) -> Option<f32> {
        let mut near = 0.0_f32;
        let mut far = limit;
        for axis in 0..3 {
            if direction[axis].abs() < EPS {
                if origin[axis] < self.min[axis] || origin[axis] > self.max[axis] {
                    return None;
                }
            } else {
                let a = (self.min[axis] - origin[axis]) / direction[axis];
                let b = (self.max[axis] - origin[axis]) / direction[axis];
                near = near.max(a.min(b));
                far = far.min(a.max(b));
                if near > far {
                    return None;
                }
            }
        }
        (far >= 0.0 && near <= limit).then_some(near)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub fraction: f32,
    pub normal: Vec3,
}

/// Sweep a point through an expanded collider (Minkowski sum).
/// Boundary contact while moving away or parallel is deliberately not a hit.
pub fn sweep_point(origin: Vec3, delta: Vec3, bounds: Aabb) -> Option<Hit> {
    let mut entry = f32::NEG_INFINITY;
    let mut exit = f32::INFINITY;
    let mut normal = Vec3::ZERO;
    for axis in 0..3 {
        if delta[axis].abs() < EPS {
            if origin[axis] <= bounds.min[axis] + EPS || origin[axis] >= bounds.max[axis] - EPS {
                return None;
            }
            continue;
        }
        let a = (bounds.min[axis] - origin[axis]) / delta[axis];
        let b = (bounds.max[axis] - origin[axis]) / delta[axis];
        let near = a.min(b);
        if near > entry {
            entry = near;
            normal = Vec3::ZERO;
            normal[axis] = -delta[axis].signum();
        }
        exit = exit.min(a.max(b));
        if entry > exit {
            return None;
        }
    }
    if !(-EPS..=1.0).contains(&entry) || exit < 0.0 || entry >= exit {
        None
    } else {
        Some(Hit {
            fraction: entry.max(0.0),
            normal,
        })
    }
}

#[derive(Debug)]
pub struct MoveResult {
    pub feet: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
    pub contacts: u32,
}

pub fn body_bounds(feet: Vec3, height: f32) -> Aabb {
    Aabb::from_center_size(feet + Vec3::Y * height * 0.5, Vec3::new(0.54, height, 0.54))
}

/// Continuous collision, then slide along contact planes. Handles thin walls and
/// high speed without discrete overlap/tunnelling. No slopes or dynamic bodies.
pub fn move_and_slide(
    feet: Vec3,
    height: f32,
    mut velocity: Vec3,
    dt: f32,
    solids: &[Aabb],
) -> MoveResult {
    let half = Vec3::new(0.27, height * 0.5, 0.27);
    let mut center = feet + Vec3::Y * half.y;
    let mut remaining = velocity * dt;
    let mut grounded = false;
    let mut contacts = 0;
    for _ in 0..8 {
        if remaining.length_squared() < EPS * EPS {
            break;
        }
        let mut closest: Option<Hit> = None;
        for solid in solids {
            if let Some(hit) = sweep_point(center, remaining, solid.expanded(half))
                && closest.is_none_or(|old| hit.fraction < old.fraction)
            {
                closest = Some(hit);
            }
        }
        let Some(hit) = closest else {
            center += remaining;
            break;
        };
        let travel = (hit.fraction - SKIN / remaining.length()).max(0.0);
        center += remaining * travel;
        remaining *= 1.0 - hit.fraction;
        let into = remaining.dot(hit.normal).min(0.0);
        remaining -= hit.normal * into;
        let speed_into = velocity.dot(hit.normal).min(0.0);
        velocity -= hit.normal * speed_into;
        grounded |= hit.normal.y > 0.5;
        contacts += 1;
    }
    MoveResult {
        feet: center - Vec3::Y * half.y,
        velocity,
        grounded,
        contacts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> Aabb {
        Aabb::from_center_size(Vec3::new(0.0, -0.5, 0.0), Vec3::new(100.0, 1.0, 100.0))
    }

    #[test]
    fn fast_body_cannot_tunnel_through_thin_wall() {
        let wall = Aabb::from_center_size(Vec3::new(2.0, 2.0, 0.0), Vec3::new(0.05, 4.0, 10.0));
        let r = move_and_slide(Vec3::ZERO, 1.78, Vec3::X * 400.0, 0.1, &[wall]);
        assert!(r.feet.x > 1.69 && r.feet.x < 1.706, "{:?}", r.feet);
        assert_eq!(r.velocity.x, 0.0);
    }

    #[test]
    fn diagonal_motion_slides_along_wall() {
        let wall = Aabb::from_center_size(Vec3::new(2.0, 2.0, 0.0), Vec3::new(0.2, 4.0, 30.0));
        let r = move_and_slide(
            Vec3::ZERO,
            1.78,
            Vec3::new(8.0, -2.0, -5.0),
            1.0,
            &[wall, floor()],
        );
        assert!(r.feet.x < 1.631);
        assert!(r.feet.z < -4.9);
        assert!(r.feet.y >= -0.001 && r.grounded);
    }

    #[test]
    fn corner_stops_both_horizontal_axes() {
        let walls = [
            Aabb::from_center_size(Vec3::new(2.0, 2.0, 0.0), Vec3::new(0.2, 4.0, 20.0)),
            Aabb::from_center_size(Vec3::new(0.0, 2.0, 2.0), Vec3::new(20.0, 4.0, 0.2)),
        ];
        let r = move_and_slide(Vec3::ZERO, 1.78, Vec3::new(12.0, 0.0, 12.0), 1.0, &walls);
        assert!(r.feet.x < 1.631 && r.feet.z < 1.631);
        assert_eq!(r.velocity, Vec3::ZERO);
    }

    #[test]
    fn floor_does_not_block_parallel_motion() {
        let r = move_and_slide(Vec3::ZERO, 1.78, Vec3::X * 3.0, 1.0, &[floor()]);
        assert!((r.feet.x - 3.0).abs() < 0.001);
    }

    #[test]
    fn wall_does_not_block_retreat() {
        let wall = Aabb::from_center_size(Vec3::new(2.0, 2.0, 0.0), Vec3::new(0.2, 4.0, 20.0));
        let r = move_and_slide(Vec3::new(1.63, 0.0, 0.0), 1.78, -Vec3::X, 1.0, &[wall]);
        assert!(r.feet.x < 0.64);
    }

    #[test]
    fn ceiling_stops_upward_velocity() {
        let ceiling = Aabb::from_center_size(Vec3::new(0.0, 2.2, 0.0), Vec3::new(5.0, 0.2, 5.0));
        let r = move_and_slide(Vec3::ZERO, 1.78, Vec3::Y * 20.0, 0.1, &[ceiling]);
        assert!(r.feet.y < 0.321 && r.velocity.y == 0.0);
    }

    #[test]
    fn ray_is_occluded_and_range_limited() {
        let wall = Aabb::from_center_size(Vec3::new(0.0, 1.0, -3.0), Vec3::new(3.0, 3.0, 0.2));
        assert!(wall.ray(Vec3::Y, -Vec3::Z, 2.0).is_none());
        assert!((wall.ray(Vec3::Y, -Vec3::Z, 5.0).unwrap() - 2.9).abs() < 0.001);
        assert!(wall.ray(Vec3::Y, Vec3::X, 5.0).is_none());
    }
}
