use bevy::{
  prelude::*,
  tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use std::collections::HashMap;

// module organization doesn't make sense
// maybe the layout abstraction doesn't work
// because all the other modules depend on the layout
// mesh, voxel generation, voxelId and chunkId meaning etc
mod generator;
mod layout;
mod mesher;
mod tracker;

use layout::*;

// #[derive(Debug)]
// pub enum VoxelTerrainEvents {
//   ChunkSpawned(Entity),
//   ChunkDespawned(Entity)
// }

#[derive(Default, Debug, Component)]
pub struct ChunkSpawner {
  pub last_loaded_chunk: Option<ChunkId>,
  pub fresh: bool,
}

#[derive(Debug, Default, Component)]
pub struct Chunk {
  pub id: ChunkId,
  pub distance_to_nearest_spawner: f32,
}

#[derive(Debug, Default, Component)]
pub struct ChunkVoxelData {
  pub voxels: HashMap<VoxelId, generator::VoxelType>,
}

#[derive(Default)]
pub struct VoxelTerrainPlugin;

impl Plugin for VoxelTerrainPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<tracker::ChunkTracker>()
      .init_resource::<generator::VoxelGenerator>()
      .init_resource::<layout::CubicVoxelLayout>()
      .add_system(spawn_chunks)
      .add_system(calc_chunk_distances)
      .add_system(load_voxels)
      .add_system(build_chunk_mesh)
      .add_system(attach_chunk_mesh)
      .add_system(despawn_chunks);
  }
}

pub fn spawn_chunks(
  mut commands: Commands,
  thread_pool: Res<AsyncComputeTaskPool>,
  layout: Res<layout::CubicVoxelLayout>,
  generator: Res<generator::VoxelGenerator>,
  mut tracker: ResMut<tracker::ChunkTracker>,
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
    let neighbors = layout.get_chunk_neighbors(&current_chunk, 2);

    // spawn chunks
    for chunk in std::iter::once(current_chunk).chain(neighbors) {
      if tracker.try_spawn(&chunk) {
        // println!("Spawning {:?}", chunk);
        let pos = layout.chunk_to_space(&chunk);

        let voxel_buffer = layout
          .get_chunk_voxels(&chunk)
          .into_iter()
          .map(|id| (id, generator::VoxelType::Air))
          .collect();

        // TODO: the voxel data might be better off in a resource
        // this allows access to the voxel data from an async task
        let load_voxels_task = generator.load_voxel_data(&thread_pool, voxel_buffer);

        // create entities for chunks
        commands
          .spawn()
          .insert(Transform::from_translation(pos))
          .insert(Chunk {
            id: chunk,
            distance_to_nearest_spawner: 0., // will be computed by another system
          })
          .insert(load_voxels_task);
      }
    }

    site.fresh = true;
    site.last_loaded_chunk = Some(current_chunk);
  }
}

pub fn calc_chunk_distances(
  layout: Res<layout::CubicVoxelLayout>,
  mut query: Query<&mut Chunk>,
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
      chunk.distance_to_nearest_spawner = min_distance;
    }
  }
}

pub fn load_voxels(
  mut commands: Commands,
  mut tasks: Query<(Entity, &Chunk, &mut Task<ChunkVoxelData>)>,
) {
  // check if voxel data load task is complete
  for (entity, chunk, mut task) in tasks.iter_mut() {
    if let Some(voxel_data) = future::block_on(future::poll_once(&mut *task)) {
      info!("voxels loaded for {:?}", chunk.id);
      // Add our new PbrBundle of components to our tagged entity
      commands
        .entity(entity)
        .insert(voxel_data)
        .remove::<Task<ChunkVoxelData>>();
    }
  }
}

pub fn build_chunk_mesh(
  mut commands: Commands,
  thread_pool: Res<AsyncComputeTaskPool>,
  query: Query<(Entity, &Chunk, &ChunkVoxelData), Without<Handle<Mesh>>>,
) {
  for (entity, chunk, voxel_data) in query.iter() {
    let gen_mesh_task = mesher::generate_mesh(&thread_pool, &voxel_data.voxels, 0);
    info!("generating mesh for {:?}", chunk.id);

    commands.entity(entity).insert(gen_mesh_task);
  }
}

pub fn attach_chunk_mesh(
  layout: Res<layout::CubicVoxelLayout>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut tasks: Query<(Entity, &Chunk, &mut Task<Mesh>), Without<Handle<Mesh>>>,
) {
  for (entity, chunk, mut task) in tasks.iter_mut() {
    if let Some(mesh) = future::block_on(future::poll_once(&mut *task)) {
      info!("generated mesh for {:?}", chunk.id);

      commands.entity(entity).insert_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.5, 0.0, 0.3).into()),
        transform: Transform::from_translation(layout.chunk_to_space(&chunk.id)),
        ..default()
      });
    }
  }
}

pub fn despawn_chunks(
  mut commands: Commands,
  mut tracker: ResMut<tracker::ChunkTracker>,
  qry: Query<(Entity, &Chunk)>,
) {
  for (entity, chunk) in qry.iter() {
    // TODO: figure out proper criteria for despawning
    if chunk.distance_to_nearest_spawner > 10000.0 && tracker.try_despawn(&chunk.id) {
      commands.entity(entity).despawn_recursive();
    }
  }
}
