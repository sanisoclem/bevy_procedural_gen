use super::{generator::VoxelType, VoxelId};
use bevy::{
  prelude::*,
  tasks::{AsyncComputeTaskPool, Task},
};
use std::collections::HashMap;

// TODO: lod
// TODO: use asset loader and return Handle<Mesh> instead of blocking
pub fn generate_mesh(
  thread_pool: &Res<AsyncComputeTaskPool>,
  _voxels: &HashMap<VoxelId, VoxelType>,
  _lod: u8,
) -> Task<Mesh> {
  // how do we use the voxel data?
  // we cannot move the voxel data out of the ecs system
  // for now we could clone it but maybe the voxel data needs to sit somewhere else
  // but! if it's not in the ecs, how do we edit the voxel data from a system?
  // and if we can edit, we need to make sure that we don't edit while we are using it to generate
  // the mesh hmmm... maybe we need some sort of double buffer?
  // edits are made in the front buffer while we use the back buffer to generate the mesh
  // we swap buffers if there are changes in the front buffer and mesh generation is complete
  thread_pool.spawn(async move { Mesh::from(shape::Plane { size: 1.0 * 23. }) })
}
