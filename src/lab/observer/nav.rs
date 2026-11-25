use ainda_vivo_core::collision::Aabb;
use bevy::prelude::*;
use std::{cmp::Reverse, collections::BinaryHeap};

const CELL: f32 = 0.8;
pub const HUMAN: Vec3 = Vec3::new(0.50, 1.12, 0.50);
pub const LOW: Vec3 = Vec3::new(0.68, 0.65, 1.24);
const EDGES: [IVec2; 4] = [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y];

#[derive(Clone, Copy)]
pub struct Zone {
    pub min: Vec2,
    pub max: Vec2,
}

impl Zone {
    pub fn contains(self, p: Vec2) -> bool {
        p.cmpge(self.min).all() && p.cmple(self.max).all()
    }
}

#[derive(Clone, Copy)]
pub struct Portal {
    pub rooms: [usize; 2],
    pub at: Vec2,
    pub width: f32,
    pub height: f32,
}

pub struct Nav {
    zones: Vec<Zone>,
    portals: Vec<Portal>,
    room: Vec<Option<usize>>,
    edges: Vec<u8>,
    origin: Vec2,
    size: IVec2,
    pub(crate) solids: Vec<Aabb>,
    pub(crate) gate: Option<Aabb>,
}

pub fn point(p: Vec2) -> Vec3 {
    Vec3::new(p.x, 0.0, p.y)
}

fn hit(bounds: Aabb, from: Vec3, to: Vec3) -> bool {
    let d = to - from;
    bounds
        .ray(from, d.normalize_or_zero(), d.length())
        .is_some()
}

fn hull(bounds: Aabb, half: Vec3) -> Aabb {
    bounds.expanded(half + Vec3::new(0.018, 0.0, 0.018))
}

impl Nav {
    pub fn new(
        zones: Vec<Zone>,
        portals: Vec<Portal>,
        solids: Vec<Aabb>,
        gate: Option<Aabb>,
    ) -> Self {
        let origin = zones
            .iter()
            .fold(Vec2::splat(f32::MAX), |min, z| min.min(z.min));
        let end = zones
            .iter()
            .fold(Vec2::splat(f32::MIN), |max, z| max.max(z.max));
        let size = ((end - origin) / CELL).ceil().as_ivec2();
        let mut nav = Self {
            zones,
            portals,
            room: vec![None; (size.x * size.y) as usize],
            edges: vec![0; (size.x * size.y) as usize],
            origin,
            size,
            solids,
            gate,
        };
        for i in 0..nav.room.len() {
            nav.room[i] = nav.sector(nav.center(i));
            nav.edges[i] = if nav.room[i].is_some() { 15 } else { 0 };
        }
        // Rasteriza apenas os quadrados atingidos por cada obstáculo expandido.
        // Inclui as arestas, mesmo quando a parede é mais fina que uma célula.
        for &solid in &nav.solids {
            let bounds = hull(solid, HUMAN);
            let y = HUMAN.y + 0.022;
            if y < bounds.min.y || y > bounds.max.y {
                continue;
            }
            let from = nav
                .coord(bounds.min.xz() - Vec2::splat(CELL))
                .max(IVec2::ZERO);
            let to = nav
                .coord(bounds.max.xz() + Vec2::splat(CELL))
                .min(size - IVec2::ONE);
            for z in from.y..=to.y {
                for x in from.x..=to.x {
                    let cell = IVec2::new(x, z);
                    let i = nav.index(cell).unwrap();
                    let a = point(nav.center(i)).with_y(y);
                    for (e, d) in EDGES.into_iter().enumerate() {
                        if hit(bounds, a, a + point(d.as_vec2()) * CELL) {
                            nav.edges[i] &= !(1 << e);
                        }
                    }
                }
            }
        }
        nav
    }

    fn coord(&self, p: Vec2) -> IVec2 {
        ((p - self.origin) / CELL).floor().as_ivec2()
    }

    fn index(&self, c: IVec2) -> Option<usize> {
        (c.cmpge(IVec2::ZERO).all() && c.cmplt(self.size).all())
            .then(|| (c.y * self.size.x + c.x) as usize)
    }

    fn cell(&self, i: usize) -> IVec2 {
        IVec2::new(i as i32 % self.size.x, i as i32 / self.size.x)
    }

    fn center(&self, i: usize) -> Vec2 {
        self.origin + (self.cell(i).as_vec2() + Vec2::splat(0.5)) * CELL
    }

    pub fn sector(&self, p: Vec2) -> Option<usize> {
        self.zones.iter().position(|z| z.contains(p))
    }

    pub fn blockers(&self, closed: bool) -> impl Iterator<Item = &Aabb> {
        self.solids
            .iter()
            .chain(self.gate.iter().filter(move |_| closed))
    }

