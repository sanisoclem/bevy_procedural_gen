use bevy::prelude::*;
use bevy::core::Timer;
use hex::*;
use mesh::*;
use noise::*;
use std::{
    collections::HashSet,
    fmt::Debug,
    hash::Hash,
    time::{Duration, Instant},
};

pub mod hex;
mod mesh;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<hex::CubeHexLayout>()
            .init_resource::<ChunkTracker<CubeHexCoord>>()
            .init_resource::<ChunkGenerator>()
            .init_resource::<TerrainOptions>()
            .add_startup_system(setup_chunk_tracker.system())
            .add_system(chunk_spawner.system())
            .add_system(chunk_solver.system())
            .add_system(chunk_shaper.system())
            .add_system(chunk_loader.system());
            // https://github.com/bevyengine/bevy/issues/135
            //.add_system(chunk_despawner.system());
    }
}

impl Default for TerrainPlugin {
    fn default() -> Self {
        TerrainPlugin
    }
}

pub struct TerrainOptions {
    pub chunk_size: i32,
    pub voxel_size: f32,
    pub origin: CubeHexCoord,
}
impl Default for TerrainOptions {
    fn default() -> Self {
        Self {
            chunk_size: 32,
            voxel_size: 32.0,
            origin: CubeHexCoord::default(),
        }
    }
}

pub struct ChunkTracker<ChunkAddress>
where
    ChunkAddress: Hash + Eq,
{
    pub loaded_chunks: HashSet<ChunkAddress>,
    pub placeholder_mesh: Option<Handle<Mesh>>,
    pub placeholder_material: Option<Handle<StandardMaterial>>,
    pub timer: Timer,
}
impl<ChunkAddress> Default for ChunkTracker<ChunkAddress>
where
    ChunkAddress: Hash + Eq,
{
    fn default() -> Self {
        ChunkTracker {
            loaded_chunks: HashSet::new(),
            placeholder_material: None,
            placeholder_mesh: None,
            timer: Timer::new(Duration::from_secs(5), true),
        }
    }
}
impl<ChunkAddress> ChunkTracker<ChunkAddress>
where
    ChunkAddress: Hash + Eq + Debug,
{
    pub fn try_spawn(&mut self, chunk: ChunkAddress) -> bool {
        if !self.loaded_chunks.contains(&chunk) {
            //println!("spawn chunk {:?}", chunk);
            self.loaded_chunks.insert(chunk);
            true
        } else {
            false
        }
    }
}

pub struct ChunkGenerator {
    pub generator: Perlin,
    pub scale: f64,
    pub bias: f64,
    pub uscale: f64,
    pub vscale: f64,
}
impl Default for ChunkGenerator {
    fn default() -> Self {
        ChunkGenerator {
            generator: Perlin::new().set_seed(20),
            scale: 1.0,
            bias: 0.0,
            uscale: 0.07,
            vscale: 0.07,
        }
    }
}

fn setup_chunk_tracker(
    hex_layout: Res<CubeHexLayout>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_tracker: ResMut<ChunkTracker<CubeHexCoord>>,
) {
    chunk_tracker.placeholder_material = Some(materials.add(Color::rgb(0.1, 0.2, 0.1).into()));
    chunk_tracker.placeholder_mesh = Some(meshes.add(mesh_hex_outline(
        Vec3::default(),
        Vec3::unit_y(),
        Vec3::unit_x(),
        hex_layout.size,
    )));
}

fn chunk_spawner(
    mut commands: Commands,
    hex_layout: Res<CubeHexLayout>,
    time: Res<Time>,
    mut chunk_tracker: ResMut<ChunkTracker<CubeHexCoord>>,
    mut query: Query<(&Translation, &mut ChunkSite)>,
) {
    // load chunks around cameras
    for (translation, mut site) in &mut query.iter() {
        // find which chunk we're currently on
        let current_chunk = hex_layout.space_to_hex(Vec2::new(translation.x(), translation.z()));

        // skip this site if it hasn't moved chunks since the last load
        if let Some(last_loaded) = site.last_loaded_chunk {
            if last_loaded == current_chunk {
                continue;
            }
        }
        //println!("checking for chunks to spawn");

        // find neighboring chunks
        let neighbors = hex_layout.get_neighbors(current_chunk, 1);

        // spawn chunks
        for chunk in std::iter::once(current_chunk).chain(neighbors) {
            if chunk_tracker.try_spawn(chunk) {
                let pos = hex_layout.hex_to_space(chunk);
                commands
                    .spawn(PbrComponents {
                        mesh: chunk_tracker.placeholder_mesh.unwrap(),
                        material: chunk_tracker.placeholder_material.unwrap(),
                        translation: Translation::new(pos.x(), 0.0, pos.y()),
                        ..Default::default()
                    })
                    .with_bundle(ChunkComponents {
                        chunk_index: chunk,
                        chunk_info: ChunkComponent::new(0.0, time.instant.unwrap()),
                    });
            }
        }
        site.fresh = true;
        site.last_loaded_chunk = Some(current_chunk);
    }

    // create entities for chunks
}

