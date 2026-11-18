use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sector {
    Access,
    Dispatch,
    Dock,
    Stores,
    Records,
    Lockers,
    Pumps,
    Filters,
    Generators,
    Tanks,
    Electrical,
    Inspection,
    Control,
    Basin,
    Exit,
}

#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub sector: Sector,
    pub title: &'static str,
    pub min: Vec2,
    pub max: Vec2,
    pub height: f32,
}

impl Room {
    pub fn contains(self, position: Vec2) -> bool {
        position.cmpge(self.min).all() && position.cmple(self.max).all()
    }
}

const fn room(
    sector: Sector,
    title: &'static str,
    x0: f32,
    z0: f32,
    x1: f32,
    z1: f32,
    height: f32,
) -> Room {
    Room {
        sector,
        title,
        min: Vec2::new(x0, z0),
        max: Vec2::new(x1, z1),
        height,
    }
}

use Sector::*;
pub const ROOMS: &[Room] = &[
    room(Access, "ACESSO", -8.0, -12.0, 8.0, 10.0, 3.0),
    room(Dispatch, "TRIAGEM", -10.0, -26.0, 10.0, -12.0, 3.8),
    room(Dock, "CARGA", -30.0, -12.0, -8.0, 10.0, 5.4),
    room(Stores, "DEPÓSITO", -29.0, -26.0, -10.0, -12.0, 4.1),
    room(Records, "ARQUIVO", 8.0, -12.0, 32.0, 10.0, 3.2),
    room(Lockers, "VESTIÁRIO", 10.0, -26.0, 29.0, -12.0, 3.4),
    room(Pumps, "BOMBAS", -14.0, -58.0, 14.0, -26.0, 6.2),
    room(Filters, "FILTROS", -31.0, -47.0, -14.0, -26.0, 4.2),
    room(Generators, "GERADORES", 14.0, -47.0, 31.0, -26.0, 4.8),
    room(Tanks, "TANQUES", -33.0, -66.0, -14.0, -47.0, 5.1),
    room(Electrical, "ELÉTRICA", 14.0, -66.0, 35.0, -47.0, 4.0),
    room(Inspection, "INSPEÇÃO", -33.0, -82.0, -12.0, -66.0, 3.5),
    room(Control, "CONTROLE", 12.0, -86.0, 35.0, -66.0, 3.7),
    room(Basin, "DECANTAÇÃO", -12.0, -86.0, 12.0, -58.0, 5.2),
    room(Exit, "SAÍDA", -3.25, -99.0, 3.25, -86.0, 3.6),
];

#[derive(Clone, Copy, Debug)]
pub struct Portal {
    pub rooms: [Sector; 2],
    pub at: Vec2,
    pub width: f32,
    pub height: f32,
}

impl Portal {
    pub fn neighbour(self, sector: Sector) -> Option<Sector> {
        self.rooms
            .iter()
            .position(|&candidate| candidate == sector)
            .map(|index| self.rooms[1 - index])
    }
}

const fn doorway(a: Sector, b: Sector, x: f32, z: f32, width: f32) -> Portal {
    Portal {
        rooms: [a, b],
        at: Vec2::new(x, z),
        width,
        height: 2.68,
    }
}

