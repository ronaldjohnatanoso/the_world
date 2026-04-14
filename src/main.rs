//! The World — Voxel Sphere World
//!
//! A 3D voxel sphere world using Bevy's mesh system.
//! CPU-side octree traversal feeds into Bevy's Mesh API for rendering.

mod camera;
mod editor;
mod icosahedron;
mod subdivision;
mod hexel;
mod hexel_visual;

use bevy::prelude::*;
use camera::{setup_camera, CameraPlugin};
use editor::{spawn_editor_overlay, EditorOverlayPlugin, EditorOverlayState};
use hexel::create_hexel_sphere_mesh;
use hexel_visual::draw_dual_graph_gizmo;
use subdivision::{create_subdivided_mesh, subdivide_icosahedron};

// ─────────────────────────────────────────────────────────────────────────────
// STATE RESOURCE - Sphere mesh state
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Resource)]
struct SphereState {
    level: u32,
    golfball: bool,
    entity: Option<Entity>,
}

impl Default for SphereState {
    fn default() -> Self {
        Self {
            level: 1,
            golfball: false,
            entity: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();

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
        .init_resource::<SphereState>()
        .add_systems(Startup, setup)
        .add_systems(Update, draw_dual_graph_gizmo_system)
        .add_systems(Update, update_sphere_if_needed)
        .run();
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEMS
// ─────────────────────────────────────────────────────────────────────────────

fn spawn_sphere(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    level: u32,
    golfball: bool,
) -> Entity {
    let mesh = if golfball {
        let (verts, faces) = subdivide_icosahedron(level);
        create_hexel_sphere_mesh(level)
    } else {
        create_subdivided_mesh(level)
    };

    let color = if golfball {
        Color::srgb(0.85, 0.85, 0.85) // Golfball white-ish
    } else {
        Color::srgb(0.4, 0.5, 0.6) // Subdivided sphere blue-ish
    };

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(color)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sphere_state: ResMut<SphereState>,
) {
    let entity = spawn_sphere(
        &mut commands,
        &mut meshes,
        &mut materials,
        sphere_state.level,
        sphere_state.golfball,
    );
    sphere_state.entity = Some(entity);

    // ── Camera ───────────────────────────────────────────────────────────────
    setup_camera(
        &mut commands,
        Vec3::new(4.0, 4.0, 6.0),
        Vec3::ZERO,
    );

    // ── Light ──────────────────────────────────────────────────────────────
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        AmbientLight {
            brightness: 0.3,
            ..default()
        },
    ));

    // ── Editor Overlay UI ──────────────────────────────────────────────────
    spawn_editor_overlay(commands);
}

/// Check if sphere needs to be respawned (level or mode changed)
fn update_sphere_if_needed(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor_state: Res<EditorOverlayState>,
    mut sphere_state: ResMut<SphereState>,
) {
    let needs_respawn =
        editor_state.subdivision_level != sphere_state.level
        || editor_state.show_golfball != sphere_state.golfball;

    if needs_respawn {
        if let Some(entity) = sphere_state.entity {
            commands.entity(entity).despawn();
        }

        sphere_state.level = editor_state.subdivision_level;
        sphere_state.golfball = editor_state.show_golfball;

        let entity = spawn_sphere(
            &mut commands,
            &mut meshes,
            &mut materials,
            sphere_state.level,
            sphere_state.golfball,
        );
        sphere_state.entity = Some(entity);

        info!(
            "Sphere respawned: level={} golfball={}",
            sphere_state.level, sphere_state.golfball
        );
    }
}

/// Draws the dual graph wireframe (when NOT in golfball mode)
fn draw_dual_graph_gizmo_system(
    mut gizmos: Gizmos,
    editor_state: Res<EditorOverlayState>,
) {
    // Only show wireframe when not in golfball mode
    if !editor_state.show_golfball {
        draw_dual_graph_gizmo(editor_state.subdivision_level, &mut gizmos);
    }
}
