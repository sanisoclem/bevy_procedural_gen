use super::VoxelId;
use bevy::{
  prelude::*,
  tasks::{AsyncComputeTaskPool, Task},
};
use std::collections::HashMap;

#[derive(Debug)]
pub enum VoxelType {
  Air,
  //  Dirt,
}

#[derive(Default)]
pub struct VoxelGenerator;

impl VoxelGenerator {
  pub fn load_voxel_data(
    &self,
    thread_pool: &Res<AsyncComputeTaskPool>,
    buffer: HashMap<VoxelId, VoxelType>,
  ) -> Task<super::ChunkVoxelData> {
    thread_pool.spawn(async move { super::ChunkVoxelData { voxels: buffer } })
  }
}