pub const PORTALS: &[Portal] = &[
    doorway(Access, Dispatch, 0.0, -12.0, 3.18),
    doorway(Access, Dock, -8.0, 4.0, 2.4),
    doorway(Access, Records, 8.0, -4.0, 2.38),
    doorway(Dispatch, Stores, -10.0, -19.0, 2.52),
    doorway(Dispatch, Lockers, 10.0, -20.0, 2.52),
    doorway(Dock, Stores, -20.0, -12.0, 3.3),
    doorway(Records, Lockers, 21.0, -12.0, 2.52),
    Portal {
        height: 3.35,
        ..doorway(Dispatch, Pumps, 0.0, -26.0, 6.0)
    },
    doorway(Stores, Filters, -21.0, -26.0, 2.55),
    doorway(Lockers, Generators, 21.0, -26.0, 2.55),
    doorway(Filters, Pumps, -14.0, -36.5, 3.1),
    doorway(Generators, Pumps, 14.0, -36.5, 3.1),
    doorway(Filters, Tanks, -24.0, -47.0, 3.05),
    doorway(Generators, Electrical, 24.0, -47.0, 3.05),
    doorway(Tanks, Pumps, -14.0, -52.5, 3.1),
    doorway(Electrical, Pumps, 14.0, -52.5, 3.1),
    doorway(Tanks, Inspection, -23.5, -66.0, 2.75),
    doorway(Electrical, Control, 24.0, -66.0, 2.75),
    Portal {
        height: 3.25,
        ..doorway(Pumps, Basin, 0.0, -58.0, 5.6)
    },
    doorway(Inspection, Basin, -12.0, -74.0, 2.8),
    doorway(Control, Basin, 12.0, -75.0, 2.8),
    Portal {
        height: 2.94,
        ..doorway(Basin, Exit, 0.0, -86.0, 4.4)
    },
];

pub const WATER_MIN: Vec2 = Vec2::new(-4.4, -79.1);
pub const WATER_MAX: Vec2 = Vec2::new(4.4, -64.8);
pub const PANEL: Vec3 = Vec3::new(34.59, 1.32, -58.25);
pub const PUMP_02: Vec3 = Vec3::new(8.45, 0.0, -43.55);
pub const PUMP_02_PANEL: Vec3 = Vec3::new(PUMP_02.x - 1.87, 1.09, PUMP_02.z + 0.83);
pub const EXIT_GATE: Vec3 = Vec3::new(0.0, 1.48, -86.13);
pub const EXIT_THRESHOLD: f32 = -95.0;

pub fn get(id: Sector) -> Room {
    *ROOMS
        .iter()
        .find(|room| room.sector == id)
        .expect("sector must have a room")
}

#[derive(Clone, Copy, Debug)]
pub enum Side {
    West,
    East,
    North,
    South,
}

impl Side {
    pub const ALL: [Self; 4] = [Self::West, Self::East, Self::North, Self::South];

    pub fn inward(self) -> Vec3 {
        match self {
            Self::West => Vec3::X,
            Self::East => Vec3::NEG_X,
            Self::North => Vec3::Z,
            Self::South => Vec3::NEG_Z,
        }
    }

    pub fn rotation(self) -> f32 {
        use std::f32::consts::{FRAC_PI_2, PI};
        match self {
            Self::West => FRAC_PI_2,
            Self::East => -FRAC_PI_2,
            Self::North => 0.0,
            Self::South => PI,
        }
    }

    pub fn along(self, point: Vec2) -> f32 {
        match self {
            Self::West | Self::East => point.y,
            Self::North | Self::South => point.x,
        }
    }

    pub fn plane(self, room: Room) -> f32 {
        match self {
            Self::West => room.min.x,
            Self::East => room.max.x,
            Self::North => room.min.y,
            Self::South => room.max.y,
        }
    }

    pub fn at(self, room: Room, along: f32, y: f32) -> Vec3 {
        let edge = self.plane(room);
        match self {
            Self::West | Self::East => Vec3::new(edge, y, along),
            Self::North | Self::South => Vec3::new(along, y, edge),
        }
    }

    pub fn size(self, width: f32, height: f32, depth: f32) -> Vec3 {
        match self {
            Self::West | Self::East => Vec3::new(depth, height, width),
            Self::North | Self::South => Vec3::new(width, height, depth),
        }
    }

    pub fn contains(self, room: Room, portal: Portal) -> bool {
        let at = self.along(portal.at);
        let position = self.at(room, at, 0.0);
        let radius = portal.width * 0.5;
        position.xz() == portal.at
            && at - radius >= self.along(room.min)
            && at + radius <= self.along(room.max)
            && portal.height <= room.height
    }
}
