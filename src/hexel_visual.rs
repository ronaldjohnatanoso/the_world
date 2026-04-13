//! Hexel boundary visualization — wireframe overlay for geodesic sphere
//!
//! Draws the subdivided triangular mesh as a wireframe, revealing the hex/pent
//! cell boundaries of the dual graph.

use bevy::prelude::*;
use crate::subdivision::subdivide_icosahedron;

/// Wireframe line color — bright enough to see on sphere
const WIREFRAME_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);

/// Draws all edges of the subdivided triangular mesh as a wireframe.
/// The hexel cell boundaries are visible as the lines that form groups of 3
/// (around hexagons) or 5 (around pentagons at the 12 icosahedron vertices).
pub fn draw_dual_graph_gizmo(
    subdivision_level: u32,
    gizmos: &mut Gizmos,
) {
    let (verts, faces) = subdivide_icosahedron(subdivision_level);

    // Draw every edge of every triangle — shows the full mesh wireframe
    for face in &faces {
        let p0 = verts[face[0] as usize];
        let p1 = verts[face[1] as usize];
        let p2 = verts[face[2] as usize];

        // Edge 0-1
        gizmos.line(p0, p1, WIREFRAME_COLOR);
        // Edge 1-2
        gizmos.line(p1, p2, WIREFRAME_COLOR);
        // Edge 2-0
        gizmos.line(p2, p0, WIREFRAME_COLOR);
    }
}
