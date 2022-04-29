use bevy::prelude::*;
use gen_terrain::{ChunkSpawner, VoxelTerrainPlugin};

mod camera;

fn main() {
  App::new()
    .insert_resource(WindowDescriptor {
      title: "Procedural Generation".to_string(),
      width: 1920.,
      height: 1080.,
      ..Default::default()
    })
    .insert_resource(Msaa { samples: 4 })
    .add_plugins(DefaultPlugins)
    .add_plugin(VoxelTerrainPlugin)
    .add_plugin(gen_camera::RtsCameraPlugin)
    .add_startup_system(setup)
    .add_system(add_chunk_spawner)
    .run();
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // cube
  commands.spawn_bundle(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    transform: Transform::from_xyz(0.0, 0.5, 0.0),
    ..default()
  });
  // light
  commands.spawn_bundle(PointLightBundle {
    point_light: PointLight {
      intensity: 1500.0,
      shadows_enabled: true,
      ..default()
    },
    transform: Transform::from_xyz(4.0, 8.0, 4.0),
    ..default()
  });
}

fn add_chunk_spawner(
  mut commands: Commands,
  qry: Query<Entity, (With<gen_camera::RtsCamera>, Without<ChunkSpawner>)>,
) {
  for entity in qry.iter() {
    commands.entity(entity).insert(ChunkSpawner::default());
  }
}
