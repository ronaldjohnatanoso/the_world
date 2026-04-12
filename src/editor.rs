//! Editor overlay module - FPS counter, grid toggle, and axis gizmo
//!
//! Provides Unity-style editor overlays using Bevy's UI system.

use bevy::prelude::*;
use bevy::ui::PositionType;

// ─────────────────────────────────────────────────────────────────────────────
// RESOURCE - Editor State
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct EditorOverlayState {
    pub show_grid: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// COMPONENT - UI Widgets
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
struct GridToggleButton;

// ─────────────────────────────────────────────────────────────────────────────
// PLUGIN
// ─────────────────────────────────────────────────────────────────────────────

pub struct EditorOverlayPlugin;

impl bevy::app::Plugin for EditorOverlayPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<EditorOverlayState>();
    }
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
            // ── FPS Counter (always visible) ─────────────────────────────────
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(px(8.0)),
                        border_radius: BorderRadius::all(px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    Name::new("FpsDisplay"),
                ))
                .with_children(|fps_parent| {
                    fps_parent.spawn((
                        Text::new("FPS: --"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

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
                    GridToggleButton,
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
                .observe(
                    |_click: On<Pointer<Click>>, mut state: ResMut<EditorOverlayState>| {
                        state.show_grid = !state.show_grid;
                        info!("Grid toggled: {}", state.show_grid);
                    },
                );
        });

    // ── Axis Gizmo (Bottom-Left) ─────────────────────────────────────────────
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12.0),
                bottom: px(12.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Name::new("AxisGizmo"),
        ))
        .with_children(|parent| {
            // Row with X axis
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(4.0),
                        ..default()
                    },
                    Name::new("AxisRowX"),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: px(40.0),
                            height: px(3.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
                    ));
                    row.spawn((
                        Text::new("X"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.2, 0.2)),
                    ));
                });

            // Row with Y axis
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(4.0),
                        ..default()
                    },
                    Name::new("AxisRowY"),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: px(3.0),
                            height: px(40.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.9, 0.2)),
                    ));
                    row.spawn((
                        Text::new("Y"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.9, 0.2)),
                    ));
                });

            // Row with Z axis
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(4.0),
                        ..default()
                    },
                    Name::new("AxisRowZ"),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: px(30.0),
                            height: px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.4, 0.9)),
                    ));
                    row.spawn((
                        Text::new("Z"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.4, 0.9)),
                    ));
                });
        });
}