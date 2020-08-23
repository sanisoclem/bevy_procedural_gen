use bevy::prelude::*;

mod chunk;
mod hex;
mod mesh;
mod voxel;
//mod biome;

pub use chunk::*;
pub use hex::*;
pub use mesh::*;
pub use voxel::*;
// pub use biome::*;

#[derive(Default)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<CubeHexLayout>()
            .init_resource::<ChunkTracker>()
            .init_resource::<HexVoxelGenerator>()
            .add_startup_system(setup.system())
            .add_system(chunk_spawner.system())
            .add_system(chunk_solver.system())
            .add_system(chunk_loader.system());
        // https://github.com/bevyengine/bevy/issues/135
        //.add_system(chunk_despawner.system());
    }
}
