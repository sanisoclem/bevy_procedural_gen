use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct ChunkId(u64, u64);
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Eq, Hash)]
pub struct VoxelId(u64, u64, u64);

#[derive(Default, Debug, Component)]
pub struct ChunkSpawner {
  pub last_loaded_chunk: Option<ChunkId>,
  pub fresh: bool,
}

#[derive(Debug, Default, Component)]
pub struct ChunkComponent<VD> {
  pub id: ChunkId,
  pub distance_to_nearest_site: f32,
  pub loaded: bool,
  pub voxels: Option<HashMap<VoxelId, VD>>,
  //dead_timer: Option,
}

#[derive(Default)]
pub struct ChunkTracker {
  pub loaded_chunks: HashSet<ChunkId>,
}
impl ChunkTracker {
  pub fn try_spawn(&mut self, chunk: ChunkId) -> bool {
    if !self.loaded_chunks.contains(&chunk) {
      self.loaded_chunks.insert(chunk);
      true
    } else {
      false
    }
  }

  pub fn try_despawn(&mut self, chunk: ChunkId) -> bool {
    self.loaded_chunks.remove(&chunk)
  }
}

pub trait VoxelSource: Sync + Send {
  type VoxelData;
  fn get_voxels(&self, buffer: &mut HashMap<VoxelId, Self::VoxelData>);
}

pub trait Layout: Sync + Send {
  //fn get_chunk_mesh(&self, voxels: &mut HashMap<VoxelId, VoxelData>) -> Mesh;
  fn get_chunk_neighbors(&self, chunk: &ChunkId, distance: f32) -> Vec<ChunkId>;
  fn get_chunk_voxels(&self, chunk: &ChunkId) -> Vec<VoxelId>;

  fn chunk_to_space(&self, chunk: &ChunkId) -> Vec3;
  fn voxel_to_chunk(&self, tile: &VoxelId) -> ChunkId;
  fn voxel_to_space(&self, tile: &VoxelId) -> Vec3;
  fn space_to_voxel(&self, space: &Vec3) -> VoxelId;
  fn space_to_chunk(&self, space: &Vec3) -> ChunkId {
    self.voxel_to_chunk(&self.space_to_voxel(space))
  }

  fn get_chunk_distance(&self, a: &ChunkId, b: &ChunkId) -> f32;
}

#[derive(Default)]
pub struct VoxelTerrainPlugin<L, S> {
  phantom: std::marker::PhantomData<(L, S)>,
}

impl<L, S> Plugin for VoxelTerrainPlugin<L, S>
where
  L: Layout + FromWorld + 'static,
  S: VoxelSource + FromWorld + 'static,
  <S as VoxelSource>::VoxelData: Component + Default,
{
  fn build(&self, app: &mut App) {
    app
      .init_resource::<ChunkTracker>()
      .init_resource::<S>()
      .init_resource::<L>()
      .add_system(Self::spawn_chunks)
      .add_system(Self::solve_chunks)
      .add_system(Self::generate_chunk_mesh)
      .add_system(Self::load_voxels)
      .add_system(Self::despawn_chunks);
  }
}

