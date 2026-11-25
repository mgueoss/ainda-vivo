use ainda_vivo_core::collision::Aabb;
use bevy::prelude::*;
use std::{cmp::Reverse, collections::BinaryHeap};

const CELL: f32 = 0.8;
pub const HUMAN: Vec3 = Vec3::new(0.50, 1.12, 0.50);
pub const LOW: Vec3 = Vec3::new(0.68, 0.65, 1.24);
const LOW_GRID: Vec3 = Vec3::new(LOW.x, LOW.y, LOW.x);
const TURN_CLEARANCE: Vec3 = Vec3::new(1.42, LOW.y, 1.42);
const EDGES: [IVec2; 4] = [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y];
const NODE_BUDGET: usize = 3_200;

#[derive(Default)]
pub(super) struct RouteScratch {
    score: Vec<u16>,
    parent: Vec<usize>,
    queue: BinaryHeap<Reverse<(u16, u16, usize)>>,
    pub(super) expanded: usize,
    pub(super) budget_hits: u32,
}

impl RouteScratch {
    pub(super) fn bytes(&self) -> usize {
        self.score.capacity() * size_of::<u16>()
            + self.parent.capacity() * size_of::<usize>()
            + self.queue.capacity() * size_of::<Reverse<(u16, u16, usize)>>()
    }
}

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
    edges: [Vec<u8>; 3],
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
            edges: std::array::from_fn(|_| vec![0; (size.x * size.y) as usize]),
            origin,
            size,
            solids,
            gate,
        };
        for i in 0..nav.room.len() {
            nav.room[i] = nav.sector(nav.center(i));
            let edge = if nav.room[i].is_some() { 15 } else { 0 };
            nav.edges[0][i] = edge;
            nav.edges[1][i] = edge;
            nav.edges[2][i] = edge;
        }
        // Rasteriza apenas os quadrados atingidos por cada obstáculo expandido.
        // Inclui as arestas, mesmo quando a parede é mais fina que uma célula.
        for (profile, &half) in [HUMAN, LOW_GRID, TURN_CLEARANCE].iter().enumerate() {
            for &solid in &nav.solids {
                let bounds = hull(solid, half);
                let y = half.y + 0.022;
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
                                nav.edges[profile][i] &= !(1 << e);
                            }
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

    pub(super) fn exits(
        &self,
        sector: usize,
        closed: bool,
    ) -> impl Iterator<Item = (usize, Vec2)> + '_ {
        self.portals.iter().filter_map(move |&portal| {
            let [a, b] = portal.rooms;
            if self.portal_gate(portal, closed) {
                return None;
            }
            let other = if a == sector {
                b
            } else if b == sector {
                a
            } else {
                return None;
            };
            let zone = self.zones[other];
            let inward = ((zone.min + zone.max) * 0.5 - portal.at).normalize_or_zero();
            Some((other, portal.at + inward * 1.15))
        })
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

    fn link(&self, a: usize, b: usize, segment: (Vec2, Vec2), closed: bool, half: Vec3) -> bool {
        a == b
            || self.portals.iter().any(|&p| {
                let (from, to) = segment;
                let perpendicular = if from.x != to.x {
                    (p.at.y - from.y).abs()
                } else {
                    (p.at.x - from.x).abs()
                };
                p.rooms.contains(&a)
                    && p.rooms.contains(&b)
                    && p.height >= half.y * 2.0
                    && p.width > half.x * 2.0
                    && !self.portal_gate(p, closed)
                    && point((from + to) * 0.5 - p.at).length() <= p.width * 0.5
                    && perpendicular <= p.width * 0.5 - half.x + 0.02
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

    fn snap(&self, at: Vec2, closed: bool, profile: usize, half: Vec3) -> Option<usize> {
        let c = self.coord(at);
        let sector = self.sector(at)?;
        (-1..=1)
            .flat_map(|z| (-1..=1).map(move |x| c + IVec2::new(x, z)))
            .filter_map(|cell| self.index(cell))
            .filter(|&i| {
                self.room[i] == Some(sector)
                    && self.edges[profile][i] != 0
                    && self.clear_body(at, self.center(i), half, closed)
            })
            .min_by(|&a, &b| {
                self.center(a)
                    .distance_squared(at)
                    .total_cmp(&self.center(b).distance_squared(at))
            })
    }

    pub fn route(&self, from: Vec2, to: Vec2, closed: bool, path: &mut Vec<Vec2>) -> bool {
        self.plan(from, to, closed, false, path, &mut RouteScratch::default())
    }

    pub(super) fn plan(
        &self,
        from: Vec2,
        to: Vec2,
        closed: bool,
        low: bool,
        path: &mut Vec<Vec2>,
        scratch: &mut RouteScratch,
    ) -> bool {
        path.clear();
        let profile = usize::from(low);
        let half = if low { LOW_GRID } else { HUMAN };
        let edges = &self.edges[profile];
        let (Some(start), Some(end)) = (
            self.snap(from, closed, profile, half),
            self.snap(to, closed, profile, half),
        ) else {
            return false;
        };
        let RouteScratch {
            score,
            parent,
            queue,
            expanded,
            budget_hits,
        } = scratch;
        score.resize(edges.len(), u16::MAX);
        score.fill(u16::MAX);
        parent.resize(edges.len(), start);
        queue.clear();
        let mut remaining = NODE_BUDGET;
        score[start] = 0;
        queue.push(Reverse((0, 0, start)));
        let goal = self.cell(end);
        while let Some(Reverse((_, cost, i))) = queue.pop() {
            if cost != score[i] {
                continue;
            }
            if remaining == 0 {
                *budget_hits += 1;
                break;
            }
            remaining -= 1;
            *expanded = expanded.saturating_add(1);
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
                if edges[i] & (1 << edge) == 0 {
                    continue;
                }
                let next = cell + direction;
                let Some(j) = self.index(next) else { continue };
                let Some(room) = self.room[j] else { continue };
                let b = self.center(j);
                let move_cost = if low && self.edges[2][i] & (1 << edge) == 0 {
                    5
                } else {
                    1
                };
                let distance = cost + move_cost;
                if distance >= score[j]
                    || !self.link(self.room[i].unwrap(), room, (a, b), closed, half)
                    || closed
                        && self.gate.is_some_and(|g| {
                            hit(
                                hull(g, half),
                                point(a).with_y(half.y + 0.022),
                                point(b).with_y(half.y + 0.022),
                            )
                        })
                {
                    continue;
                }
                score[j] = distance;
                parent[j] = i;
                let delta = (next - goal).abs();
                queue.push(Reverse((
                    distance + (delta.x + delta.y) as u16,
                    distance,
                    j,
                )));
            }
        }
        false
    }
}
