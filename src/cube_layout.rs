use crate::terrain::{ChunkId, Layout, VoxelId, VoxelData};
use crate::mesh::{get_hex_vertices, calculate_normals};
use bevy::{ecs::lazy_static::lazy_static, math::Mat2, prelude::*};
use std::{
    hash::Hash,
    ops::{Add, Sub}, collections::HashMap,
};

lazy_static! {
    static ref ROTATE_4X: [Mat2; 4] =
        [ Mat2::from_cols_array(&[0.0, 1.0, -1.0, 0.0])
        , Mat2::from_cols_array(&[-1.0, 0.0, 0.0, -1.0])
        , Mat2::from_cols_array(&[0.0, -1.0, 1.0, 0.0])
        , Mat2::from_cols_array(&[1.0, 0.0, 0.0, 1.0]) ];
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct ChunkCoord(pub i32, pub i32);
impl ChunkCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self(x, y)
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.1
    }
}
impl Add for ChunkCoord {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self(
            self.x() + other.x(),
            self.y() + other.y()
        )
    }
}
impl Sub for ChunkCoord {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(
            self.x() - other.x(),
            self.y() - other.y()
        )
    }
}
impl ChunkId for ChunkCoord {}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct VoxelCoord(pub i32, pub i32, pub i32);
impl VoxelCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(x, y, z)
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
}
impl VoxelId for VoxelCoord {
    fn u(&self) -> i32 {
        self.x()
    }

    fn v(&self) -> i32 {
        self.z()
    }

    fn h(&self) -> i32 {
        self.y()
    }
}
impl Add for VoxelCoord {
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
impl Sub for VoxelCoord {
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

pub struct CubeLayout {
    pub origin: ChunkCoord,
    voxel_side_length: f32,
    chunk_voxel_length: i32,
    chunk_voxel_height: i32,
}
impl CubeLayout {
    #[inline]
    pub fn chunk_side_length(&self) -> f32 { self.chunk_voxel_full_length() as f32 * self.voxel_side_length }

    #[inline]
    pub fn chunk_voxel_full_length(&self) -> i32 { 1 + (self.chunk_voxel_length * 2) }

    #[inline]
    pub fn get_center_voxel(&self, chunk: &ChunkCoord) -> VoxelCoord {
        VoxelCoord::new(chunk.x() * self.chunk_voxel_full_length(), 0, chunk.y() * self.chunk_voxel_full_length())
    }

    #[inline]
    pub fn get_voxel(&self, chunk: &ChunkCoord, x: i32, y: i32, z: i32) -> VoxelCoord {
        let vx = x + (chunk.x() * self.chunk_voxel_full_length());
        let vz = z + (chunk.y() * self.chunk_voxel_full_length());
        VoxelCoord::new(vx, y, vz)
    }

    pub fn new(
        origin: ChunkCoord,
        voxel_side_length: f32,
        chunk_voxel_length: i32,
        chunk_voxel_height: i32,
    ) -> Self {
        Self {
            origin,
            voxel_side_length,
            chunk_voxel_length,
            chunk_voxel_height
        }
    }
}
impl Default for CubeLayout {
    fn default() -> Self {
        Self::new(ChunkCoord::default(), 1.0, 50, 10)
    }
}
impl Layout for CubeLayout {
    type TChunkId = ChunkCoord;
    type TChunkIdIterator = Box<dyn Iterator<Item = Self::TChunkId>>;
    type TVoxelId = VoxelCoord;

    fn get_placeholder_mesh(&self) -> Mesh {
        Mesh::from(shape::Plane { size: self.chunk_side_length() })
    }

    fn get_chunk_mesh(&self, voxels: &mut HashMap<Self::TVoxelId, VoxelData>) -> Mesh {
       todo!()
    }