fn chunk_solver(
    mut query: Query<(&CubeHexCoord, &mut ChunkComponent)>,
    mut site_query: Query<(Entity, &mut ChunkSite)>,
) {
    for (_entity, mut site) in &mut site_query.iter() {
        // don't do anything if the site hasn't moved
        if !site.fresh {
            continue;
        }
        site.fresh = false;

        // loop through all chunks and update distances
        for (coord, mut chunk) in &mut query.iter() {
            // TODO: handle multiple chunk sites
            chunk.distance_to_nearest_site = site.last_loaded_chunk.unwrap().distance_step(coord);
        }
    }
}

fn chunk_shaper(
    keyboard_input: Res<Input<KeyCode>>,
    mut generator: ResMut<ChunkGenerator>,
    mut query: Query<(&CubeHexCoord, &mut ChunkComponent)>,
) {
    if keyboard_input.pressed(KeyCode::Up) {
        generator.bias += 0.1;
    }

    if keyboard_input.pressed(KeyCode::Down) {
        generator.bias -= 0.1;
    }

    if keyboard_input.pressed(KeyCode::Left) {
        generator.scale -= 0.1;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        generator.scale += 0.1;
    }

    if keyboard_input.pressed(KeyCode::Numpad8) {
        generator.uscale += 0.01;
        generator.vscale += 0.01;
    }

    if keyboard_input.pressed(KeyCode::Numpad2) {
        generator.uscale -= 0.01;
        generator.vscale -= 0.01;
    }

    if keyboard_input.pressed(KeyCode::Numpad4) {
        generator.uscale -= 0.1;
        generator.vscale -= 0.1;
    }

    if keyboard_input.pressed(KeyCode::Numpad6) {
        generator.uscale += 0.1;
        generator.vscale += 0.1;
    }

    let sp = ScalePoint::new(&generator.generator).set_all_scales(
        generator.uscale,
        generator.vscale,
        0.0,
        0.0,
    );
    let noise_gen = ScaleBias::new(&sp)
        .set_bias(generator.bias)
        .set_scale(generator.scale);
    for (coord, mut chunk) in &mut query.iter() {
        // find which chunk we're currently on
        let biome = noise_gen.get([coord.0 as f64, coord.1 as f64]);
        if biome != chunk.biome {
            chunk.biome = biome;
            chunk.chunk_loaded = false;
        }
    }
}

fn chunk_loader(
    hex_layout: Res<CubeHexLayout>,
    chunk_tracker: Res<ChunkTracker<CubeHexCoord>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut ChunkComponent, &mut Handle<Mesh>)>,
) {
    // enumerate chunks that needs to be loaded
    for (mut chunk_info, mut mesh) in &mut query.iter() {
        // skip chunks that are already loaded
        if chunk_info.chunk_loaded {
            continue;
        }
        //println!("Loading chunk {:?}", chunk);

        // TODO: check if there is any persisted chunk state
        // TODO: if yes, load from disk
        // if no, procedurally generate chunk
        // loading a chunk might need multiple cycles
        // once completely loaded, mark the chunk as loaded

        let new_mesh = mesh_hex_voxel(
            Vec3::new(0.0, chunk_info.biome as f32, 0.0),
            Vec3::new(0.0, (chunk_info.biome - 5.0) as f32, 0.0),
            Vec3::unit_y(),
            Vec3::unit_x(),
            hex_layout.size,
        );
        if mesh.id == chunk_tracker.placeholder_mesh.unwrap().id {
            *mesh = meshes.add(new_mesh);
        } else {
            meshes.set(*mesh, new_mesh)
        }
        //meshes.set(*mesh, mesh_hex_voxel(Vec3::new(0.0,chunk_info.biome as f32, 0.0), Vec3::default(), Vec3::unit_y(), Vec3::unit_x(), hex_layout.size));

        chunk_info.chunk_loaded = true;
    }
}

fn chunk_despawner(
    mut commands: Commands,
    mut chunk_tracker: ResMut<ChunkTracker<CubeHexCoord>>,
    time: Res<Time>,
    mut query: Query<(Entity, &ChunkComponent, &CubeHexCoord)>,
) {
    // only try to unload when timer is done
    chunk_tracker.timer.tick(time.delta_seconds);
    if chunk_tracker.timer.finished {
        for (entity, chunk_info, coord) in &mut query.iter() {
            if chunk_info.distance_to_nearest_site > 5 {
                // despawn chunk
                commands.despawn(entity);

                println!("Despawning {:?}", coord);
                // TODO: queue and cleanup tasks
            } else {
                println!("phew, not despawning {:?}", coord);
            }
        }

        chunk_tracker.timer.reset();
    }
    // find chunks that can be unloaded
    // mark them for despawning
}

// fn chunk_cleaner() {
//     // find chunks marked for despawning
//     // save chunk data to disk
//     // despawn chunks
// }

#[derive(Default, Debug)]
pub struct ChunkSite {
    pub last_loaded_chunk: Option<CubeHexCoord>,
    pub fresh: bool,
}

#[derive(Debug)]
pub struct ChunkComponent {
    pub chunk_loaded: bool,
    pub created: Instant,
    pub biome: f64,
    pub distance_to_nearest_site: i32,
}

impl ChunkComponent {
    pub fn new(biome: f64, time: Instant) -> Self {
        ChunkComponent {
            chunk_loaded: false,
            created: time,
            biome,
            distance_to_nearest_site: 0,
        }
    }
}

#[derive(Bundle)]
pub struct ChunkComponents {
    pub chunk_index: CubeHexCoord,
    pub chunk_info: ChunkComponent,
}
