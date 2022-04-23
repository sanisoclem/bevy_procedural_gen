use super::{generator::VoxelType, VoxelId};
use bevy::prelude::*;
use std::collections::HashMap;

// TODO: lod
// TODO: use asset loader and return Handle<Mesh> instead of blocking
pub fn generate_mesh(_voxels: &HashMap<VoxelId, VoxelType>, _lod: u8) -> Mesh {
  Mesh::from(shape::Plane { size: 1.0 * 50. })
}
