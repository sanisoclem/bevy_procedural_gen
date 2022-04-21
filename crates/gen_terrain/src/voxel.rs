

struct ChunkId(u64, u64);
struct VoxelId(u64, u64, u64)

#[derive(Default, Debug, Component)]
pub struct ChunkSpawner
{
    pub last_loaded_chunk: Option<ChunkId>,
    pub fresh: bool,
}

pub struct ChunkTracker
{
    pub loaded_chunks: HashSet<ChunkId>,
    pub despawn_timer: Timer,
    pub min_despawn_distance: f32,
}
impl<TChunk> Default for ChunkTracker<TChunk>
where
    TChunk: ChunkId,
{
    fn default() -> Self {
        ChunkTracker {
            loaded_chunks: HashSet::new(),
            despawn_timer: Timer::new(Duration::from_secs(1), true),
            min_despawn_distance: 10,
        }
    }
}
impl<TChunk> ChunkTracker<TChunk>
where
    TChunk: ChunkId,
{
    pub fn try_spawn(&mut self, chunk: TChunk) -> bool {
        if !self.loaded_chunks.contains(&chunk) {
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


#[derive(Default)]
pub struct VoxelTerrainGeneratorPlugin<L> {
  phantom: std::marker::PhantomData<&'static L>,
};

impl<L> Plugin for VoxelTerrainGeneratorPlugin<L>
where
  L: Layout
{
    fn build(&self, app: &mut App) {
        app.init_resource::<Placeholders>()
            .init_resource::<ChunkTracker<TChunkId>>()
            .init_resource::<TGenerator>()
            .init_resource::<TLayout>()
            .add_system(Self::chunk_solver)
            .add_system(Self::chunk_despawner)
            .add_system(Self::chunk_spawner);
    }
}

impl<L> VoxelTerrainGeneratorPlugin<L>
where
  L : Layout {

  pub fn chunk_spawner(
    mut commands: Commands,
    time: Res<Time>,
    layout: Res<L>,
    mut tracker: ResMut<ChunkTracker<TChunkId>>,
    mut query: Query<(&Transform, &mut ChunkSiteComponent<TChunkId>)>,
  ) {
    // load chunks around ChunkSites
    for (transform, mut site) in &mut query.iter() {
        // find which chunk we're currently on
        let current_chunk = layout.space_to_chunk(&transform.translation);

        // skip this site if it hasn't moved chunks since the last load
        if let Some(last_loaded) = site.last_loaded_chunk {
            if last_loaded == current_chunk {
                continue;
            }
        }

        // find neighboring chunks
        let neighbors = layout.get_chunk_neighbors(current_chunk, 2);

        // spawn chunks
        for chunk in std::iter::once(current_chunk).chain(neighbors) {
            if tracker.try_spawn(chunk) {
                //println!("Spawning {:?}", chunk);
                let pos = layout.chunk_to_space(&chunk);

                // create entities for chunks
                commands
                    .spawn(PbrComponents {
                        mesh: placeholders.placeholder_mesh.unwrap(),
                        material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
                        translation: Translation::new(pos.x(), pos.y(), pos.z()),
                        ..Default::default()
                    })
                    .with(ChunkComponent::<TChunkId, TVoxelId> {
                        id: chunk,
                        loaded: false,
                        created: time.instant.unwrap(),
                        distance_to_nearest_site: 0, // will be computed by another system
                        voxels: None,
                    });
            }
        }

        site.fresh = true;
        site.last_loaded_chunk = Some(current_chunk);
    }
  }
}
