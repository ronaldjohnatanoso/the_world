//! The World — Voxel Sphere World
//!
//! A 3D voxel sphere world using Bevy's mesh system.
//! CPU-side octree traversal feeds into Bevy's Mesh API for rendering.

mod camera;
mod editor;

use bevy::prelude::*;
use camera::{setup_camera, CameraPlugin};
use editor::{spawn_editor_overlay, EditorOverlayPlugin};

// ─────────────────────────────────────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(EditorOverlayPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ── Cube Mesh ────────────────────────────────────────────────────────────
    let mesh = Mesh::from(Cuboid::from_size(Vec3::splat(2.0))); // 2x2x2 cube

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // ── Ground Plane ─────────────────────────────────────────────────────────
    let ground_mesh = Mesh::from(Plane3d::default().mesh().size(20.0, 20.0));
    commands.spawn((
        Mesh3d(meshes.add(ground_mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))), // green ground
        Transform::from_xyz(0.0, -1.0, 0.0),                          // below the cube
    ));

    // ── Camera ───────────────────────────────────────────────────────────────
    setup_camera(
        &mut commands,
        Vec3::new(0.0, 2.0, 8.0), // further back to see the whole cube
        Vec3::ZERO,               // looking at origin
    );

    // ── Light ──────────────────────────────────────────────────────────────
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // ── Editor Overlay UI ──────────────────────────────────────────────────
    spawn_editor_overlay(commands);
}