    fn get_chunk_neighbors(&self, chunk: Self::TChunkId, distance: i32) -> Self::TChunkIdIterator {
        Box::new((1..=distance)
            .flat_map(move |ring|
                (0..(2 *ring)).flat_map(move |offset|
                    ROTATE_4X.iter().map(move |rot|  rot.mul_vec2(Vec2::new((-ring + offset) as f32, -ring as f32)))
                .map(move |v2| chunk + ChunkCoord::new(v2.x() as i32, v2.y() as i32)))))
    }

    fn get_chunk_voxels(&self, chunk: &Self::TChunkId) -> Vec<Self::TVoxelId> {
        (0..self.chunk_voxel_full_length()).flat_map(|x|
            (0..self.chunk_voxel_full_length()).flat_map(move |z|
                (0..self.chunk_voxel_height).map(move |y| self.get_voxel(chunk, x - self.chunk_voxel_length, y, z - self.chunk_voxel_length))))
            .collect()
    }

    fn chunk_to_space(&self, chunk: &Self::TChunkId) -> Translation {
        self.voxel_to_space(&self.get_center_voxel(chunk))
    }

    fn voxel_to_chunk(&self, voxel: &Self::TVoxelId) -> Self::TChunkId {
        let x = (voxel.x() + self.chunk_voxel_length).div_euclid(self.chunk_voxel_full_length());
        let y = (voxel.z() + self.chunk_voxel_length).div_euclid(self.chunk_voxel_full_length());
        ChunkCoord::new(x, y)
    }

    fn voxel_to_space(&self, voxel: &Self::TVoxelId) -> Translation {
        let center = self.get_center_voxel(&self.origin);
        let transposed = *voxel - center;
        let x = transposed.x() as f32 * self.voxel_side_length;
        let y = transposed.y() as f32 * self.voxel_side_length;
        let z = transposed.z() as f32 * self.voxel_side_length;
        Translation::new(x, y, z)
    }

    fn space_to_voxel(&self, space: &Vec3) -> Self::TVoxelId {
        let center = self.get_center_voxel(&self.origin);
        let divisor = self.voxel_side_length as i32;
        let x = (space.x() as i32).div_euclid(divisor);
        let y = (space.y() as i32).div_euclid(self.chunk_voxel_height as i32);
        let z = (space.z() as i32).div_euclid(divisor);
        VoxelCoord::new(x, y, z) + center
    }

    fn space_to_chunk(&self, space: &Vec3) -> Self::TChunkId {
        self.voxel_to_chunk(&self.space_to_voxel(space))
    }

    fn get_chunk_distance(&self, a: &Self::TChunkId, b: &Self::TChunkId) -> i32 {
         (Vec2::new(a.x() as f32, a.y() as f32) - Vec2::new(b.x() as f32, b.y() as f32)).abs().length() as i32
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn chunk_should_have_appropriate_number_of_neighbors(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..50, distance in 1i32..10) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let chunk = layout.voxel_to_chunk(&voxel);
            let count =  layout.get_chunk_neighbors(chunk, distance).count();
            let expected = ((distance * 2) + 1) * ((distance * 2) + 1) - 1;
            assert_eq!(expected, count as i32);
        }

        #[test]
        fn neighbor_should_have_correct_distance(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..50, distance in 1i32..10) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let chunk = layout.voxel_to_chunk(&voxel);
            for neighbor in layout.get_chunk_neighbors(chunk, distance) {
                let diff = neighbor - chunk;
                let x = diff.x().abs();
                let y = diff.y().abs();
                let max = if x > y { x } else { y };
                assert!(max <= distance);
            }
        }

        #[test]
        fn neighbor_should_be_mutual(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..50, distance in 1i32..10) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let chunk = layout.voxel_to_chunk(&voxel);
            for neighbor in layout.get_chunk_neighbors(chunk, distance) {
                let ns: Vec<_> = layout.get_chunk_neighbors(neighbor, distance).collect();
                let original: Vec<_> = ns.clone().into_iter().filter(|n| *n == chunk).collect();
                assert_eq!(original.len(), 1);
                assert_eq!(original[0], chunk);
            }
        }