    pub fn clear(&self, from: Vec3, to: Vec3, closed: bool) -> bool {
        !self.blockers(closed).any(|&bounds| hit(bounds, from, to))
    }

    pub fn clear_body(&self, a: Vec2, b: Vec2, half: Vec3, closed: bool) -> bool {
        self.sector(a).is_some()
            && self.sector(b).is_some()
            && !self.blockers(closed).any(|&s| {
                hit(
                    hull(s, half),
                    point(a).with_y(half.y + 0.022),
                    point(b).with_y(half.y + 0.022),
                )
            })
    }

    fn portal_gate(&self, p: Portal, closed: bool) -> bool {
        closed
            && self.gate.is_some_and(|g| {
                let d = (g.min.xz() + g.max.xz()) * 0.5 - p.at;
                d.length_squared() < 1.1
            })
    }

    fn link(&self, a: usize, b: usize, at: Vec2, closed: bool) -> bool {
        a == b
            || self.portals.iter().any(|&p| {
                p.rooms.contains(&a)
                    && p.rooms.contains(&b)
                    && p.height >= HUMAN.y * 2.0
                    && p.width > HUMAN.x * 2.0
                    && !self.portal_gate(p, closed)
                    && p.at.distance(at) <= p.width * 0.5 - HUMAN.x + 0.02
            })
    }

    pub fn acoustic(&self, a: Vec2, b: Vec2, closed: bool) -> f32 {
        let (Some(a), Some(b)) = (self.sector(a), self.sector(b)) else {
            return 0.0;
        };
        if a == b {
            return 1.0;
        }
        let mut reached = 1u32 << a;
        for attenuation in [0.72, 0.49, 0.30] {
            let mut next = reached;
            for &p in &self.portals {
                let [x, y] = p.rooms;
                if !self.portal_gate(p, closed) && reached & ((1 << x) | (1 << y)) != 0 {
                    next |= (1 << x) | (1 << y);
                }
            }
            if next & (1 << b) != 0 {
                return attenuation;
            }
            reached = next;
        }
        0.0
    }

    fn snap(&self, at: Vec2, closed: bool) -> Option<usize> {
        let c = self.coord(at);
        let sector = self.sector(at)?;
        (-1..=1)
            .flat_map(|z| (-1..=1).map(move |x| c + IVec2::new(x, z)))
            .filter_map(|cell| self.index(cell))
            .filter(|&i| {
                self.room[i] == Some(sector)
                    && self.edges[i] != 0
                    && self.clear_body(at, self.center(i), HUMAN, closed)
            })
            .min_by(|&a, &b| {
                self.center(a)
                    .distance_squared(at)
                    .total_cmp(&self.center(b).distance_squared(at))
            })
    }

    pub fn route(&self, from: Vec2, to: Vec2, closed: bool, path: &mut Vec<Vec2>) -> bool {
        path.clear();
        let (Some(start), Some(end)) = (self.snap(from, closed), self.snap(to, closed)) else {
            return false;
        };
        let mut score = vec![u16::MAX; self.edges.len()];
        let mut parent = vec![start; self.edges.len()];
        let mut queue = BinaryHeap::new();
        score[start] = 0;
        queue.push(Reverse((0, 0, start)));
        let goal = self.cell(end);
        while let Some(Reverse((_, cost, i))) = queue.pop() {
            if cost != score[i] {
                continue;
            }
            if i == end {
                path.push(to);
                let mut current = i;
                while current != start {
                    path.push(self.center(current));
                    current = parent[current];
                }
                path.push(self.center(start));
                path.reverse();
                return true;
            }
            let cell = self.cell(i);
            let a = self.center(i);
            for (edge, direction) in EDGES.into_iter().enumerate() {
                if self.edges[i] & (1 << edge) == 0 {
                    continue;
                }
                let next = cell + direction;
                let Some(j) = self.index(next) else { continue };
                let Some(room) = self.room[j] else { continue };
                let b = self.center(j);
                if cost + 1 >= score[j]
                    || !self.link(self.room[i].unwrap(), room, (a + b) * 0.5, closed)
                    || closed
                        && self.gate.is_some_and(|g| {
                            hit(
                                hull(g, HUMAN),
                                point(a).with_y(1.142),
                                point(b).with_y(1.142),
                            )
                        })
                {
                    continue;
                }
                score[j] = cost + 1;
                parent[j] = i;
                let delta = (next - goal).abs();
                queue.push(Reverse((
                    cost + 1 + (delta.x + delta.y) as u16,
                    cost + 1,
                    j,
                )));
            }
        }
        false
    }
}
