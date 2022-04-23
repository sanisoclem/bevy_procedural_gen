use super::VoxelId;
use std::collections::HashMap;

#[derive(Debug)]
pub enum VoxelType {
  Air,
//  Dirt,
}

#[derive(Default)]
pub struct VoxelGenerator;

impl VoxelGenerator {
  pub fn get_voxels(&self, _buffer: &mut HashMap<VoxelId, VoxelType>) {

  }
}
