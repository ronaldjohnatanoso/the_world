//! The World — Voxel Sphere World
//!
//! A 3D voxel sphere world using Bevy's mesh system.
//! CPU-side octree traversal feeds into Bevy's Mesh API for rendering.

mod camera;
mod editor;
mod hexel;

use bevy::prelude::*;
use camera::{setup_camera, CameraPlugin};
use editor::{spawn_editor_overlay, EditorOverlayPlugin};
use hexel::create_hexel_mesh;

// ─────────────────────────────────────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();

    // Configure window for higher FPS (disable vsync) and fullscreen
    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The World".to_string(),
                present_mode: bevy::window::PresentMode::Immediate,
                mode: bevy::window::WindowMode::BorderlessFullscreen(
                    bevy::window::MonitorSelection::Primary,
                ),
                resolution: bevy::window::WindowResolution::new(1920.0 as u32, 1080.0 as u32),
                ..default()
            }),
            ..default()
        }),
    );

    app.add_plugins((CameraPlugin, EditorOverlayPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ── Single Hexel ────────────────────────────────────────────────────────
    let hexel_mesh = create_hexel_mesh();

    commands.spawn((
        Mesh3d(meshes.add(hexel_mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // ── Camera ───────────────────────────────────────────────────────────────
    // Position to see the hexel from isometric-ish angle
    setup_camera(
        &mut commands,
        Vec3::new(4.0, 4.0, 6.0), // angled view to see hexel shape
        Vec3::ZERO,                  // looking at origin
    );

    // ── Light ──────────────────────────────────────────────────────────────
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add ambient light so we can see all faces
    commands.spawn((
        AmbientLight {
            brightness: 0.3,
            ..default()
        },
    ));

    // ── Editor Overlay UI ──────────────────────────────────────────────────
    spawn_editor_overlay(commands);
}
