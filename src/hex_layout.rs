use crate::terrain::{ChunkId, Layout, VoxelId, VoxelData};
use crate::mesh::{get_hex_vertices, calculate_normals};
use bevy::{ecs::lazy_static::lazy_static, math::Mat2, prelude::*};
use std::{
    hash::Hash,
    ops::{Add, Sub}, collections::HashMap,
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

    #[inline]
    pub fn x(&self) -> i32 {
        self.0
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.1
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.2
    }

    pub fn from_fractional_xz(x: f32, z: f32) -> CubeHexCoord {
        let y = -x - z;

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

    pub fn distance_step(&self, b: &CubeHexCoord) -> i32 {
        (i32::abs(self.x() - b.x()) + i32::abs(self.y() - b.y()) + i32::abs(self.z() - b.z())) / 2
    }
}
impl Add for CubeHexCoord {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self(
            self.x() + other.x(),
            self.y() + other.y(),
            self.z() + other.z(),
        )
    }
}
impl Sub for CubeHexCoord {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(
            self.x() - other.x(),
            self.y() - other.y(),
            self.z() - other.z(),
        )
    }
}
impl ChunkId for CubeHexCoord {}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct ExtrudedCubeHexCoord(pub i32, pub i32, pub i32, pub i32);
impl ExtrudedCubeHexCoord {
    pub fn from_hex2d(hex: CubeHexCoord, height: i32) -> Self {
        ExtrudedCubeHexCoord(hex.x(), hex.y(), hex.z(), height)
    }

    pub fn from_xzh(x: i32, z: i32, h: i32) -> Self {
        ExtrudedCubeHexCoord(x, -x - z, z, h)
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.1
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.2
    }

    #[inline]
    pub fn h(&self) -> i32 {
        self.3
    }

    #[inline]
    pub fn get_base(&self) -> CubeHexCoord {
        CubeHexCoord(self.x(), self.y(), self.z())
    }
}
impl VoxelId for ExtrudedCubeHexCoord {
    fn u(&self) -> i32 {
        self.x()
    }

    fn v(&self) -> i32 {
        self.y()
    }

    fn h(&self) -> i32 {
        self.h()
    }
}

pub struct CubeHexLayout {
    pub space_origin: CubeHexCoord,
    voxel_radius: f32,
    voxel_height: f32,
    chunk_voxel_radius: i32,
    chunk_voxel_period: i32,
    chunk_voxel_height: i32,
    chunk_lookup: Vec<CubeHexCoord>,
}
impl CubeHexLayout {
    #[inline]
    pub fn chunk_radius(&self) -> f32 {
        self.chunk_diameter() / 2.0
    }

    #[inline]
    pub fn chunk_diameter(&self) -> f32 {
        self.chunk_voxel_diameter() as f32 * self.voxel_diameter()
    }

    #[inline]
    pub fn chunk_voxel_diameter(&self) -> i32 {
        (self.chunk_voxel_radius * 2) + 1
    }

    #[inline]
    pub fn chunk_voxel_radius(&self) -> i32 {
        self.chunk_voxel_radius
    }

    #[inline]
    pub fn voxel_radius(&self) -> f32 {
        self.voxel_radius
    }

    #[inline]
    pub fn voxel_diameter(&self) -> f32 {
        self.voxel_radius * 2.0
    }

    fn get_ring<'a>(
        &'a self,
        center: CubeHexCoord,
        distance: i32,
    ) -> Box<(dyn Iterator<Item = CubeHexCoord> + 'a)> {
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

    fn build_lookup(chunk_voxel_radius: i32) -> (i32, Vec<CubeHexCoord>) {
        let radius = chunk_voxel_radius;
        let offset_base = 3 * radius + 1;
        let period = (3 * radius * radius) + offset_base;
        let half_period = (period - 1) / 2; // period is always odd
        let mut chunk_lookup = vec![CubeHexCoord::default(); period as usize + 1];

        for &phase in ([1, -1]).iter() {
            let mut section_start = radius;
            let mut upper = radius * 2;
            let mut lower = radius + 1;
            let mut is_lower = true;

            for offset in 0..=half_period {
                let key = if phase == 1 { offset } else { period - offset } as usize;

                if offset <= radius {
                    chunk_lookup[key] = CubeHexCoord::from_xz(offset * phase, 0);
                } else {
                    let inner_max = if is_lower { lower } else { upper };
                    let inner_offset = offset - section_start;
                    let x_offset = if is_lower {
                        radius + 1
                    } else {
                        inner_max - radius
                    };
                    let z_phase = if is_lower { 1 } else { -1 };

                    chunk_lookup[key] = CubeHexCoord::from_xz(
                        (inner_offset - x_offset) * phase,
                        ((radius + radius + 1) - inner_max) * phase * z_phase,
                    );

                    if inner_offset + 1 > inner_max {
                        if is_lower {
                            lower += 1;
                        } else {
                            upper -= 1;
                        };
                        is_lower = !is_lower;
                        section_start = offset;
                    }
                }
            }
        }
        (period, chunk_lookup)
    }

    pub fn new(
        origin: CubeHexCoord,
        voxel_radius: f32,
        chunk_radius: i32,
        chunk_height: i32,
        voxel_extrusion_height: f32,
    ) -> Self {
        let (period, lookup) = Self::build_lookup(chunk_radius);
        CubeHexLayout {
            space_origin: origin,
            voxel_radius,
            voxel_height: voxel_extrusion_height,
            chunk_voxel_radius: chunk_radius,
            chunk_voxel_height: chunk_height,
            chunk_lookup: lookup,
            chunk_voxel_period: period,
        }
    }
}
impl Default for CubeHexLayout {
    fn default() -> Self {
        CubeHexLayout::new(CubeHexCoord::default(), 1.0, 3, 20, 1.0)
    }
}
impl Layout for CubeHexLayout {
    type TChunkId = CubeHexCoord;
    type TChunkIdIterator = Box<dyn Iterator<Item = CubeHexCoord>>;
    type TVoxelId = ExtrudedCubeHexCoord;