        #[test]
        fn chunk_space_coordinates_should_be_zero_when_at_origin(x1 in -10000i32..=10000, y1 in -10000i32..=10000, voxel_length in 1i32..50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let coords = layout.chunk_to_space(&layout.origin);
            assert_eq!(coords.x(), 0.0);
            assert_eq!(coords.y(), 0.0);
            assert_eq!(coords.z(), 0.0);
        }

        #[test]
        fn voxel_space_coordinates_should_be_reversible(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..=50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let space_coords = layout.voxel_to_space(&voxel);
            let result = layout.space_to_voxel(&space_coords);
            assert_eq!(result, voxel, "Coords: {:?}", space_coords);
        }

        #[test]
        fn chunk_space_coordinates_should_be_reversible(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..=50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let chunk = layout.voxel_to_chunk(&voxel);
            let space_coords = layout.chunk_to_space(&chunk);
            let result = layout.space_to_chunk(&space_coords);
            assert_eq!(result, chunk, "Chunk coords: {:?}", space_coords);
        }

        #[test]
        fn voxel_should_resolve_to_same_chunk_in_space(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..=50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let space_coords = layout.voxel_to_space(&voxel);
            let space_chunk = layout.space_to_chunk(&space_coords);
            let voxel_chunk = layout.voxel_to_chunk(&voxel);
            assert_eq!(space_chunk, voxel_chunk);
        }

        #[test]
        fn voxel_to_chunk_xz_distance_should_be_voxel_length_or_less(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..=50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let chunk = layout.voxel_to_chunk(&voxel);
            let chunk_center = layout.get_center_voxel(&chunk);
            let diff = voxel - chunk_center;
            let distance = if diff.x() > diff.z() { diff.x() } else { diff.z() };
            assert!(distance <= layout.chunk_voxel_length);
        }

        #[test]
        fn voxel_to_chunk_vertical_distance_should_be_voxel_length_or_less(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..=50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);
            let voxel = VoxelCoord::new(x2, 0, z2);
            let chunk = layout.voxel_to_chunk(&voxel);
            let chunk_center = layout.get_center_voxel(&chunk);
            let diff = voxel - chunk_center;
            let distance = diff.y().abs();
            assert!(distance <= layout.chunk_voxel_length);
        }

        #[test]
        fn voxel_to_chunk_should_return_same_value_for_same_chunk(x1 in -10000i32..=10000, y1 in -10000i32..=10000, ring_num in 0i32..10, index in 0i32..1000, voxel_length in 1i32..=50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, voxel_length);

            // find a random chunk via neighbors
            let mut chunk = ChunkCoord::default();
            for _ring in 0..ring_num {
                let mut n: Vec<_> = layout.get_chunk_neighbors(chunk,1).collect();
                chunk = n.remove((index % n.len() as i32) as usize);
            }
            for voxel in layout.get_chunk_voxels(&chunk) {
                let result = layout.voxel_to_chunk(&voxel);
                assert_eq!(result, chunk, "Voxel: {:?}, expected chunk: {:?}, actual: {:?}", voxel, chunk, result);
            }
        }

        #[test]
        fn chunk_should_have_correct_number_of_voxels(x1 in -10000i32..=10000, y1 in -10000i32..=10000, x2 in -10000i32..=10000, z2 in -10000i32..=10000, voxel_length in 1i32..=50, height in 0i32..=50) {
            let layout = CubeLayout::new(ChunkCoord::new(x1, y1), 1.0, voxel_length, height);

            let voxel = VoxelCoord::new(x2, 0, z2);
            let chunk = layout.voxel_to_chunk(&voxel);
            let voxel_count = layout.get_chunk_voxels(&chunk).len() as i32;
            let expected = (layout.chunk_voxel_full_length() * layout.chunk_voxel_full_length()) * height; // 6 triangle cross-sections (excl center), each section has a number of voxels equal to the nth triangle number * height
            assert_eq!(expected, voxel_count);
        }
    }
}
