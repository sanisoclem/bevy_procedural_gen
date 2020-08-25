use bevy::prelude::*;
use std::{
    collections::HashSet,
    hash::Hash,
    fmt::Debug,
    marker::PhantomData,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct TerrainPlugin<'a, TChunkId, TTileId, TLayout> {
    phantom: PhantomData<&'a (TChunkId, TTileId, TLayout)>,
}

impl<TChunkId, TTileId, TLayout> Plugin for TerrainPlugin<'static, TChunkId, TTileId, TLayout>
where
    TChunkId: ChunkId,
    TTileId: TileId,
    TLayout: Layout<TChunkId = TChunkId> + Default,
{
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Placeholders>()
            .init_resource::<ChunkTracker<TChunkId>>()
            .init_resource::<TLayout>()
            .add_startup_system(Self::setup.system())
            .add_system(Self::chunk_spawner.system())
            .add_system(Self::chunk_solver.system());
    }
}

impl<TChunkId, TTileId, TLayout> TerrainPlugin<'static, TChunkId, TTileId, TLayout>
where
    TChunkId: 'static + ChunkId,
    TTileId: 'static + TileId,
    TLayout: 'static + Layout<TChunkId = TChunkId>,
{
    pub fn setup(
        layout: Res<TLayout>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut placeholders: ResMut<Placeholders>,
    ) {
        placeholders.placeholder_mat = Some(materials.add(Color::rgb(0.1, 0.2, 0.1).into()));
        placeholders.placeholder_mesh = Some(meshes.add(layout.get_placeholder_mesh()));
    }

    pub fn chunk_spawner(
        mut commands: Commands,
        time: Res<Time>,
        layout: Res<TLayout>,
        placeholders: Res<Placeholders>,
        mut tracker: ResMut<ChunkTracker<TChunkId>>,
        mut query: Query<(&Translation, &mut ChunkSiteComponent<TChunkId>)>,
    ) {
        // load chunks around ChunkSites
        for (translation, mut site) in &mut query.iter() {
            // find which chunk we're currently on
            let current_chunk = layout.space_to_chunk(&translation);

            // skip this site if it hasn't moved chunks since the last load
            if let Some(last_loaded) = site.last_loaded_chunk {
                if last_loaded == current_chunk {
                    continue;
                }
            }

            // find neighboring chunks
            let neighbors = layout.get_chunk_neighbors(current_chunk, 1);

            // spawn chunks
            for chunk in std::iter::once(current_chunk).chain(neighbors) {
                if tracker.try_spawn(chunk) {
                    println!("Spawning {:?}", chunk);
                    let pos = layout.chunk_to_space(&chunk);

                    // create entities for chunks
                    commands
                        .spawn(PbrComponents {
                            mesh: placeholders.placeholder_mesh.unwrap(),
                            material: placeholders.placeholder_mat.unwrap(),
                            translation: Translation::new(pos.x(), pos.y(), pos.z()),
                            ..Default::default()
                        })
                        .with(ChunkComponent {
                            id: chunk,
                            loaded: false,
                            created: time.instant.unwrap(),
                            distance_to_nearest_site: 0, // will be computed by another system
                        });
                }
            }

            site.fresh = true;
            site.last_loaded_chunk = Some(current_chunk);
        }
    }

    pub fn chunk_solver(
        layout: Res<TLayout>,
        mut query: Query<&mut ChunkComponent<TChunkId>>,
        mut site_query: Query<(Entity, &mut ChunkSiteComponent<TChunkId>)>,
    ) {
        // compute chunk distances (for LODs and despawning)
        for (_entity, mut site) in &mut site_query.iter() {
            // don't do anything if the site hasn't moved
            if !site.fresh {
                continue;
            }
            site.fresh = false;

            // loop through all chunks and update distances
            for mut chunk in &mut query.iter() {
                // TODO: handle multiple chunk sites
                chunk.distance_to_nearest_site =
                    layout.get_chunk_distance(&chunk.id, &site.last_loaded_chunk.unwrap());
            }
        }
    }
}

pub trait TileId: Eq + Hash + Sync + Send + Copy + Debug {}
pub trait ChunkId: Eq + Hash + Sync + Send + Copy + Debug {}

pub trait Layout: Sync + Send {
    type TTileId: TileId;
    type TChunkId: ChunkId;
    type TChunkIdIterator: Iterator<Item = Self::TChunkId>;

    fn get_placeholder_mesh(&self) -> Mesh;
    fn get_chunk_neighbors(&self, chunk: Self::TChunkId, distance: i32) -> Self::TChunkIdIterator;

    fn chunk_to_space(&self, chunk: &Self::TChunkId) -> Translation;
    fn tile_to_chunk(&self, tile: &Self::TTileId) -> Self::TChunkId;
    fn tile_to_space(&self, tile: &Self::TTileId) -> Translation;
    fn space_to_tile(&self, space: &Vec3) -> Self::TTileId;
    fn space_to_chunk(&self, space: &Vec3) -> Self::TChunkId;

    fn get_chunk_distance(&self, a: &Self::TChunkId, b: &Self::TChunkId) -> i32;
}

#[derive(Default, Debug)]
pub struct ChunkSiteComponent<TChunk>
where
    TChunk: ChunkId,
{
    pub last_loaded_chunk: Option<TChunk>,
    pub fresh: bool,
}

#[derive(Debug)]
pub struct ChunkComponent<TChunk>
where
    TChunk: ChunkId,
{
    pub id: TChunk,
    pub created: Instant,
    pub distance_to_nearest_site: i32,
    pub loaded: bool,
}

// #[derive(Bundle)]
// pub struct ChunkComponents<TChunk> where TChunk: ChunkId {
//     pub chunk: ChunkComponent<TChunk>,
//     //pub voxel: HexVoxelChunkComponent,
// }

pub struct ChunkTracker<TChunk>
where
    TChunk: ChunkId,
{
    pub loaded_chunks: HashSet<TChunk>,
    pub despawn_timer: Timer,
    pub min_despawn_distance: i32,
}
impl<TChunk> Default for ChunkTracker<TChunk>
where
    TChunk: ChunkId,
{
    fn default() -> Self {
        ChunkTracker {
            loaded_chunks: HashSet::new(),
            despawn_timer: Timer::new(Duration::from_secs(5), true),
            min_despawn_distance: 5,
        }
    }
}
impl<TChunk> ChunkTracker<TChunk>
where
    TChunk: ChunkId,
{
    pub fn try_spawn(&mut self, chunk: TChunk) -> bool {
        if !self.loaded_chunks.contains(&chunk) {
            //println!("spawn chunk {:?}", chunk);
            self.loaded_chunks.insert(chunk);
            true
        } else {
            false
        }
    }

    pub fn try_despawn(&mut self, chunk: TChunk) -> bool {
        self.loaded_chunks.remove(&chunk)
    }
}

#[derive(Default, Debug)]
pub struct Placeholders {
    pub placeholder_mesh: Option<Handle<Mesh>>,
    pub placeholder_mat: Option<Handle<StandardMaterial>>,
}
