use crate::terrain::{ChunkId, Layout, TileId};
use bevy::{ecs::lazy_static::lazy_static, math::Mat2, prelude::*};
use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Add, Sub},
};

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
        Vec2::new(self.0 as f32, self.2 as f32)
    }
}
impl Add for CubeHexCoord {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}
impl Sub for CubeHexCoord {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}
impl ChunkId for CubeHexCoord {}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct ExtrudedCubeHexCoord(pub i32, pub i32, pub i32, pub i32);
impl ExtrudedCubeHexCoord {
    pub fn new(hex: CubeHexCoord, height: i32) -> Self {
        ExtrudedCubeHexCoord(hex.0, hex.1, hex.2, height)
    }

    #[inline]
    pub fn h(&self) -> i32 {
        self.3
    }

    #[inline]
    pub fn get_base(&self) -> CubeHexCoord {
        CubeHexCoord(self.0, self.1, self.2)
    }
}
impl TileId for ExtrudedCubeHexCoord {}

pub struct CubeHexLayout {
    pub space_origin: CubeHexCoord,
    tile_radius: f32,
    tile_extrusion_height: f32,
    chunk_radius_step: i32,
    chunk_lookup: HashMap<i32, CubeHexCoord>,
    period: i32,
}
impl CubeHexLayout {
    #[inline]
    pub fn chunk_radius(&self) -> f32 {
        self.chunk_diameter()/2.0
    }
    #[inline]
    pub fn chunk_diameter(&self) -> f32 {
        self.chunk_diameter_step() as f32 * self.tile_diameter()
    }

    #[inline]
    pub fn chunk_diameter_step(&self) -> i32 {
        (self.chunk_radius_step * 2) + 1
    }

    #[inline]
    pub fn chunk_radius_step(&self) -> i32 {
        self.chunk_radius_step
    }

    #[inline]
    pub fn tile_radius(&self) -> f32 {
        self.tile_radius
    }

    #[inline]
    pub fn tile_diameter(&self) -> f32 {
        self.tile_radius * 2.0
    }

    pub fn hex_coord_from_fractional_coord(&self, frac: Vec2) -> CubeHexCoord {
        let x = frac.x();
        let y = -(frac.x() + frac.y());
        let z = frac.y();

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

    pub fn new(
        origin: CubeHexCoord,
        tile_radius: f32,
        chunk_radius: i32,
        tile_extrusion_height: f32,
    ) -> Self {
        // TODO: clean up
        let mut chunk_lookup = HashMap::new();
        let radius = chunk_radius;
        let offset_base = 3 * radius + 1;
        let period = (3 * radius * radius) + offset_base;
        let half_period = (period - 1) / 2; // period is always odd
        let edge_length = radius + 1;
        //println!("LOOKUPSTART");
        for &phase in ([1, -1]).iter() {
            let mut bottom = edge_length + radius;
            let mut current_slice = radius * 2;
            for offset in 0..=half_period {
                let key = if phase == 1 { offset } else { period - offset };
                let chunk;

                if offset <= radius {
                    chunk = CubeHexCoord::from_xz(offset * phase, 0);
                } else if offset <= edge_length + radius {
                    chunk = CubeHexCoord::from_xz(
                        (offset - edge_length - radius) * phase,
                        radius * phase,
                    );
                } else {
                    let inner_offset = offset - bottom;
                    let inner_phase = if current_slice % 2 == 0 { 1 } else { -1 };
                    let x_offset = if current_slice % 2 == 0 {
                        current_slice - radius
                    } else {
                        radius + 1
                    };

                    chunk = CubeHexCoord::from_xz(
                        (inner_offset - x_offset) * phase,
                        ((radius + radius + 1) - current_slice) * -phase * inner_phase,
                    );

                    if inner_offset + 1 > current_slice {
                        bottom += current_slice;
                        current_slice -= 1;
                    }
                }

                //println!("{:?}", (key, chunk) );
                chunk_lookup.insert(key, chunk);
            }
        }

        CubeHexLayout {
            space_origin: origin,
            tile_radius,
            tile_extrusion_height,
            chunk_radius_step: chunk_radius,
            chunk_lookup,
            period,
        }
    }
}
impl Default for CubeHexLayout {
    fn default() -> Self {
        CubeHexLayout::new(CubeHexCoord::default(), 1.0, 3, 1.0)
    }
}
impl Layout for CubeHexLayout {
    type TChunkId = CubeHexCoord;
    type TChunkIdIterator = Box<dyn Iterator<Item = CubeHexCoord>>;
    type TTileId = ExtrudedCubeHexCoord;