    fn get_placeholder_mesh(&self) -> Mesh {
        crate::mesh::mesh_hex_plane(
            Vec3::default(),
            Vec3::unit_y(),
            Vec3::unit_z() * -1.0,
            self.chunk_radius() * 0.75,
        )
    }
    fn get_chunk_mesh(&self, voxels: &mut HashMap<Self::TVoxelId, VoxelData>) -> Mesh {
       todo!()
    }

    fn get_chunk_neighbors(&self, chunk: Self::TChunkId, distance: i32) -> Self::TChunkIdIterator {
        let radius = self.chunk_voxel_radius;
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

    fn get_chunk_voxels(&self, chunk: &Self::TChunkId) -> Vec<Self::TVoxelId> {
        std::iter::once(chunk.clone())
            .chain(
                (1..=self.chunk_voxel_radius()).flat_map(|ring| self.get_ring(chunk.clone(), ring)),
            )
            .flat_map(|base_hex| {
                (0..=self.chunk_voxel_height)
                    .map(move |h| ExtrudedCubeHexCoord::from_hex2d(base_hex, h))
            })
            .collect()
    }

    fn chunk_to_space(&self, chunk: &Self::TChunkId) -> Translation {
        self.voxel_to_space(&ExtrudedCubeHexCoord::from_hex2d(*chunk, 0))
    }

    fn voxel_to_chunk(&self, voxel: &Self::TVoxelId) -> Self::TChunkId {
        let radius = self.chunk_voxel_radius;
        let offset_base = 3 * radius + 1;
        let x_offset_based_on_z = (voxel.2 * offset_base) % self.chunk_voxel_period;
        let x_transposed_axis = x_offset_based_on_z - voxel.0; // chunk center if multiple of period
        let x_closest = x_transposed_axis % self.chunk_voxel_period;
        let x_upper = if x_closest < 0 {
            x_closest + self.chunk_voxel_period
        } else {
            x_closest
        } as usize;

        voxel.get_base() + self.chunk_lookup[x_upper]
    }

    fn voxel_to_space(&self, voxel: &Self::TVoxelId) -> Translation {
        let transposed = voxel.get_base() - self.space_origin;
        let result = HEX2SPACE.mul_vec2(Vec2::new(transposed.x() as f32, transposed.z() as f32))
            * self.voxel_radius;
        Translation::new(result.x(), voxel.h() as f32 * self.voxel_height, result.y())
    }

    fn space_to_voxel(&self, space: &Vec3) -> Self::TVoxelId {
        let frac = SPACE2HEX.mul_vec2(Vec2::new(space.x(), space.z())) / self.voxel_radius;
        let y = (space.y() / self.voxel_height).floor() as i32;
        let hex = CubeHexCoord::from_fractional_xz(frac.x(), frac.y()) + self.space_origin;
        ExtrudedCubeHexCoord::from_hex2d(hex, y)
    }

    fn space_to_chunk(&self, space: &Vec3) -> Self::TChunkId {
        self.voxel_to_chunk(&self.space_to_voxel(space))
    }

    fn get_chunk_distance(&self, a: &Self::TChunkId, b: &Self::TChunkId) -> i32 {
        a.distance_step(b) / self.chunk_voxel_diameter()
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
            prop_assert_eq!(coord.x() + coord.y() + coord.z(), 0);
        }
        #[test]
        fn from_axis_coords_must_total_to_zero(q in -10000i32..10000,
                                        r in -10000i32..10000) {
            let coord = CubeHexCoord::from_axis_coord(q, r);
            prop_assert_eq!(coord.x() + coord.y() + coord.z(), 0);
        }
    }

