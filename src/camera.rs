use bevy::prelude::*;

#[derive(Component)]
pub struct CameraTarget(Option<Entity>);

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app.add_system(look_at_target);
  }
}

pub fn look_at_target(
  mut _qry: ParamSet<(
    Query<(&Camera, &CameraTarget, &mut Transform)>,
    Query<&Transform>,
  )>,
) {
  // TODO: move camera to look at target
}
