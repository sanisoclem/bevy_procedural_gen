use bevy::ecs::lazy_static::lazy_static;
use bevy::math::Mat2;
use bevy::prelude::*;
use std::ops::Add;

lazy_static! {
    static ref HEX2SPACE: Mat2 =
        Mat2::from_cols_array(&[3.0f32.sqrt(), 0.0, 3.0f32.sqrt() / 2.0, 3.0 / 2.0]);
    static ref SPACE2HEX: Mat2 =
        Mat2::from_cols_array(&[3.0f32.sqrt() / 3.0, 0.0, -1.0 / 3.0, 2.0 / 3.0]);
}

pub trait HexLayout<HexCoord, SpaceCoord> {
    type HexCoordIterator: Iterator<Item = HexCoord>;

    //fn distance_step(&self, h1: HexCoord, h2: HexCoord) -> i32;
    // fn distance(&self, h1: HexCoord, h2: HexCoord) ->  f32;

    fn hex_to_space(&self, hex: HexCoord) -> SpaceCoord;
    fn space_to_hex(&self, space: SpaceCoord) -> HexCoord;
    fn get_neighbors(&self, hex: HexCoord, max_distance: i32) -> Self::HexCoordIterator;
    fn get_ring(&self, center: HexCoord, distance: i32) -> Self::HexCoordIterator;
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct CubeHexCoord(pub i32, pub i32, pub i32);
impl CubeHexCoord {
    pub fn from_axis_coord(q: i32, r: i32) -> Self {
        CubeHexCoord(q, r, -(q + r))
    }
    pub fn distance_step(&self, b: &CubeHexCoord) -> i32 {
        (i32::abs(self.0 - b.0) + i32::abs(self.1 - b.1) + i32::abs(self.2 - b.2)) / 2
    }
}
impl Into<Vec2> for CubeHexCoord {
    fn into(self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }
}
impl Add for CubeHexCoord {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + self.2)
    }
}

pub struct CubeHexLayout {
    pub space_origin: CubeHexCoord,
    pub size: f32,
}
impl CubeHexLayout {
    pub fn hex_coord_from_fractional_coord(&self, frac: Vec2) -> CubeHexCoord {
        let x = frac.x();
        let y = frac.y();
        let z = -(frac.x() + frac.y());
        let mut rx = x.round();
        let mut ry = y.round();
        let mut rz = z.round();

        let x_diff = (rx - x).abs();
        let y_diff = (ry - y).abs();
        let z_diff = (rz - z).abs();

        if x_diff > y_diff && x_diff > z_diff {
            rx = -ry - rz;
        } else if y_diff > z_diff {
            ry = -rx - rz;
        } else {
            rz = -rx - ry;
        }

        CubeHexCoord(rx as i32, ry as i32, rz as i32)
    }
}
impl Default for CubeHexLayout {
    fn default() -> Self {
        CubeHexLayout {
            space_origin: CubeHexCoord(0, 0, 0),
            size: 30.0,
        }
    }
}

impl HexLayout<CubeHexCoord, Vec2> for CubeHexLayout {
    type HexCoordIterator = Box<dyn Iterator<Item = CubeHexCoord>>;

    fn hex_to_space(&self, hex: CubeHexCoord) -> Vec2 {
        HEX2SPACE.mul_vec2((self.space_origin + hex).into()) * self.size
    }

    fn space_to_hex(&self, space: Vec2) -> CubeHexCoord {
        let frac = SPACE2HEX.mul_vec2(space) / self.size;
        self.space_origin + self.hex_coord_from_fractional_coord(frac)
    }

    fn get_neighbors(&self, hex: CubeHexCoord, max_distance: i32) -> Self::HexCoordIterator {
        Box::new((1..=max_distance).flat_map(move |ring| {
            (0..=ring).flat_map(move |i| {
                let indexes = [i, ring - i, -ring];
                // rotate 6 times
                (0..6).map(move |rot| {
                    let m = if rot % 2 == 1 { -1 } else { 1 };
                    let xi = (0 + rot) % 3;
                    let yi = (1 + rot) % 3;
                    hex + CubeHexCoord::from_axis_coord(indexes[xi] * m, indexes[yi] * m)
                })
            })
        }))
    }

    fn get_ring(&self, center: CubeHexCoord, distance: i32) -> Self::HexCoordIterator {
        Box::new((0..=distance).flat_map(move |i| {
            let indexes = [i, distance - i, -distance];
            // rotate 6 times
            (0..6).map(move |rot| {
                let m = if rot % 2 == 1 { -1 } else { 1 };
                let xi = (0 + rot) % 3;
                let yi = (1 + rot) % 3;
                center + CubeHexCoord::from_axis_coord(indexes[xi] * m, indexes[yi] * m)
            })
        }))
    }
}
