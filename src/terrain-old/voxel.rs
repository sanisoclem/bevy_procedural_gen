use bevy::prelude::*;
use noise::*;

use super::CubeHexCoord;

#[derive(Default)]
pub struct HexVoxelChunkComponent {
    pub voxels: Vec<HexVoxel>,
    pub height: i32,
    pub radius: i32,
    pub loaded: bool,
}

pub struct HexVoxel {}

pub struct HexVoxelGenerator {
    pub chunk_height: i32,
    pub chunk_radius: i32,
    pub generator: Perlin,
    pub scale: f64,
    pub bias: f64,
    pub uscale: f64,
    pub vscale: f64,
}

impl Default for HexVoxelGenerator {
    fn default() -> Self {
        HexVoxelGenerator {
            chunk_height: 10,
            chunk_radius: 20,
            generator: Perlin::new(),
            scale: 1.0,
            bias: 0.0,
            uscale: 0.07,
            vscale: 0.07,
        }
    }
}

impl HexVoxelGenerator {
    pub fn build_voxel_chunk(&self, chunk_coord: &CubeHexCoord) -> HexVoxelChunkComponent {
        let sp = ScalePoint::new(&self.generator).set_all_scales(self.uscale, self.vscale, 0.0, 0.0);
        let noise_gen: ScaleBias<Point3<f64>> =
            ScaleBias::new(&sp)
            .set_bias(self.bias)
            .set_scale(self.scale);
        // find global coord of chunk center
        let distance = chunk_coord.distance_step(&CubeHexCoord::default());
        todo!()
    }
    pub fn build_mesh(&self, voxel: &HexVoxel) -> Mesh {
        todo!()
    }
}
