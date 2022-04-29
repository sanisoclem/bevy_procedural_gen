use super::ChunkId;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Default)]
pub struct ChunkTracker {
  pub loaded_chunks: HashSet<ChunkId>,
}
impl ChunkTracker {
  pub fn try_spawn(&mut self, chunk: &ChunkId) -> bool {
    if !self.loaded_chunks.contains(chunk) {
      self.loaded_chunks.insert(chunk.clone());
      info!("spawned chunk {:?}", chunk);
      true
    } else {
      false
    }
  }

  pub fn try_despawn(&mut self, chunk: &ChunkId) -> bool {
    let retval = self.loaded_chunks.remove(chunk);
    if retval {
      info!("despawned chunk {:?}", chunk);
    }
    retval
  }
}
