//! Camera module - FPS-style controllable camera
//!
//! Provides camera controls: WASD + mouse look (on click-hold), scroll zoom.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// COMPONENT
// ─────────────────────────────────────────────────────────────────────────────

/// Camera controller: WASD movement + mouse look (on click-hold) + scroll zoom
#[derive(Component)]
pub struct CameraController {
    /// Movement speed in units per second
    pub move_speed: f32,
    /// Scroll zoom sensitivity
    pub zoom_speed: f32,
    /// Mouse sensitivity for look
    pub mouse_sensitivity: f32,
    /// Is mouse look active (right button held)
    is_looking: bool,
    /// Pitch angle (up/down look) in radians
    pitch: f32,
    /// Yaw angle (left/right look) in radians
    yaw: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            zoom_speed: 30.0,
            mouse_sensitivity: 0.002,
            is_looking: false,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PLUGIN
// ─────────────────────────────────────────────────────────────────────────────

pub struct CameraPlugin;

impl bevy::app::Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, camera_control);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEM
// ─────────────────────────────────────────────────────────────────────────────

/// Camera control system - runs every frame
/// WASD movement, mouse look (when right button held), scroll to zoom
fn camera_control(
    mut camera_query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut controller)) = camera_query.single_mut() else {
        return;
    };

    // ── Mouse Look (only when right button held) ───────────────────────────
    // Update looking state
    if mouse_input.just_pressed(MouseButton::Right) {
        controller.is_looking = true;
    } else if mouse_input.just_released(MouseButton::Right) {
        controller.is_looking = false;
    }

    // Apply mouse look if looking
    if controller.is_looking {
        let mouse_delta = mouse_motion.delta;

        controller.yaw -= mouse_delta.x * controller.mouse_sensitivity;
        controller.pitch -= mouse_delta.y * controller.mouse_sensitivity;

        // Clamp pitch to prevent flipping
        controller.pitch = controller.pitch.clamp(-1.54, 1.54);
    }

    // Apply rotation using Euler angles (YXZ order for FPS-style)
    transform.rotation =
        Quat::from_euler(bevy::math::EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);

    // ── WASD Movement ─────────────────────────────────────────────────────
    let mut velocity = Vec3::ZERO;

    // Get camera's local axes (camera faces -Z by default)
    let local_forward = transform.rotation * -Vec3::Z;
    let local_right = transform.rotation * Vec3::X;

    // Project onto horizontal plane for ground-relative movement
    let forward_horizontal = Vec3::new(local_forward.x, 0.0, local_forward.z).normalize();
    let right_horizontal = Vec3::new(local_right.x, 0.0, local_right.z).normalize();

    if key_input.pressed(KeyCode::KeyW) {
        velocity += forward_horizontal;
    }
    if key_input.pressed(KeyCode::KeyS) {
        velocity -= forward_horizontal;
    }
    if key_input.pressed(KeyCode::KeyA) {
        velocity -= right_horizontal;
    }
    if key_input.pressed(KeyCode::KeyD) {
        velocity += right_horizontal;
    }

    // Normalize and apply movement
    if velocity.length() > 0.0 {
        velocity = velocity.normalize() * controller.move_speed * time.delta_secs();
        transform.translation += velocity;
    }

    // ── Scroll Zoom ────────────────────────────────────────────────────────
    let zoom_delta = scroll.delta.y * controller.zoom_speed * time.delta_secs();
    if zoom_delta.abs() > 0.001 {
        // Move along the camera's local Z axis (forward/backward)
        let zoom_vector = transform.rotation * Vec3::Z * -zoom_delta;
        transform.translation += zoom_vector;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SETUP HELPER
// ─────────────────────────────────────────────────────────────────────────────

/// Spawns a camera at the given position looking at target
pub fn setup_camera(
    commands: &mut Commands,
    position: Vec3,
    looking_at: Vec3,
) {
    commands.spawn((
        CameraController::default(),
        Camera3d::default(),
        Transform::from_translation(position).looking_at(looking_at, Vec3::Y),
    ));
}
