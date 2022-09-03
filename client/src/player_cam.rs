use bevy::{
	input::{Input, mouse::MouseMotion},
	math::{Quat, Vec2, Vec3},
	prelude::*
};

/* Heavily based off https://github.com/mcpar-land/bevy_fly_camera/ */

#[derive(Component)]
pub struct PlayerCamera {
    /// Camera acceleration speed. Default: `1.0`
    pub accel: f32,

    /// Maximum movement speed for the PlayerCamera. Default: `0.5`
	pub max_speed: f32,
	/// The sensitivity of the PlayerCamera's motion, based on mouse movement. Default: `3.0`
	pub sensitivity: f32,
	/// The amount of deceleration to apply to the camera's motion. Default: `1.0`
	pub friction: f32,
	/// The current pitch of the PlayerCamera in degrees. This value is always up-to-date, enforced by [PlayerCameraPlugin](struct.PlayerCameraPlugin.html)
	pub pitch: f32,
	/// The current pitch of the PlayerCamera in degrees. This value is always up-to-date, enforced by [PlayerCameraPlugin](struct.PlayerCameraPlugin.html)
	pub yaw: f32,
	/// The current velocity of the PlayerCamera. This value is always up-to-date, enforced by [PlayerCameraPlugin](struct.PlayerCameraPlugin.html)
	pub velocity: Vec3,
    /// Key used to move forward. Default: <kbd>W</kbd>
	pub key_forward: KeyCode,
	/// Key used to move backward. Default: <kbd>S</kbd>
	pub key_backward: KeyCode,
	/// Key used to move left. Default: <kbd>A</kbd>
	pub key_left: KeyCode,
	/// Key used to move right. Default: <kbd>D</kbd>
	pub key_right: KeyCode,
	/// Key used to move up. Default: <kbd>Space</kbd>
	pub key_up: KeyCode,
	/// Key used to move forward. Default: <kbd>LShift</kbd>
	pub key_down: KeyCode,
	/// If `false`, disable keyboard control of the camera. Default: `true`
	pub enabled: bool,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            accel: 1.,
            max_speed: 0.5,
            sensitivity: 3.,
            friction: 1.,
            pitch: 0.,
            yaw: 0.,
            velocity: Vec3::ZERO,
            key_forward: KeyCode::W,
			key_backward: KeyCode::S,
			key_left: KeyCode::A,
			key_right: KeyCode::D,
			key_up: KeyCode::Space,
			key_down: KeyCode::LShift,
            enabled: true
        }
    }
}

fn forward_vector(rotation: &Quat) -> Vec3 {
	rotation.mul_vec3(Vec3::Z).normalize()
}

fn forward_walk_vector(rotation: &Quat) -> Vec3 {
	let f = forward_vector(rotation);
	let f_flattened = Vec3::new(f.x, 0.0, f.z).normalize();
	f_flattened
}

fn strafe_vector(rotation: &Quat) -> Vec3 {
	// Rotate it 90 degrees to get the strafe direction
	Quat::from_rotation_y(90.0f32.to_radians())
		.mul_vec3(forward_walk_vector(rotation))
		.normalize()
}

fn camera_movement_system(
	time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
	mut query: Query<(&mut PlayerCamera, &mut Transform)>,
) {
    for (mut options, mut transform) in query.iter_mut() {
		let (axis_h, axis_v, axis_float) = if options.enabled {
			(
				movement_axis(&keyboard_input, options.key_right, options.key_left),
				movement_axis(
					&keyboard_input,
					options.key_backward,
					options.key_forward,
				),
				movement_axis(&keyboard_input, options.key_up, options.key_down),
			)
		} else {
			(0.0, 0.0, 0.0)
		};

		let rotation = transform.rotation;
		let accel: Vec3 = (strafe_vector(&rotation) * axis_h)
			+ (forward_walk_vector(&rotation) * axis_v)
			+ (Vec3::Y * axis_float);
		let accel: Vec3 = if accel.length() != 0.0 {
			accel.normalize() * options.accel
		} else {
			Vec3::ZERO
		};

		let friction: Vec3 = if options.velocity.length() != 0.0 {
			options.velocity.normalize() * -1.0 * options.friction
		} else {
			Vec3::ZERO
		};

		options.velocity += accel * time.delta_seconds();

		// clamp within max speed
		if options.velocity.length() > options.max_speed {
			options.velocity = options.velocity.normalize() * options.max_speed;
		}

		let delta_friction = friction * time.delta_seconds();

		options.velocity = if (options.velocity + delta_friction).signum()
			!= options.velocity.signum()
		{
			Vec3::ZERO
		} else {
			options.velocity + delta_friction
		};

		transform.translation += options.velocity;
	}
}

fn mouse_motion_system(
	time: Res<Time>,
	mut mouse_motion_event_reader: EventReader<MouseMotion>,
	mut query: Query<(&mut PlayerCamera, &mut Transform)>,
) {
	let mut delta: Vec2 = Vec2::ZERO;
    
	for event in mouse_motion_event_reader.iter() {
		delta += event.delta;
	}
	if delta.is_nan() {
		return;
	}

	for (mut options, mut transform) in query.iter_mut() {
        if !options.enabled {
			continue;
		}

		options.yaw -= delta.x * options.sensitivity * time.delta_seconds();

		if options.yaw > 180. {
			options.yaw = -180.;
		} else if options.yaw < -180. {
			options.yaw = 180.
		}

		options.pitch += delta.y * options.sensitivity * time.delta_seconds();

		options.pitch = options.pitch.clamp(-89.0, 89.9);
		// println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

		let yaw_radians = options.yaw.to_radians();
		let pitch_radians = options.pitch.to_radians();

		transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
			* Quat::from_axis_angle(-Vec3::X, pitch_radians);
	}
}

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(camera_movement_system)
			.add_system(mouse_motion_system);
	}
}

pub fn movement_axis(
	input: &Res<Input<KeyCode>>,
	plus: KeyCode,
	minus: KeyCode,
) -> f32 {
	let mut axis = 0.0;
	if input.pressed(plus) {
		axis += 1.0;
	}
	if input.pressed(minus) {
		axis -= 1.0;
	}
	axis
}