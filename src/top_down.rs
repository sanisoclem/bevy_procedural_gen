use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
    render::{
        camera::{Camera, PerspectiveProjection, VisibleEntities},
        render_graph::base,
    },
};

pub struct TopDownPlugin;

impl Plugin for TopDownPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(camera_movement_system.system());
    }
}

impl Default for TopDownPlugin {
    fn default() -> Self {
        TopDownPlugin
    }
}

pub struct TopDownCameraOptions {
    pub speed: f32,
    pub sensitivity: f32,
}
impl Default for TopDownCameraOptions {
    fn default() -> Self {
        Self {
            speed: 100.0,
            sensitivity: 3.0,
        }
    }
}
#[derive(Bundle)]
pub struct TopDownCamera {
    pub options: TopDownCameraOptions,
    pub camera: Camera,
    pub perspective_projection: PerspectiveProjection,
    pub visible_entities: VisibleEntities,
    pub transform: Transform,
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
}

impl Default for TopDownCamera {
    fn default() -> Self {
        Self {
            options: TopDownCameraOptions::default(),
            camera: Camera {
                name: Some(base::camera::CAMERA3D.to_string()),
                ..Default::default()
            },
            perspective_projection: Default::default(),
            visible_entities: Default::default(),
            transform: Transform::default(),
            translation: Translation::default(),
            rotation: Rotation::default(),
            scale: Scale::default(),
        }
    }
}

impl TopDownCamera {
    pub fn create_facing(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let transform = Mat4::face_toward(eye, center, up);
        let (_scale, rotation, translation) = transform.to_scale_rotation_translation();

        Self {
            transform: Transform::new(transform),
            translation: Translation::from(translation),
            rotation: Rotation::from(rotation),
            scale: Scale::default(),
            ..Default::default()
        }
    }
}

fn forward_vector(rotation: &Rotation) -> Vec3 {
    rotation.mul_vec3(Vec3::unit_z()).normalize()
}

fn forward_walk_vector(rotation: &Rotation) -> Vec3 {
    let f = forward_vector(rotation);
    let f_flattened = Vec3::new(f.x(), 0.0, f.z()).normalize();
    f_flattened
}

fn strafe_vector(rotation: &Rotation) -> Vec3 {
    Rotation::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_walk_vector(rotation))
        .normalize()
}

fn movement_axis(input: &Res<Input<KeyCode>>, plus: KeyCode, minus: KeyCode) -> f32 {
    let mut axis = 0.0;
    if input.pressed(plus) {
        axis += 1.0;
    }
    if input.pressed(minus) {
        axis -= 1.0;
    }
    axis
}

fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &TopDownCameraOptions,
        &crate::terrain::ChunkSiteComponent<crate::hex_layout::CubeHexCoord>,
        &mut Translation,
        &Rotation,
    )>,
) {
    let axis_h = movement_axis(&keyboard_input, KeyCode::D, KeyCode::A);
    let axis_v = movement_axis(&keyboard_input, KeyCode::S, KeyCode::W);

    let axis_float = movement_axis(&keyboard_input, KeyCode::Space, KeyCode::LShift);

    for (options, _chunk_site, mut translation, rotation) in &mut query.iter() {
        let delta_f = forward_walk_vector(&rotation) // Vec3::unit_z().normalize()
			* axis_v
			* options.speed
			* time.delta_seconds;

        let delta_strafe = strafe_vector(rotation) * axis_h * options.speed * time.delta_seconds;

        let delta_float = Vec3::unit_y() * axis_float * options.speed * time.delta_seconds;

        translation.0 += delta_f + delta_strafe + delta_float;
    }
}
