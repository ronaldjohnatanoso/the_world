//! Hexel module - hexagonal voxel primitives
//!
//! A hexel is a hexagonal prism with 6-bit collapse masks (64 states).
//! This module defines the hexel geometry and mesh generation.

use bevy::prelude::*;

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

    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Center vertices for bottom and top faces
    let bottom_center_idx = 0u32;
    positions.push(Vec3::new(0.0, 0.0, -half_height));
    normals.push(Vec3::NEG_Z);

    let top_center_idx = (HEXAGON_SIDES + 1) as u32;
    positions.push(Vec3::new(0.0, 0.0, half_height));
    normals.push(Vec3::Z);

    // Bottom ring vertices
    for v in &hex_2d {
        positions.push(Vec3::new(v.x, v.y, -half_height));
        normals.push(Vec3::NEG_Z);
    }

    // Top ring vertices
    for v in &hex_2d {
        positions.push(Vec3::new(v.x, v.y, half_height));
        normals.push(Vec3::Z);
    }

    // Bottom face (fan from center)
    for i in 0..HEXAGON_SIDES {
        let i0 = 1 + i;
        let i1 = 1 + (i + 1) % HEXAGON_SIDES;
        indices.push(bottom_center_idx);
        indices.push(i0 as u32);
        indices.push(i1 as u32);
    }

    // Top face (fan from center, reversed winding)
    for i in 0..HEXAGON_SIDES {
        let i0 = 1 + HEXAGON_SIDES + i;
        let i1 = 1 + HEXAGON_SIDES + (i + 1) % HEXAGON_SIDES;
        indices.push(top_center_idx);
        indices.push(i1 as u32);
        indices.push(i0 as u32);
    }

    // Side faces
    for i in 0..HEXAGON_SIDES {
        let bi0 = 1 + i;
        let bi1 = 1 + (i + 1) % HEXAGON_SIDES;
        let ti0 = 1 + HEXAGON_SIDES + i;
        let ti1 = 1 + HEXAGON_SIDES + (i + 1) % HEXAGON_SIDES;

        // Compute outward normal for this side
        let v0 = hex_2d[i];
        let v1 = hex_2d[(i + 1) % HEXAGON_SIDES];
        let edge_mid = (v0 + v1) / 2.0;
        let normal = Vec3::new(edge_mid.x, edge_mid.y, 0.0).normalize();

        // Add 4 vertices for this side quad
        let base_idx = positions.len() as u32;

        let bottom_i = Vec3::new(v0.x, v0.y, -half_height);
        let bottom_ip1 = Vec3::new(v1.x, v1.y, -half_height);
        let top_ip1 = Vec3::new(v1.x, v1.y, half_height);
        let top_i = Vec3::new(v0.x, v0.y, half_height);

        positions.push(bottom_i);
        normals.push(normal);
        positions.push(bottom_ip1);
        normals.push(normal);
        positions.push(top_ip1);
        normals.push(normal);
        positions.push(top_i);
        normals.push(normal);

        // Two triangles for quad
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    // Create mesh using Mesh::from()
    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}
