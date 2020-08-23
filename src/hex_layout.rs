use bevy::{ecs::lazy_static::lazy_static, math::Mat2, prelude::*};
use std::{hash::Hash, ops::Add};

lazy_static! {
    static ref HEX2SPACE: Mat2 =
        Mat2::from_cols_array(&[3.0f32.sqrt(), 0.0, 3.0f32.sqrt() / 2.0, 3.0 / 2.0]);
    static ref SPACE2HEX: Mat2 =
        Mat2::from_cols_array(&[3.0f32.sqrt() / 3.0, 0.0, -1.0 / 3.0, 2.0 / 3.0]);
}

pub trait TileId: Eq + Hash {}
pub trait ChunkId: Eq + Hash {}

pub trait Layout {
    type TTileId: TileId;
    type TChunkId: ChunkId;
    type TChunkIdIterator: Iterator<Item = Self::TChunkId>;

    fn get_placeholder_mesh(&self) -> Mesh;
    fn get_chunk_neighbors(&self, chunk: &Self::TChunkId, distance: u16) -> Self::TChunkIdIterator;

    fn chunk_to_space(&self, chunk: &Self::TChunkId) -> Vec3;
    fn tile_to_chunk(&self, tile: &Self::TTileId) -> Self::TChunkId;
    fn tile_to_space(&self, tile: &Self::TTileId) -> Translation;
    fn space_to_tile(&self, space: Vec3) -> Self::TTileId;
    fn space_to_chunk(&self, space: Vec3) -> Self::TChunkId;
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
impl ChunkId for CubeHexCoord { }
impl TileId for CubeHexCoord { }

pub struct CubeHexLayout {
    pub space_origin: CubeHexCoord,
    pub tile_radius: f32,
    pub chunk_radius: i32,
}
impl CubeHexLayout {
    pub fn get_chunk_radius(&self) -> f32 {
        (self.chunk_radius * 2 * self.tile_radius) + self.tile_radius
    }
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
            tile_radius: 10.0,
            chunk_radius: 10,
        }
    }
}
impl Layout for CubeHexLayout {
    type TChunkId = CubeHexCoord;
    type TChunkIdIterator = Box<dyn Iterator<Item = CubeHexCoord>>;

    fn get_placeholder_mesh(&self) -> Mesh {
        crate::mesh::mesh_hex_outline(Vec3::default(), Vec3::unit_y(), Vec3::unit_z(), self.get_chunk_radius())
    }

    fn get_chunk_neighbors(&self, chunk: &Self::TChunkId, distance: u16) -> Self::TChunkIdIterator {
        let radius = self.chunk_radius;
        let inc = 2 * radius + 1;

        Box::new((1..=distance).flat_map(move |ring| {
            let anchor = [-ring * inc, radius * ring + ring, radius * ring];

            (0..ring).flat_map(move |i| {
                let indexes = [anchor[0] + (i * radius), anchor[1] - (i * inc), anchor[2] + (i * inc) - (i * radius)];

                // rotate 6 times
                (0..6).map(move |rot| {
                    let m = if rot % 2 == 1 { -1 } else { 1 };
                    let xi = (0 + rot) % 3;
                    let yi = (1 + rot) % 3;
                    chunk + CubeHexCoord::from_axis_coord(indexes[xi] * m, indexes[yi] * m)
                })
            })
        }))
    }

    fn chunk_to_space(&self, chunk: &Self::TChunkId) -> Vec3 {
        HEX2SPACE.mul_vec2((self.space_origin + chunk).into()) * self.hex_size
    }
    fn tile_to_chunk(&self, tile: &Self::TTileId) -> Self::TChunkId {
        let radius = self.chunk_radius;
        let offset_base = 3 * radius + 1;
        let period = (3 * radius * radius) + offset_base;
        let x_offset_based_on_z = tile.2 * offset_base;
        let x_transposed_axis = tile.0 - x_offset_based_on_z; // chunk center if multiple of period
        let xperiods = x_transposed_axis / period;
        let x_lower = xperiods + x_offset_based_on_z;
        let x_upper = x_lower + period;
        todo!()
    }
    fn tile_to_space(&self, tile: &Self::TTileId) -> Translation {
        todo!()
    }
    fn space_to_tile(&self, space: Vec3) -> Self::TTileId {
        todo!()
    }
    fn space_to_chunk(&self, space: Vec3) -> Self::TChunkId {
        todo!()
    }
}
impl HexLayout<CubeHexCoord, Vec2> for CubeHexLayout {
    type HexCoordIterator = Box<dyn Iterator<Item = CubeHexCoord>>;

    fn hex_to_space(&self, hex: CubeHexCoord) -> Vec2 {
        HEX2SPACE.mul_vec2((self.space_origin + hex).into()) * self.hex_size
    }

    fn space_to_hex(&self, space: Vec2) -> CubeHexCoord {
        let frac = SPACE2HEX.mul_vec2(space) / self.hex_size;
        self.space_origin + self.hex_coord_from_fractional_coord(frac)
    }

    fn get_neighbors(&self, hex: CubeHexCoord, max_distance: i32) -> Self::HexCoordIterator {
        Box::new((1..=max_distance).flat_map(move |ring| {
            (1..=ring).flat_map(move |i| {
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

    fn get_chunk_neighbors(&self, chunk_center: CubeHexCoord, max_distance: i32) -> Self::HexCoordIterator {
        let radius = self.chunk_radius;
        Box::new((1..=max_distance).flat_map(move |ring| {
            let anchor = [-ring * (2 * radius + 1), radius * ring + ring, radius * ring];

            (1..=ring).flat_map(move |i| {
                let indexes = [anchor[0], anchor[1] - (i * (2 * radius + 1)) , anchor[2] - (radius * i)];

                // rotate 6 times
                (0..6).map(move |rot| {
                    let m = if rot % 2 == 1 { -1 } else { 1 };
                    let xi = (0 + rot) % 3;
                    let yi = (1 + rot) % 3;
                    chunk_center + CubeHexCoord::from_axis_coord(indexes[xi] * m, indexes[yi] * m)
                })
            })
        }))
    }

    fn get_ring(&self, center: CubeHexCoord, distance: i32) -> Self::HexCoordIterator {
        Box::new((1..=distance).flat_map(move |i| {
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
