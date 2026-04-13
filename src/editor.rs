//! Editor overlay module - FPS counter, grid toggle, and axis gizmo
//!
//! Provides Unity-style editor overlays using Bevy's UI system.

use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::picking::events::{Drag, DragEnd, DragStart, Pointer};
use bevy::prelude::*;
use bevy::ui::PositionType;

// ─────────────────────────────────────────────────────────────────────────────
// RESOURCE - Editor State
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct EditorOverlayState {
    pub show_grid: bool,
    pub show_axes: bool,
    pub subdivision_level: u32,
    pub subdivision_slider: f32, // 0.0..1.0 maps to level 0..MAX
    pub show_golfball: bool,
}

impl Default for EditorOverlayState {
    fn default() -> Self {
        Self {
            show_grid: false,
            show_axes: true,
            subdivision_level: 1,
            subdivision_slider: 0.2, // starts at level 1
            show_golfball: false,
        }
    }
}

const MAX_SUBDIVISION_LEVEL: u32 = 5;
const MIN_SUBDIVISION_LEVEL: u32 = 0;

/// Tracks whether the user is currently dragging the slider and the cursor x at drag start.
#[derive(Resource)]
pub struct SliderDragState {
    pub dragging: bool,
    pub drag_start_x: f32,
    pub drag_start_slider: f32,
}

