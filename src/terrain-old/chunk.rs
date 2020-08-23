use super::{mesh_hex_outline, CubeHexCoord, CubeHexLayout, HexLayout, HexVoxelChunkComponent, HexVoxelGenerator};
use bevy::{core::Timer, prelude::*};
use std::{
    collections::HashSet,
    fmt::Debug,
    time::{Duration, Instant},
};

#[derive(Default, Debug)]
pub struct ChunkSiteComponent {
    pub last_loaded_chunk: Option<CubeHexCoord>,
    pub fresh: bool,
}

#[derive(Debug)]
pub struct ChunkComponent {
    pub center: CubeHexCoord,
    pub created: Instant,
    pub distance_to_nearest_site: i32,
    pub loaded: bool,
}

#[derive(Bundle)]
pub struct ChunkComponents {
    pub chunk: ChunkComponent,
    pub voxel: HexVoxelChunkComponent,
}

pub struct ChunkTracker {
    pub loaded_chunks: HashSet<CubeHexCoord>,
    pub placeholder_mesh: Option<Handle<Mesh>>,
    pub placeholder_material: Option<Handle<StandardMaterial>>,
    pub despawn_timer: Timer,
    pub min_despawn_distance: i32,
}
impl Default for ChunkTracker {
    fn default() -> Self {
        ChunkTracker {
            loaded_chunks: HashSet::new(),
            placeholder_material: None,
            placeholder_mesh: None,
            despawn_timer: Timer::new(Duration::from_secs(5), true),
            min_despawn_distance: 5,
        }
    }
}
impl ChunkTracker {
    pub fn try_spawn(&mut self, chunk: CubeHexCoord) -> bool {
        if !self.loaded_chunks.contains(&chunk) {
            //println!("spawn chunk {:?}", chunk);
            self.loaded_chunks.insert(chunk);
            true
        } else {
            false
        }
    }

    pub fn try_despawn(&mut self, chunk: CubeHexCoord) -> bool {
        self.loaded_chunks.remove(&chunk)
    }
}

pub fn setup(
    hex_layout: Res<CubeHexLayout>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_tracker: ResMut<ChunkTracker>,
) {
    chunk_tracker.placeholder_material = Some(materials.add(Color::rgb(0.1, 0.2, 0.1).into()));
    chunk_tracker.placeholder_mesh = Some(meshes.add(mesh_hex_outline(
        Vec3::default(),
        Vec3::unit_y(),
        Vec3::unit_x(),
        hex_layout.hex_size,
    )));
}

pub fn chunk_spawner(
    mut commands: Commands,
    time: Res<Time>,
    hex_layout: Res<CubeHexLayout>,
    mut chunk_tracker: ResMut<ChunkTracker>,
    mut query: Query<(&Translation, &mut ChunkSiteComponent)>,
) {
    // load chunks around ChunkSites
    for (translation, mut site) in &mut query.iter() {
        // find which chunk we're currently on
        let current_chunk = hex_layout.space_to_hex(Vec2::new(translation.x(), translation.z()));

        // skip this site if it hasn't moved chunks since the last load
        if let Some(last_loaded) = site.last_loaded_chunk {
            if last_loaded == current_chunk {
                continue;
            }
        }

        // find neighboring chunks
        let neighbors = hex_layout.get_chunk_neighbors(current_chunk,  1);

        // spawn chunks
        for chunk in std::iter::once(current_chunk).chain(neighbors) {
            if chunk_tracker.try_spawn(chunk) {
                let pos = hex_layout.hex_to_space(chunk);

                // create entities for chunks
                commands
                    .spawn(PbrComponents {
                        mesh: chunk_tracker.placeholder_mesh.unwrap(),
                        material: chunk_tracker.placeholder_material.unwrap(),
                        translation: Translation::new(pos.x(), 0.0, pos.y()),
                        ..Default::default()
                    })
                    .with_bundle(ChunkComponents {
                        chunk: ChunkComponent {
                            center: chunk,
                            loaded: false,
                            distance_to_nearest_site: 0, // will be computed by another system
                            created: time.instant.unwrap(),
                        },
                        voxel: HexVoxelChunkComponent::default(),
                    });
            }
        }

        site.fresh = true;
        site.last_loaded_chunk = Some(current_chunk);
    }
}

pub fn chunk_solver(
    mut query: Query<(&mut ChunkComponent)>,
    mut site_query: Query<(Entity, &mut ChunkSiteComponent)>,
) {
    // compute chunk distances (for LODs and despawning)
    for (_entity, mut site) in &mut site_query.iter() {
        // don't do anything if the site hasn't moved
        if !site.fresh {
            continue;
        }
        site.fresh = false;

        // loop through all chunks and update distances
        for (mut chunk) in &mut query.iter() {
            // TODO: handle multiple chunk sites
            chunk.distance_to_nearest_site = site.last_loaded_chunk.unwrap().distance_step(&chunk.center);
        }
    }
}

pub fn chunk_voxel_loader (
    hex_layout: Res<CubeHexLayout>,
    voxel_gen: Res<HexVoxelGenerator>,
    mut query: Query<(&mut HexVoxelChunkComponent, &ChunkComponent)>,
) {
    for (mut voxel_component, mut chunk) in &mut query.iter() {
        if (voxel_component.loaded) {
            continue;
        }

        // mark as loaded
        voxel_component.loaded = true;


    }
}

pub fn chunk_loader(
    hex_layout: Res<CubeHexLayout>,
    chunk_tracker: Res<ChunkTracker>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut ChunkComponent, &mut Handle<Mesh>)>,
) {
    // enumerate chunks that needs to be loaded
    for (mut chunk_info, mut mesh) in &mut query.iter() {
        // skip chunks that are already loaded
        if chunk_info.loaded {
            continue;
        }

        // TODO: check if there is any persisted chunk state
        // TODO: if yes, load from disk
        // if no, procedurally generate chunk
        // loading a chunk might need multiple cycles
        // once completely loaded, mark the chunk as loaded

        // let new_mesh = mesh_hex_voxel(
        //     Vec3::new(0.0, chunk_info.biome as f32, 0.0),
        //     Vec3::new(0.0, (chunk_info.biome - 5.0) as f32, 0.0),
        //     Vec3::unit_y(),
        //     Vec3::unit_x(),
        //     hex_layout.size,
        // );

        if mesh.id == chunk_tracker.placeholder_mesh.unwrap().id {
            //*mesh = meshes.add(new_mesh);
        } else {
            //meshes.set(*mesh, new_mesh)
        }

        chunk_info.loaded = true;
    }
}

pub fn chunk_despawner(
    mut commands: Commands,
    time: Res<Time>,
    mut chunk_tracker: ResMut<ChunkTracker>,
    mut query: Query<(Entity, &ChunkComponent)>,
) {
    // only try to unload when timer is done
    chunk_tracker.despawn_timer.tick(time.delta_seconds);
    if chunk_tracker.despawn_timer.finished {
        for (entity, chunk_info) in &mut query.iter() {
            if chunk_info.distance_to_nearest_site > chunk_tracker.min_despawn_distance {
                // despawn chunk
                commands.despawn(entity);

                // TODO: queue and cleanup tasks
            }
        }

        chunk_tracker.despawn_timer.reset();
    }
    // find chunks that can be unloaded
    // mark them for despawning
}

// fn chunk_cleaner() {
//     // find despawned chunks
//     // save state to disk
//     // cleanup resources
// }
