//! Icosahedron module - base geometry for geodesic sphere
//!
//! An icosahedron has exactly:
//! - 12 vertices
//! - 30 edges
//! - 20 triangular faces

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

// Golden ratio — key to icosahedron geometry
const PHI: f32 = 1.618033988749895;

/// Returns the 12 vertices of a unit icosahedron (normalized to radius 1)
pub fn icosahedron_vertices() -> Vec<Vec3> {
    // An icosahedron is built from 3 mutually perpendicular golden rectangles
    // Each rectangle has dimensions 1 × PHI
    // We normalize each vertex to sit on the unit sphere
    let raw = vec![
        // Rectangle in XY plane
        Vec3::new(-1.0,  PHI,  0.0), // 0
        Vec3::new( 1.0,  PHI,  0.0), // 1
        Vec3::new(-1.0, -PHI,  0.0), // 2
        Vec3::new( 1.0, -PHI,  0.0), // 3
        // Rectangle in YZ plane
        Vec3::new( 0.0, -1.0,  PHI), // 4
        Vec3::new( 0.0,  1.0,  PHI), // 5
        Vec3::new( 0.0, -1.0, -PHI), // 6
        Vec3::new( 0.0,  1.0, -PHI), // 7
        // Rectangle in XZ plane
        Vec3::new( PHI,  0.0, -1.0), // 8
        Vec3::new( PHI,  0.0,  1.0), // 9
        Vec3::new(-PHI,  0.0, -1.0), // 10
        Vec3::new(-PHI,  0.0,  1.0), // 11
    ];

    // Normalize all to unit sphere
    raw.into_iter().map(|v| v.normalize()).collect()
}

/// Returns the 20 faces of a regular icosahedron
/// Winding is CCW when viewed from outside (outward facing normals)
pub fn icosahedron_faces() -> Vec<[u32; 3]> {
    vec![
        // 5 faces around top vertex (0)
        [0,  11,  5],
        [0,   5,  1],
        [0,   1,  7],
        [0,   7, 10],
        [0,  10, 11],
        // 5 adjacent faces (upper band)
        [11,  4,  5],
        [ 5,  9,  1],
        [ 1,  8,  7],
        [ 7,  6, 10],
        [10,  2, 11],
        // 5 adjacent faces (lower band)
        [ 4, 11,  2],
        [ 9,  5,  4],
        [ 8,  1,  9],
        [ 6,  7,  8],
        [ 2, 10,  6],
        // 5 faces around bottom vertex (3)
        [ 4,  2,  3],
        [ 9,  4,  3],
        [ 8,  9,  3],
        [ 6,  8,  3],
        [ 2,  6,  3],
    ]
}

/// Creates a Bevy Mesh for a regular icosahedron
///
/// Flat-shaded: each face has its own 3 vertices with per-face normals.
/// This makes edges crisp and visible instead of smooth-shaded.
#[allow(dead_code)]
pub fn create_icosahedron_mesh() -> Mesh {
    let verts = icosahedron_vertices();
    let faces = icosahedron_faces();

    // Build flat-shaded mesh: 3 vertices per face = 60 vertices total
    // Each face gets its own copy of its 3 vertices, all with the face normal
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(faces.len() * 3);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(faces.len() * 3);
    let mut indices: Vec<u32> = Vec::with_capacity(faces.len() * 3);

    for (face_idx, face) in faces.iter().enumerate() {
        let p0 = verts[face[0] as usize];
        let p1 = verts[face[1] as usize];
        let p2 = verts[face[2] as usize];

        // Compute face normal (outward, since vertices are on sphere)
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let normal = edge1.cross(edge2).normalize();

        let base = (face_idx * 3) as u32;

        // Vertex 0
        positions.push([p0.x, p0.y, p0.z]);
        normals.push([normal.x, normal.y, normal.z]);
        // Vertex 1
        positions.push([p1.x, p1.y, p1.z]);
        normals.push([normal.x, normal.y, normal.z]);
        // Vertex 2
        positions.push([p2.x, p2.y, p2.z]);
        normals.push([normal.x, normal.y, normal.z]);

        // CCW triangle: 0, 1, 2
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}