impl<L, S> VoxelTerrainPlugin<L, S>
where
  L: Layout + 'static,
  S: VoxelSource + 'static,
  <S as VoxelSource>::VoxelData: Component + Default,
{
  pub fn spawn_chunks(
    mut commands: Commands,
    layout: Res<L>,
    mut tracker: ResMut<ChunkTracker>,
    mut query: Query<(&Transform, &mut ChunkSpawner)>,
  ) {
    for (transform, mut site) in query.iter_mut() {
      // find which chunk we're currently on
      let current_chunk = layout.space_to_chunk(&transform.translation);

      // skip this site if it hasn't moved chunks since the last load
      if let Some(last_loaded) = site.last_loaded_chunk {
        if last_loaded == current_chunk {
          continue;
        }
      }

      // find neighboring chunks
      let neighbors = layout.get_chunk_neighbors(&current_chunk, 2.0);

      // spawn chunks
      for chunk in std::iter::once(current_chunk).chain(neighbors) {
        if tracker.try_spawn(chunk) {
          //println!("Spawning {:?}", chunk);
          let pos = layout.chunk_to_space(&chunk);

          // create entities for chunks
          commands
            .spawn()
            .insert(Transform::from_translation(pos))
            .insert(ChunkComponent::<<S as VoxelSource>::VoxelData> {
              id: chunk,
              loaded: false,
              distance_to_nearest_site: 0., // will be computed by another system
              voxels: None,
            });
        }
      }

      site.fresh = true;
      site.last_loaded_chunk = Some(current_chunk);
    }
  }

  pub fn solve_chunks(
    layout: Res<L>,
    mut query: Query<&mut ChunkComponent<<S as VoxelSource>::VoxelData>>,
    mut site_query: Query<&mut ChunkSpawner>,
  ) {
    let mut fresh_sites = site_query
      .iter_mut()
      .filter(|site| site.fresh)
      .collect::<Vec<_>>();
    if fresh_sites.len() == 0 {
      return;
    }

    // compute chunk distances (for LODs and despawning)
    for mut chunk in query.iter_mut() {
      let mut min_distance = std::f32::MAX;
      for site in fresh_sites.iter_mut() {
        site.fresh = false;

        min_distance = layout
          .get_chunk_distance(
            &chunk.id,
            &site
              .last_loaded_chunk
              .expect("a fresh site should have a loaded chunk"),
          )
          .min(min_distance);
        chunk.distance_to_nearest_site = min_distance;
      }
    }
  }

  pub fn generate_chunk_mesh(
    _layout: Res<L>,
    _meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(
      &mut ChunkComponent<<S as VoxelSource>::VoxelData>,
      &mut Handle<Mesh>,
    )>,
  ) {
    // build chunk mesh
    for (mut chunk, mut _mesh) in &mut query.iter_mut() {
      // skip loaded chunks or chunks without voxels yet
      if chunk.voxels.is_none() || chunk.loaded {
        continue;
      }

      //*mesh = meshes.add(layout.get_chunk_mesh(&mut chunk.voxels.unwrap()));
      chunk.loaded = true;
    }
  }

  pub fn load_voxels(
    layout: Res<L>,
    generator: Res<S>,
    mut query: Query<&mut ChunkComponent<<S as VoxelSource>::VoxelData>>,
  ) {
    // load voxel data
    for mut chunk in &mut query.iter_mut() {
      if let Some(_) = chunk.voxels {
        continue;
      }

      let mut voxels = layout
        .get_chunk_voxels(&chunk.id)
        .into_iter()
        .map(|id| (id, S::VoxelData::default()))
        .collect();
      generator.get_voxels(&mut voxels);
      chunk.voxels = Some(voxels);

      // only load one chunk per frame
      break;
    }
  }

  pub fn despawn_chunks(
    mut _commands: Commands,
    _time: Res<Time>,
    mut _tracker: ResMut<ChunkTracker>,
    mut _query: Query<(Entity, &ChunkComponent<<S as VoxelSource>::VoxelData>)>,
  ) {
    // TODO: despawn inactive chunks (faraway and have not been in the camera for a while)
    // only try to unload when timer is done
    // tracker.despawn_timer.tick(time.delta_seconds);
    // if tracker.despawn_timer.finished {
    //     for (entity, chunk_info) in &mut query.iter() {
    //         if chunk_info.distance_to_nearest_site > tracker.min_despawn_distance {
    //             // despawn chunk
    //             if tracker.try_despawn(chunk_info.id) {
    //                 commands.despawn(entity);
    //             }
    //             // TODO: queue and cleanup tasks
    //         }
    //     }
    //     tracker.despawn_timer.reset();
    // }
    // find chunks that can be unloaded
    // mark them for despawning
  }
}