impl Default for SliderDragState {
    fn default() -> Self {
        Self {
            dragging: false,
            drag_start_x: 0.0,
            drag_start_slider: 0.2,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MARKERS
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct CoordinateDisplay;

#[derive(Component)]
pub struct SubdivisionDisplay;

#[derive(Component)]
pub struct SubdivisionSlider;

#[derive(Component)]
pub struct SliderFill;

#[derive(Component)]
pub struct SliderTrack;

// ─────────────────────────────────────────────────────────────────────────────
// PLUGIN
// ─────────────────────────────────────────────────────────────────────────────

pub struct EditorOverlayPlugin;

impl bevy::app::Plugin for EditorOverlayPlugin {
    fn build(&self, app: &mut App) {
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
            .init_resource::<SliderDragState>()
            .add_systems(
                Update,
                (
                    button_visual_feedback,
                    draw_grid_gizmo,
                    draw_axis_gizmo,
                    update_coordinate_display,
                    update_subdivision_display,
                    update_slider_fill,
                ),
            );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEMS - Grid
// ─────────────────────────────────────────────────────────────────────────────

fn draw_grid_gizmo(state: Res<EditorOverlayState>, mut gizmos: Gizmos) {
    if !state.show_grid {
        return;
    }

    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let isometry = Isometry3d::new(Vec3::new(0.0, -1.0, 0.0), rotation);
    gizmos.grid(
        isometry,
        UVec2::splat(20),
        Vec2::splat(1.0),
        Color::srgba(0.4, 0.4, 0.4, 0.5),
    );

    let grid_size = 20.0;
    let major_interval = 5.0;
    let half_size = grid_size / 2.0;

    let mut x: f32 = -half_size;
    while x <= half_size {
        if x.abs() > 0.1 {
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
            gizmos.line(
                Vec3::new(-half_size, 0.0, z),
                Vec3::new(half_size, 0.0, z),
                Color::srgb(0.6, 0.6, 0.6),
            );
        }
        z += major_interval;
    }

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

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEMS - Axis
// ─────────────────────────────────────────────────────────────────────────────

fn draw_axis_gizmo(state: Res<EditorOverlayState>, mut gizmos: Gizmos) {
    if !state.show_axes {
        return;
    }

    let origin = Vec3::ZERO;
    let length = 5.0;

    gizmos.line(origin, Vec3::new(length, 0.0, 0.0), Color::srgb(1.0, 0.0, 0.0));
    gizmos.line(origin, Vec3::new(0.0, length, 0.0), Color::srgb(0.0, 1.0, 0.0));
    gizmos.line(origin, Vec3::new(0.0, 0.0, length), Color::srgb(0.0, 0.3, 1.0));
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEMS - Displays
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

fn update_subdivision_display(
    state: Res<EditorOverlayState>,
    mut text: Query<&mut Text, With<SubdivisionDisplay>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    let level = state.subdivision_level;
    let face_count = 20 * 4u64.pow(level);
    let mode = if state.show_golfball { "Golf" } else { "Flat" };
    text.0 = format!("Subdiv: {}  {}  Faces:{}", level, mode, face_count);
}

// ─────────────────────────────────────────────────────────────────────────────
// SLIDER DRAG SYSTEM
// ─────────────────────────────────────────────────────────────────────────────

const SLIDER_TRACK_WIDTH: f32 = 118.0;

fn handle_slider_drag(
    trigger: On<Pointer<Drag>>,
    drag_state: ResMut<SliderDragState>,
    mut state: ResMut<EditorOverlayState>,
) {
    if !drag_state.dragging {
        return;
    }
    // trigger is On<Pointer<Drag>> which derefs to Pointer<Drag>
    // Drag.distance is total distance from drag start (screen pixels)
    let delta = trigger.distance.x;
    let norm = (drag_state.drag_start_slider + delta / SLIDER_TRACK_WIDTH).clamp(0.0, 1.0);
    state.subdivision_slider = norm;
    let n_levels = (MAX_SUBDIVISION_LEVEL - MIN_SUBDIVISION_LEVEL) as f32;
    state.subdivision_level =
        (norm * n_levels).round() as u32 + MIN_SUBDIVISION_LEVEL;
}

fn start_slider_drag(
    trigger: On<Pointer<DragStart>>,
    mut drag_state: ResMut<SliderDragState>,
    state: Res<EditorOverlayState>,
) {
    drag_state.dragging = true;
    // trigger derefs to Pointer<DragStart> which has pointer_location.position
    drag_state.drag_start_x = trigger.pointer_location.position.x;
    drag_state.drag_start_slider = state.subdivision_slider;
}

fn end_slider_drag(
    _trigger: On<Pointer<DragEnd>>,
    mut drag_state: ResMut<SliderDragState>,
) {
    drag_state.dragging = false;
}

// ─────────────────────────────────────────────────────────────────────────────
// SLIDER FILL UPDATE SYSTEM
// ─────────────────────────────────────────────────────────────────────────────

fn update_slider_fill(
    state: Res<EditorOverlayState>,
    mut fills: Query<&mut Node, With<SliderFill>>,
) {
    let Ok(mut fill_node) = fills.single_mut() else {
        return;
    };

    let n_levels = (MAX_SUBDIVISION_LEVEL - MIN_SUBDIVISION_LEVEL) as f32;
    let norm = if n_levels > 0.0 {
        (state.subdivision_level as f32 - MIN_SUBDIVISION_LEVEL as f32) / n_levels
    } else {
        0.0
    };

    let track_width = 118.0; // 120px - 2px rounded ends
    fill_node.width = Val::Px(track_width * norm);
}

// ─────────────────────────────────────────────────────────────────────────────
// OBSERVERS
// ─────────────────────────────────────────────────────────────────────────────

fn toggle_grid(_click: On<Pointer<Click>>, mut state: ResMut<EditorOverlayState>) {
    state.show_grid = !state.show_grid;
}

fn toggle_axes(_click: On<Pointer<Click>>, mut state: ResMut<EditorOverlayState>) {
    state.show_axes = !state.show_axes;
}

fn toggle_golfball(_click: On<Pointer<Click>>, mut state: ResMut<EditorOverlayState>) {
    state.show_golfball = !state.show_golfball;
    bevy::log::info!("Golfball toggled: {}", state.show_golfball);
}

fn inc_subdivision(_click: On<Pointer<Click>>, mut state: ResMut<EditorOverlayState>) {
    if state.subdivision_level < MAX_SUBDIVISION_LEVEL {
        state.subdivision_level += 1;
    }
}

fn dec_subdivision(_click: On<Pointer<Click>>, mut state: ResMut<EditorOverlayState>) {
    if state.subdivision_level > MIN_SUBDIVISION_LEVEL {
        state.subdivision_level -= 1;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEM - Button Visual Feedback
// ─────────────────────────────────────────────────────────────────────────────

fn button_visual_feedback(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<Button>, Changed<Interaction>),
    >,
) {
    for (interaction, mut bg, mut border) in &mut button_query {
        match *interaction {
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.25, 0.45, 0.65));
                *border = BorderColor::all(Color::srgb(0.4, 0.7, 1.0));
            }
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgb(0.15, 0.3, 0.5));
                *border = BorderColor::all(Color::srgb(0.3, 0.6, 0.9));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border = BorderColor::all(Color::srgb(0.4, 0.4, 0.45));
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPAWN HELPER
// ─────────────────────────────────────────────────────────────────────────────

/// Spawns the editor overlay UI into the world
pub fn spawn_editor_overlay(mut commands: Commands) {
    commands.spawn((Camera2d, Camera { order: 1, ..default() }));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12.0),
                top: px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(6.0),
                ..default()
            },
            Name::new("EditorOverlay_TopLeft"),
        ))
        .with_children(|p| {
            // ── Sphere Section ───────────────────────────────────────────────
            p.spawn((
                Text::new("Sphere"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));

            // Subdivision row: [-] Level [+]
            p.spawn((
                Node {
                    width: px(120.0),
                    height: px(26.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Name::new("SubdivisionControls"),
            ))
            .with_children(|row| {
                // Decrement button
                row.spawn((Button, btn_node(), btn_visual()))
                    .with_children(|btn| {
                        btn.spawn((Text::new("-"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.9, 0.9, 0.9))));
                    })
                    .observe(dec_subdivision);

                // Level label
                row.spawn((Text::new("Level"), TextFont { font_size: 11.0, ..default() }, TextColor(Color::srgb(0.6, 0.6, 0.6))));

                // Increment button
                row.spawn((Button, btn_node(), btn_visual()))
                    .with_children(|btn| {
                        btn.spawn((Text::new("+"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.9, 0.9, 0.9))));
                    })
                    .observe(inc_subdivision);
            });

            p.spawn(Node { height: px(6.0), ..default() });

            // Golfball toggle button
            p.spawn((Button, btn_node(), btn_visual()))
                .with_children(|btn| {
                    btn.spawn((Text::new("Golfball"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::srgb(0.9, 0.9, 0.9))));
                })
                .observe(toggle_golfball);

            p.spawn(Node { height: px(4.0), ..default() });

            // ── Subdivision Slider ────────────────────────────────────────────
            p.spawn((
                Text::new("Subdiv Morph"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));

            // Slider row: track + fill
            p.spawn((
                Node {
                    width: px(120.0),
                    height: px(16.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    border: UiRect::all(px(1.0)),
                    border_radius: BorderRadius::all(px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                SubdivisionSlider,
                SliderTrack,
                Name::new("SliderTrack"),
            ))
            .observe(start_slider_drag)
            .observe(handle_slider_drag)
            .observe(end_slider_drag)
            .with_children(|track| {
                // Fill bar
                track.spawn((
                    Node {
                        width: px(0.0),
                        height: px(6.0),
                        margin: UiRect::left(px(1.0)),
                        border_radius: BorderRadius::all(px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.4, 0.7, 1.0)),
                    SliderFill,
                    Name::new("SliderFill"),
                ));
            });

            p.spawn(Node { height: px(6.0), ..default() });

            // ── View Section ─────────────────────────────────────────────────
            p.spawn((
                Text::new("View"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));

            // Toggle Grid
            p.spawn((Button, btn_node(), btn_visual()))
                .with_children(|btn| {
                    btn.spawn((Text::new("Toggle Grid"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::srgb(0.9, 0.9, 0.9))));
                })
                .observe(toggle_grid);

            // Toggle Axes
            p.spawn((Button, btn_node(), btn_visual()))
                .with_children(|btn| {
                    btn.spawn((Text::new("Toggle Axes"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::srgb(0.9, 0.9, 0.9))));
                })
                .observe(toggle_axes);

            p.spawn(Node { height: px(6.0), ..default() });

            // ── Debug Section ───────────────────────────────────────────────
            p.spawn((
                Text::new("Debug"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));

            // Coordinate display
            p.spawn((
                CoordinateDisplay,
                Text::new("X: 0.00  Y: 0.00  Z: 0.00"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Subdivision level display
            p.spawn((
                SubdivisionDisplay,
                Text::new("Subdiv: 1  Flat  Faces:80"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn btn_node() -> Node {
    Node {
        width: px(120.0),
        height: px(26.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(px(4.0)),
        border: UiRect::all(px(1.0)),
        border_radius: BorderRadius::all(px(4.0)),
        ..default()
    }
}

fn btn_visual() -> (BackgroundColor, BorderColor) {
    (
        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
        BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
    )
}
