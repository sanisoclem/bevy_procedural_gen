use bevy::{ecs::lazy_static::lazy_static, math::Mat2, prelude::*};
use std::{hash::Hash, ops::Add, collections::HashMap};
use crate::terrain::{ChunkId,TileId,Layout};

lazy_static! {
    static ref HEX2SPACE: Mat2 =
        Mat2::from_cols_array(&[3.0f32.sqrt(), 0.0, 3.0f32.sqrt() / 2.0, 3.0 / 2.0]);
    static ref SPACE2HEX: Mat2 =
        Mat2::from_cols_array(&[3.0f32.sqrt() / 3.0, 0.0, -1.0 / 3.0, 2.0 / 3.0]);
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct CubeHexCoord(pub i32, pub i32, pub i32);
impl CubeHexCoord {
    pub fn from_axis_coord(q: i32, r: i32) -> Self {
        CubeHexCoord(q, r, -(q + r))
    }
    pub fn from_xz(x: i32, z: i32) -> Self {
        CubeHexCoord(x, -(x + z), z)
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
    tile_radius: f32,
    chunk_radius_step: u32,
    chunk_lookup: HashMap<u32, CubeHexCoord>,
    period: u32,
}
impl CubeHexLayout {
    #[inline]
    pub fn chunk_radius(&self) -> f32 {
        (self.chunk_radius_step * 2 * self.tile_radius) + self.tile_radius
    }
    #[inline]
    pub fn chunk_radius_step(&self) -> u32 { self.chunk_radius_step }
    #[inline]
    pub fn tile_radius(&self) -> f32 { self.tile_radius }

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
    pub fn new(origin: CubeHexCoord, tile_radius: f32, chunk_radius: u32) -> Self {
        // TODO: clean up
        let chunk_lookup = HashMap::new();
        let radius = chunk_radius;
        let offset_base = 3 * radius + 1;
        let period = (3 * radius * radius) + offset_base;
        let half_period = (period - 1) / 2; // period is always odd
        let edge_length = radius + 1;

        let mut bottom = edge_length + radius;
        let mut current_slice = radius * 2;
        for phase in ([-1, 1]).into_iter() {
            for offset in 0..=half_period {
                let key = if phase == 1 { offset } else { period - offset };

                if offset <= radius {
                    chunk_lookup.insert(key, CubeHexCoord::from_xz(offset * phase, 0));
                }
                else if offset <= edge_length + radius {
                    chunk_lookup.insert(key, CubeHexCoord::from_xz((offset - edge_length - radius) * phase, radius * phase));
                } else {
                    let inner_offset = offset - bottom;
                    let inner_phase = if current_slice % 2 == 0  { 1 } else { -1 };

                    chunk_lookup.insert(key, CubeHexCoord::from_xz((offset - edge_length - radius) * phase, radius * phase));

                    if inner_offset + 1 > current_slice {
                        bottom += current_slice;
                        current_slice -= 1;
                    }
                }
            }
        }

        // offset: number of hexes to the nearest center to the postive x axis
        (0..=half_period).flat_map(|offset| ([-1, 1]).iter().map(|phase| {
            let key = if phase == 1 { offset } else { period - offset };
            if offset <= radius {
                CubeHexCoord::from_xz(offset * phase, 0)
            }
            else if offset <= edge_length + radius {
                CubeHexCoord::from_xz((offset - edge_length - radius) * phase, radius * phase)
            } else {
                let mut bottom = edge_length + radius;


                todo!()
            }
        }));

        CubeHexLayout {
            space_origin: origin,
            tile_radius: tile_radius,
            chunk_radius_step: chunk_radius,
            chunk_lookup: chunk_lookup,
            period: period
        }
    }
}
impl Default for CubeHexLayout {
    fn default() -> Self {
        CubeHexLayout::new(CubeHexCoord::default(), 10.0, 10)
    }
}
impl Layout for CubeHexLayout {
    type TChunkId = CubeHexCoord;
    type TChunkIdIterator = Box<dyn Iterator<Item = CubeHexCoord>>;

    fn get_placeholder_mesh(&self) -> Mesh {
        crate::mesh::mesh_hex_outline(Vec3::default(), Vec3::unit_y(), Vec3::unit_z(), self.chunk_radius())
    }

    fn get_chunk_neighbors(&self, chunk: &Self::TChunkId, distance: u16) -> Self::TChunkIdIterator {
        let radius = self.chunk_radius_step;
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
        HEX2SPACE.mul_vec2((self.space_origin + chunk).into()) * self.tile_radius
    }
    fn tile_to_chunk(&self, tile: &Self::TTileId) -> Self::TChunkId {
        let radius = self.chunk_radius_step;
        let offset_base = 3 * radius + 1;
        let x_offset_based_on_z = tile.2 * offset_base;
        let x_transposed_axis = tile.0 - x_offset_based_on_z; // chunk center if multiple of period
        let x_closest = x_transposed_axis % self.period;
        let x_upper = if x_closest < 0 { x_closest + self.period } else { x_closest };

        tile + self.chunk_lookup[x_upper];
    }
    fn tile_to_space(&self, tile: &Self::TTileId) -> Translation {
        HEX2SPACE.mul_vec2((self.space_origin + tile).into()) * self.tile_radius
    }
    fn space_to_tile(&self, space: Vec3) -> Self::TTileId {
        let frac = SPACE2HEX.mul_vec2(space) / self.tile_radius;
        self.space_origin + self.hex_coord_from_fractional_coord(frac)
    }
    fn space_to_chunk(&self, space: Vec3) -> Self::TChunkId {
        self.tile_to_chunk(self.space_to_tile(space))
    }
}