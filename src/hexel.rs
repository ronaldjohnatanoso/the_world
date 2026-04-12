//! Hexel module - hexagonal voxel primitives
//!
//! A hexel is a hexagonal prism with 6-bit collapse masks (64 states).
//! This module defines the hexel geometry and mesh generation.

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

// ─────────────────────────────────────────────────────────────────────────────
// CONSTANTS
// ─────────────────────────────────────────────────────────────────────────────

/// Number of vertices on the hexagon face
pub const HEXAGON_SIDES: usize = 6;

/// Radius of the hexel (distance from center to vertex)
pub const HEXEL_RADIUS: f32 = 0.5;

/// Height/depth of the hexel prism
pub const HEXEL_HEIGHT: f32 = 0.5;

/// Angle between each vertex in a regular hexagon
const HEX_ANGLE_STEP: f32 = std::f32::consts::TAU / HEXAGON_SIDES as f32;

/// Starting angle offset (points "up" at 0, then clockwise)
const HEX_START_ANGLE: f32 = std::f32::consts::FRAC_PI_2; // 90 degrees - point up

// ─────────────────────────────────────────────────────────────────────────────
// HEXEL GEOMETRY
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the 2D positions of hexagon vertices (centered at origin)
pub fn hexagon_vertices(radius: f32) -> Vec<Vec2> {
    (0..HEXAGON_SIDES)
        .map(|i| {
            let angle = HEX_START_ANGLE + i as f32 * HEX_ANGLE_STEP;
            Vec2::new(
                radius * angle.cos(),
                radius * angle.sin(),
            )
        })
        .collect()
}

/// Creates a Bevy Mesh for a hexel prism
pub fn create_hexel_mesh() -> Mesh {
    let radius = HEXEL_RADIUS;
    let height = HEXEL_HEIGHT;
    let hex_2d = hexagon_vertices(radius);
    let half_height = height / 2.0;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Build the hex prism as triangles (not quads)

    // Top face - triangle fan (pointing up +Z)
    // Center vertex
    positions.push([0.0, 0.0, half_height]);
    normals.push([0.0, 0.0, 1.0]);

    // Ring vertices (CCW when viewed from above)
    for v in &hex_2d {
        positions.push([v.x, v.y, half_height]);
        normals.push([0.0, 0.0, 1.0]);
    }

    // Top face triangles (fan from center)
    for i in 0..HEXAGON_SIDES {
        let center = 0u32;
        let i0 = (1 + i) as u32;
        let i1 = (1 + (i + 1) % HEXAGON_SIDES) as u32;
        indices.push(center);
        indices.push(i0);
        indices.push(i1);
    }

    // Bottom face - triangle fan (pointing down -Z)
    // Center vertex
    positions.push([0.0, 0.0, -half_height]);
    normals.push([0.0, 0.0, -1.0]);

    // Ring vertices (CW when viewed from below - reversed winding for correct normals)
    for v in &hex_2d {
        positions.push([v.x, v.y, -half_height]);
        normals.push([0.0, 0.0, -1.0]);
    }

    let bottom_center_idx = (HEXAGON_SIDES + 2) as u32;
    let bottom_ring_start = bottom_center_idx + 1;

    // Bottom face triangles (fan from center, reversed winding)
    for i in 0..HEXAGON_SIDES {
        let center = bottom_center_idx;
        let i0 = bottom_ring_start + i as u32;
        let i1 = bottom_ring_start + ((i + 1) % HEXAGON_SIDES) as u32;
        indices.push(center);
        indices.push(i1); // Reversed!
        indices.push(i0);
    }

    // Side faces - 2 triangles per side
    for i in 0..HEXAGON_SIDES {
        let v0 = hex_2d[i];
        let v1 = hex_2d[(i + 1) % HEXAGON_SIDES];

        // Compute outward normal for this side (in XY plane, pointing away from center)
        let normal = Vec3::new(v0.x + v1.x, v0.y + v1.y, 0.0).normalize();

        let base_idx = positions.len() as u32;

        // 4 vertices of the side quad
        // Bottom-left, bottom-right, top-right, top-left (when viewed from outside)
        positions.push([v0.x, v0.y, -half_height]); // 0: bottom i
        normals.push([normal.x, normal.y, normal.z]);
        positions.push([v1.x, v1.y, -half_height]); // 1: bottom i+1
        normals.push([normal.x, normal.y, normal.z]);
        positions.push([v1.x, v1.y, half_height]); // 2: top i+1
        normals.push([normal.x, normal.y, normal.z]);
        positions.push([v0.x, v0.y, half_height]); // 3: top i
        normals.push([normal.x, normal.y, normal.z]);

        // Two triangles for this quad (CCW when viewed from outside)
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);

        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    // Create mesh
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}