    proptest! {
        #[test]
        fn neighbor_should_have_correct_distance_in_space(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..50, distance in 1i32..10) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20, 1.0);
            let voxel = ExtrudedCubeHexCoord::from_xzh(x2, z2, 0);
            let chunk = layout.voxel_to_chunk(&voxel);
            let origin_space = layout.chunk_to_space(&chunk).0;
            let expected_distance_upper_bound = layout.chunk_diameter() * distance as f32;
            for neighbor in layout.get_chunk_neighbors(chunk, distance) {
                let neighbor_space = layout.chunk_to_space(&neighbor).0;
                let result = (neighbor_space - origin_space).abs().length();
                assert!(result < expected_distance_upper_bound, "expected upper bound: {}, actual: {}, pointA: {}, pointB: {}", expected_distance_upper_bound, result, origin_space, neighbor_space);
            }
        }

        #[test]
        fn neighbor_should_be_mutual(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..50, distance in 1i32..10) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20, 1.0);
            let voxel = ExtrudedCubeHexCoord::from_xzh(x2, z2, 0);
            let chunk = layout.voxel_to_chunk(&voxel);
            for neighbor in layout.get_chunk_neighbors(chunk, distance) {
                let ns: Vec<_> = layout.get_chunk_neighbors(neighbor, distance).collect();
                let original: Vec<_> = ns.clone().into_iter().filter(|n| *n == chunk).collect();
                assert_eq!(original.len(), 1);
                assert_eq!(original[0], chunk);
            }
        }

        #[test]
        fn chunk_space_coordinates_should_be_zero_when_at_origin(x1 in -10000i32..=10000, z1 in -10000i32..=10000, radius in 1i32..50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20, 1.0);
            let coords = layout.chunk_to_space(&layout.space_origin);
            assert_eq!(coords.x(), 0.0);
            assert_eq!(coords.y(), 0.0);
            assert_eq!(coords.z(), 0.0);
        }

        #[test]
        fn voxel_space_coordinates_should_be_reversible(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20, 1.0);
            let voxel = ExtrudedCubeHexCoord::from_xzh(x2, z2, 0);
            let space_coords = layout.voxel_to_space(&voxel);
            let result = layout.space_to_voxel(&space_coords);
            assert_eq!(result, voxel, "Coords: {:?}", space_coords);
        }

        #[test]
        fn chunk_space_coordinates_should_be_reversible(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20, 1.0);
            let voxel = ExtrudedCubeHexCoord::from_xzh(x2, z2, 0);
            let chunk = layout.voxel_to_chunk(&voxel);
            let space_coords = layout.chunk_to_space(&chunk);
            let result = layout.space_to_chunk(&space_coords);
            assert_eq!(result, chunk, "Chunk coords: {:?}", space_coords);
        }

        #[test]
        fn voxel_should_resolve_to_same_chunk_in_space(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20, 1.0);
            let voxel = ExtrudedCubeHexCoord::from_xzh(x2, z2, 0);
            let space_coords = layout.voxel_to_space(&voxel);
            let space_chunk = layout.space_to_chunk(&space_coords);
            let voxel_chunk = layout.voxel_to_chunk(&voxel);
            assert_eq!(space_chunk, voxel_chunk);
        }

        #[test]
        fn voxel_to_chunk_distance_should_be_radius_or_less(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20, 1.0);
            let voxel = ExtrudedCubeHexCoord::from_xzh(x2, z2, 0);
            let chunk = layout.voxel_to_chunk(&voxel);
            let distance = chunk.distance_step(&voxel.get_base());
            assert!(distance <= layout.chunk_voxel_radius(), "Expected upper bound: {:?}, distance: {:?}, voxel: {:?}, chunk: {:?}", layout.chunk_voxel_radius(), distance, voxel, chunk);
        }

        #[test]
        fn voxel_to_chunk_should_return_same_value_for_same_chunk(x1 in -10000i32..=10000, z1 in -10000i32..=10000, ring_num in 0i32..10, index in 0i32..1000, radius in 1i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, 20,  1.0);

            // find a random chunk via neighbors
            let mut chunk = CubeHexCoord::default();
            for _ring in 0..ring_num {
                let mut n: Vec<_> = layout.get_chunk_neighbors(chunk,1).collect();
                chunk = n.remove((index % n.len() as i32) as usize);
            }
            for ring in 1..=radius {
                for voxel in layout.get_ring(chunk, ring) {
                    let result = layout.voxel_to_chunk(&ExtrudedCubeHexCoord::from_hex2d(voxel, 0));
                    assert_eq!(result, chunk, "Voxel: {:?}, expected chunk: {:?}, actual: {:?}", voxel, chunk, result);
                }
            }
        }

        #[test]
        fn chunk_should_have_correct_number_of_voxels(x1 in -10000i32..=10000, z1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, radius in 1i32..=50, height in 0i32..=50) {
            let layout = CubeHexLayout::new(CubeHexCoord::from_xz(x1, z1), 1.0, radius, height, 1.0);

            let voxel = ExtrudedCubeHexCoord::from_xzh(x2, z2, 0);
            let chunk = layout.voxel_to_chunk(&voxel);
            let voxel_count = layout.get_chunk_voxels(&chunk).len() as i32;
            let expected = ((3 * radius * radius) + (3 * radius) + 1) * (height + 1); // 6 triangle cross-sections (excl center), each section has a number of voxels equal to the nth triangle number * height
            assert_eq!(expected, voxel_count);
        }
    }
}
