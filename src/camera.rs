//! Camera module - WASD + scroll zoom + click-hold pan camera
//!
//! Provides camera controls: WASD movement, scroll to zoom, click-hold to pan.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// COMPONENT
// ─────────────────────────────────────────────────────────────────────────────

/// Camera controller: WASD + scroll zoom + click-hold pan
#[derive(Component)]
pub struct CameraController {
    /// Movement speed in units per second
    pub move_speed: f32,
    /// Scroll zoom sensitivity
    pub zoom_speed: f32,
    /// Pan speed when mouse button is held
    pub pan_speed: f32,
    /// Is the left mouse button currently held for panning
    is_panning: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            zoom_speed: 10.0,
            pan_speed: 0.05,
            is_panning: false,
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
/// WASD movement, scroll to zoom, click-hold to pan
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
    if zoom_delta != 0.0 {
        // Move along the camera's local Z axis (forward/backward)
        let zoom_vector = transform.rotation * Vec3::Z * -zoom_delta;
        transform.translation += zoom_vector;
    }

    // ── Click-Hold Pan ────────────────────────────────────────────────────
    // Detect pan start/end
    if mouse_input.just_pressed(MouseButton::Left) {
        controller.is_panning = true;
    } else if mouse_input.just_released(MouseButton::Left) {
        controller.is_panning = false;
    }

    // Pan when holding left mouse button
    if controller.is_panning {
        let mouse_delta = mouse_motion.delta;
        let pan_x = -mouse_delta.x * controller.pan_speed;
        let pan_y = mouse_delta.y * controller.pan_speed;

        // Pan in camera's local X/Y plane (horizontal and vertical relative to view)
        let pan_right = transform.rotation * Vec3::X * pan_x;
        let pan_up = transform.rotation * Vec3::Y * pan_y;

        transform.translation += pan_right + pan_up;
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
