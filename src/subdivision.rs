//! Subdivision module — geodesic sphere tessellation
//!
//! Midpoint subdivision: each triangular face splits into 4 smaller triangles.
//! Vertices are projected back onto the unit sphere after each split.

use crate::icosahedron::{icosahedron_faces, icosahedron_vertices};
use bevy::prelude::*;

/// Subdivides a single triangle by splitting each edge at its midpoint,
/// projecting the midpoint to the sphere, then forming 4 sub-triangles.
///
/// Input vertices should be on the unit sphere.
/// Returns 4 triangles, each with 3 vertex indices into the new vertex list.
#[allow(dead_code)]
fn subdivide_triangle(
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    all_vertices: &mut Vec<Vec3>,
) -> [[u32; 3]; 4] {
    // Compute midpoints and project to sphere
    let m01 = ((v0 + v1) / 2.0).normalize();
    let m12 = ((v1 + v2) / 2.0).normalize();
    let m20 = ((v2 + v0) / 2.0).normalize();

    let idx0 = all_vertices.len() as u32;
    all_vertices.push(v0);
    let idx1 = all_vertices.len() as u32;
    all_vertices.push(v1);
    let idx2 = all_vertices.len() as u32;
    all_vertices.push(v2);
    let idx01 = all_vertices.len() as u32;
    all_vertices.push(m01);
    let idx12 = all_vertices.len() as u32;
    all_vertices.push(m12);
    let idx20 = all_vertices.len() as u32;
    all_vertices.push(m20);

    // 4 sub-triangles:
    //     v0
    //    /  \
    //  m01--m20
    //  / \  / \
    // v1--m12--v2
    [
        [idx0, idx01, idx20],  // top triangle
        [idx01, idx1, idx12], // left triangle
        [idx20, idx12, idx2],  // right triangle
        [idx01, idx12, idx20], // center triangle
    ]
}

/// Subdivides the entire icosahedron by 1 level
///
/// Returns a new vertex list and face list representing the subdivided mesh.
#[allow(dead_code)]
pub fn subdivide_icosahedron_once() -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let verts = icosahedron_vertices();
    let faces = icosahedron_faces();

    let mut new_vertices: Vec<Vec3> = Vec::with_capacity(faces.len() * 6);
    let mut new_faces: Vec<[u32; 3]> = Vec::with_capacity(faces.len() * 4);

    // Copy original vertices first
    for v in &verts {
        new_vertices.push(*v);
    }

    // Subdivide each face
    for face in &faces {
        let v0 = verts[face[0] as usize];
        let v1 = verts[face[1] as usize];
        let v2 = verts[face[2] as usize];

        let sub_tris = subdivide_triangle(v0, v1, v2, &mut new_vertices);
        for tri in &sub_tris {
            new_faces.push(*tri);
        }
    }

    (new_vertices, new_faces)
}

/// Subdivides the icosahedron by N levels
pub fn subdivide_icosahedron(levels: u32) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    if levels == 0 {
        return (icosahedron_vertices(), icosahedron_faces());
    }

    let (mut verts, mut faces) = subdivide_icosahedron_once();

    for _ in 1..levels {
        let (new_verts, new_faces) = subdivide_(&verts, &faces);
        verts = new_verts;
        faces = new_faces;
    }

    (verts, faces)
}

fn subdivide_(verts: &[Vec3], faces: &[[u32; 3]]) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let mut new_vertices: Vec<Vec3> = Vec::with_capacity(verts.len() + faces.len() * 3);
    let mut new_faces: Vec<[u32; 3]> = Vec::with_capacity(faces.len() * 4);

    // Copy original vertices
    for v in verts {
        new_vertices.push(*v);
    }

    for face in faces {
        let v0 = verts[face[0] as usize];
        let v1 = verts[face[1] as usize];
        let v2 = verts[face[2] as usize];

        let sub_tris = subdivide_triangle(v0, v1, v2, &mut new_vertices);
        for tri in &sub_tris {
            new_faces.push(*tri);
        }
    }

    (new_vertices, new_faces)
}

/// Creates a Bevy Mesh from subdivided icosahedron data with flat shading
pub fn create_subdivided_mesh(levels: u32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let (verts, faces) = subdivide_icosahedron(levels);

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(faces.len() * 3);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(faces.len() * 3);

    for face in &faces {
        let p0 = verts[face[0] as usize];
        let p1 = verts[face[1] as usize];
        let p2 = verts[face[2] as usize];

        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let normal = edge1.cross(edge2).normalize();

        positions.push([p0.x, p0.y, p0.z]);
        positions.push([p1.x, p1.y, p1.z]);
        positions.push([p2.x, p2.y, p2.z]);

        normals.push([normal.x, normal.y, normal.z]);
        normals.push([normal.x, normal.y, normal.z]);
        normals.push([normal.x, normal.y, normal.z]);
    }

    let indices: Vec<u32> = (0..faces.len() * 3).map(|i| i as u32).collect();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}