    fn get_placeholder_mesh(&self) -> Mesh {
        crate::mesh::mesh_hex_outline(
            Vec3::default(),
            Vec3::unit_y(),
            Vec3::unit_x(),
            self.chunk_radius(),
        )
    }

    fn get_chunk_neighbors(&self, chunk: Self::TChunkId, distance: i32) -> Self::TChunkIdIterator {
        let radius = self.chunk_radius_step;
        let inc = 2 * radius + 1;

        Box::new((1..=distance).flat_map(move |ring| {
            let anchor = [-ring * inc, radius * ring + ring, radius * ring];

            (0..ring).flat_map(move |i| {
                let indexes = [
                    anchor[0] + (i * radius),
                    anchor[1] - (i * inc),
                    anchor[2] + (i * inc) - (i * radius),
                ];

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

    fn chunk_to_space(&self, chunk: &Self::TChunkId) -> Translation {
        self.tile_to_space(&ExtrudedCubeHexCoord::new(*chunk, 0))
    }

    fn tile_to_chunk(&self, tile: &Self::TTileId) -> Self::TChunkId {
        let radius = self.chunk_radius_step;
        let offset_base = 3 * radius + 1;
        let x_offset_based_on_z = (tile.2 * offset_base) % self.period;
        let x_transposed_axis = x_offset_based_on_z - tile.0; // chunk center if multiple of period
        let x_closest = x_transposed_axis % self.period ;
        let x_upper = if x_closest < 0 {
            x_closest + self.period
        } else {
            x_closest
        };

        //println!("upper for {:?} is {:?}, period: {:?}, radius {:?}, lookup: {:?}", tile, x_upper, self.period,self.chunk_radius_step(), self.chunk_lookup[&x_upper]);
        tile.get_base() + self.chunk_lookup[&x_upper]
    }

    fn tile_to_space(&self, tile: &Self::TTileId) -> Translation {
        let result =
            HEX2SPACE.mul_vec2((tile.get_base() - self.space_origin).into()) * self.tile_radius;
        Translation::new(
            result.x(),
            tile.h() as f32 * self.tile_extrusion_height,
            result.y(),
        )
    }

    fn space_to_tile(&self, space: &Vec3) -> Self::TTileId {
        let frac = SPACE2HEX.mul_vec2(Vec2::new(space.x(), space.z())) / self.tile_radius;
        println!("Frac: {:?} input: {:?}, origin: {:?}", frac, space, self.space_origin);
        let y = (space.y() / self.tile_extrusion_height).floor() as i32;
        let hex = self.hex_coord_from_fractional_coord(frac) + self.space_origin;
        ExtrudedCubeHexCoord::new(hex, y)
    }

    fn space_to_chunk(&self, space: &Vec3) -> Self::TChunkId {
        self.tile_to_chunk(&self.space_to_tile(space))
    }

    fn get_chunk_distance(&self, a: &Self::TChunkId, b: &Self::TChunkId) -> i32 {
        0
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn from_xz_coords_must_total_to_zero(x in -10000i32..10000,
                                        z in -10000i32..10000) {
            let coord = CubeHexCoord::from_xz(x, z);
            prop_assert_eq!(coord.0 + coord.1 + coord.2, 0);
        }
        #[test]
        fn from_axis_coords_must_total_to_zero(q in -10000i32..10000,
                                        r in -10000i32..10000) {
            let coord = CubeHexCoord::from_axis_coord(q, r);
            prop_assert_eq!(coord.0 + coord.1 + coord.2, 0);
        }
    }

    proptest! {
        #[test]
        fn neighbor_should_have_correct_distance_in_space(x1 in -10000i32..=10000, z1 in -10000i32..=10000, radius in 1i32..50, distance in 1i32..10) {
            // TODO: doesnt work for arbitrary starting chunks
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 1.0);
            let chunk = CubeHexCoord::from_xz(0, 0);
            let origin_space = layout.chunk_to_space(&chunk).0;
            let expected_distance_upper_bound = layout.chunk_diameter() * distance as f32;
            for neighbor in layout.get_chunk_neighbors(chunk, distance) {
                let neighbor_space = layout.chunk_to_space(&neighbor).0;
                let result = (neighbor_space - origin_space).abs().length();
                assert!(result < expected_distance_upper_bound, "expected upper bound: {}, actual: {}, pointA: {}, pointB: {}", expected_distance_upper_bound, result, origin_space, neighbor_space);
            }
        }
        #[test]
        fn neighbor_should_be_mutual(x1 in -10000i32..=10000, z1 in -10000i32..=10000, radius in 1i32..50, distance in 1i32..10) {
            // TODO: doesnt work for arbitrary starting chunks
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 1.0);
            let chunk = CubeHexCoord::from_xz(0, 0);
            for neighbor in layout.get_chunk_neighbors(chunk, distance) {
                let ns: Vec<_> = layout.get_chunk_neighbors(neighbor, distance).collect();
                let original: Vec<_> = ns.clone().into_iter().filter(|n| *n == chunk).collect();
                assert_eq!(original.len(), 1);
                assert_eq!(original[0], chunk);
            }
        }

        #[test]
        fn chunk_space_coordinates_should_be_zero_when_at_origin(x1 in -10000i32..=10000, z1 in -10000i32..=10000, radius in 1i32..50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 1.0);
            let coords = layout.chunk_to_space(&layout.space_origin);
            assert_eq!(coords.x(), 0.0);
            assert_eq!(coords.y(), 0.0);
            assert_eq!(coords.z(), 0.0);
        }

        #[test]
        fn tile_space_coordinates_should_be_reversible(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, 1, 1.0);
            let tile = ExtrudedCubeHexCoord::new(CubeHexCoord::from_xz(x2, z2), 0);
            let space_coords = layout.tile_to_space(&tile);
            let result = layout.space_to_tile(&space_coords);
            assert_eq!(result, tile, "Coords: {:?}", space_coords);
        }

        #[test]
        fn chunk_space_coordinates_should_be_reversible(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, 1, 1.0);
            let tile = ExtrudedCubeHexCoord::new(CubeHexCoord::from_xz(x2, z2), 0);
            let chunk = layout.tile_to_chunk(&tile);
            let space_coords = layout.chunk_to_space(&chunk);
            let result = layout.space_to_chunk(&space_coords);
            assert_eq!(result, chunk, "Chunk coords: {:?}", space_coords);
        }

        #[test]
        fn tile_should_resolve_to_same_chunk_in_space(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 1.0);
            let tile = ExtrudedCubeHexCoord::new(CubeHexCoord::from_xz(x2, z2), 0);
            let space_coords = layout.tile_to_space(&tile);
            let space_chunk = layout.space_to_chunk(&space_coords);
            let tile_chunk = layout.tile_to_chunk(&tile);
            assert_eq!(space_chunk, tile_chunk);
        }

        #[test]
        fn tile_to_to_chunk_distance_should_be_radius_or_less(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 1.0);
            let tile = ExtrudedCubeHexCoord::new(CubeHexCoord::from_xz(x2, z2), 0);
            let chunk = layout.tile_to_chunk(&tile);
            let distance = chunk.distance_step(&tile.get_base());
            //println!("Lookup {:?}", layout.chunk_lookup);
            assert!(distance <= layout.chunk_radius_step(), "Expected upper bound: {:?}, distance: {:?}, tile: {:?}, chunk: {:?}", layout.chunk_radius_step(), distance, tile, chunk);
        }
    }
}
