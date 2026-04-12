//! Editor overlay module - FPS counter, grid toggle, and axis gizmo
//!
//! Provides Unity-style editor overlays using Bevy's UI system.

use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::prelude::*;
use bevy::ui::PositionType;

// ─────────────────────────────────────────────────────────────────────────────
// RESOURCE - Editor State
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct EditorOverlayState {
    pub show_grid: bool,
    pub show_axes: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// MARKER - Coordinate Display UI Node
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct CoordinateDisplay;

// ─────────────────────────────────────────────────────────────────────────────
// PLUGIN
// ─────────────────────────────────────────────────────────────────────────────

pub struct EditorOverlayPlugin;

impl bevy::app::Plugin for EditorOverlayPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // Add FPS overlay
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 14.0,
                    ..default()
                },
                text_color: Color::srgb(0.9, 0.9, 0.9),
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
                frame_time_graph_config: bevy::dev_tools::fps_overlay::FrameTimeGraphConfig {
                    enabled: false,
                    min_fps: 30.0,
                    target_fps: 144.0,
                },
            },
        });

        app.init_resource::<EditorOverlayState>()
            .add_systems(Update, (draw_grid_gizmo, draw_axis_gizmo, update_coordinate_display));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEM - Draw Grid Gizmo
// ─────────────────────────────────────────────────────────────────────────────

fn draw_grid_gizmo(
    state: Res<EditorOverlayState>,
    mut gizmos: Gizmos,
) {
    if state.show_grid {
        // Draw a 20x20 grid on the XZ plane at Y = -1 (flat on ground)
        // Default grid is on XY plane, so rotate 90 degrees around X to make it flat
        let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        let isometry = Isometry3d::new(Vec3::new(0.0, -1.0, 0.0), rotation);
        gizmos.grid(
            isometry,
            UVec2::splat(20),
            Vec2::splat(1.0),
            Color::srgba(0.4, 0.4, 0.4, 0.5),
        );

        // Draw thicker lines every 5 units for scale reference
        let grid_size = 20.0;
        let major_interval = 5.0;
        let half_size = grid_size / 2.0;

        // Major grid lines (thicker, brighter)
        let mut x: f32 = -half_size;
        while x <= half_size {
            if x.abs() > 0.1 {
                // Vertical lines (along Z)
                gizmos.line(
                    Vec3::new(x, 0.0, -half_size),
                    Vec3::new(x, 0.0, half_size),
                    Color::srgb(0.6, 0.6, 0.6),
                );
            }
            x += major_interval;
        }

        let mut z: f32 = -half_size;
        while z <= half_size {
            if z.abs() > 0.1 {
                // Horizontal lines (along X)
                gizmos.line(
                    Vec3::new(-half_size, 0.0, z),
                    Vec3::new(half_size, 0.0, z),
                    Color::srgb(0.6, 0.6, 0.6),
                );
            }
            z += major_interval;
        }

        // Origin cross (highlight X and Z axes at origin)
        gizmos.line(
            Vec3::new(0.0, 0.001, -half_size),
            Vec3::new(0.0, 0.001, half_size),
            Color::srgb(0.8, 0.8, 0.8),
        );
        gizmos.line(
            Vec3::new(-half_size, 0.001, 0.0),
            Vec3::new(half_size, 0.001, 0.0),
            Color::srgb(0.8, 0.8, 0.8),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OBSERVER - Grid Toggle
// ─────────────────────────────────────────────────────────────────────────────

fn toggle_grid(
    _click: On<Pointer<Click>>,
    mut state: ResMut<EditorOverlayState>,
) {
    state.show_grid = !state.show_grid;
    info!("Grid toggled: {}", state.show_grid);
}

fn toggle_axes(
    _click: On<Pointer<Click>>,
    mut state: ResMut<EditorOverlayState>,
) {
    state.show_axes = !state.show_axes;
    info!("Axes toggled: {}", state.show_axes);
}

// ─────────────────────────────────────────────────────────────────────────────
// SPAWN HELPER - Creates the editor overlay UI
// ─────────────────────────────────────────────────────────────────────────────

/// Spawns the editor overlay UI into the world
pub fn spawn_editor_overlay(mut commands: Commands) {
    commands.spawn((Camera2d, Camera { order: 1, ..default() }));

    // ── Main Editor UI Container (Top-Left) ──────────────────────────────────
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12.0),
                top: px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(8.0),
                ..default()
            },
            Name::new("EditorOverlay_TopLeft"),
        ))
        .with_children(|parent| {
            // ── Grid Toggle Button ─────────────────────────────────────────────
            parent
                .spawn((
                    Button,
                    Node {
                        width: px(120.0),
                        height: px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(px(4.0)),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(4.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                ))
                .with_children(|btn_child| {
                    btn_child.spawn((
                        Text::new("Toggle Grid"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                })
                .observe(toggle_grid);

            // ── Axes Toggle Button ───────────────────────────────────────────
            parent
                .spawn((
                    Button,
                    Node {
                        width: px(120.0),
                        height: px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(px(4.0)),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(4.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                ))
                .with_children(|btn_child| {
                    btn_child.spawn((
                        Text::new("Toggle Axes"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                })
                .observe(toggle_axes);

            // ── Coordinate Display ───────────────────────────────────────────
            parent
                .spawn((
                    CoordinateDisplay,
                    Text::new("X: 0.00  Y: 0.00  Z: 0.00"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEM - Draw Axis Gizmo in 3D
// ─────────────────────────────────────────────────────────────────────────────

fn draw_axis_gizmo(
    state: Res<EditorOverlayState>,
    mut gizmos: Gizmos,
) {
    if !state.show_axes {
        return;
    }

    let origin = Vec3::ZERO;
    let length = 5.0;

    // X axis - Bright Red (positive X direction)
    gizmos.line(origin, Vec3::new(length, 0.0, 0.0), Color::srgb(1.0, 0.0, 0.0));

    // Y axis - Bright Green (positive Y direction)
    gizmos.line(origin, Vec3::new(0.0, length, 0.0), Color::srgb(0.0, 1.0, 0.0));

    // Z axis - Bright Blue (positive Z direction)
    gizmos.line(origin, Vec3::new(0.0, 0.0, length), Color::srgb(0.0, 0.3, 1.0));
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEM - Update Coordinate Display
// ─────────────────────────────────────────────────────────────────────────────

fn update_coordinate_display(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut coord_text: Query<&mut Text, With<CoordinateDisplay>>,
) {
    let Ok(transform) = camera_query.single() else {
        return;
    };
    let Ok(mut text) = coord_text.single_mut() else {
        return;
    };

    let pos = transform.translation;
    text.0 = format!("X: {:.2}  Y: {:.2}  Z: {:.2}", pos.x, pos.y, pos.z);
}