use bevy::{prelude::*, window::CursorMoved};

#[derive(Component)]
pub struct RtsCamera;

pub struct RtsCameraPlugin;

impl Plugin for RtsCameraPlugin {
  fn build(&self, app: &mut App) {
    app.add_startup_system(setup).add_system(rts_camera_system);
  }
}

const MOUSE_PAN_SPEED: f32 = 100.0;
const MOUSE_PAN_MARGINS: f32 = 0.1;

#[derive(Default)]
pub struct State {
  pos: Vec2,
}

pub fn setup(mut commands: Commands) {
  commands
    .spawn_bundle(PerspectiveCameraBundle {
      transform: Transform::from_xyz(-2.0, 10.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
      ..default()
    })
    .insert(RtsCamera);
}

pub fn rts_camera_system(
  mut state: Local<State>,
  time: Res<Time>,
  windows: Res<Windows>,
  mut cursor_moved_events: EventReader<CursorMoved>,
  mut camera_query: Query<&mut Transform, With<RtsCamera>>,
) {
  // Get latest cursor location
  if let Some(event) = cursor_moved_events.iter().next_back() {
    // Adjust for window size and store in 0.0 - 1.0 range
    let window = windows.get(event.id).expect("window not found");
    state.pos.x = event.position.x / (window.width() as f32);
    state.pos.y = event.position.y / (window.height() as f32);
  }

  let pos = state.pos;

  // Check if mouse is within edge margins for x
  let horizontal = if pos.x < MOUSE_PAN_MARGINS {
    -(MOUSE_PAN_MARGINS - pos.x) * MOUSE_PAN_SPEED
  } else if pos.x > (1.0 - MOUSE_PAN_MARGINS) {
    (pos.x - (1.0 - MOUSE_PAN_MARGINS)) * MOUSE_PAN_SPEED
  } else {
    0.
  };

  // Check if mouse is within edge margins for y
  let vertical = if pos.y < MOUSE_PAN_MARGINS {
    (MOUSE_PAN_MARGINS - pos.y) * MOUSE_PAN_SPEED
  } else if pos.y > (1.0 - MOUSE_PAN_MARGINS) {
    -(pos.y - (1.0 - MOUSE_PAN_MARGINS)) * MOUSE_PAN_SPEED
  } else {
    0.
  };

  // Apply movement to camera
  if let Ok(mut transform) = camera_query.get_single_mut() {
    transform.translation.x += horizontal * time.delta_seconds();
    transform.translation.z += vertical * time.delta_seconds();
  }
}
