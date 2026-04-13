//! Hexel module — geodesic hexagonal sphere (Unity-style dual graph)
//!
//! Strategy: The subdivided icosahedron is a triangle mesh.
//! Each vertex of that mesh is surrounded by N triangles (5 at icosahedron vertices = pentagons,
//! 6 elsewhere = hexagons). We build a flat polygon by connecting the CENTERS of all
//! surrounding faces (lerped from vertex toward face center), then project to sphere.
//! This gives perfect tiling with no gaps — shared face centers ensure edges match exactly.
//!
//! Reference: Unity Hexasphere Tile.cs

use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// ICOSAHEDRON + SUBDIVISION
// ─────────────────────────────────────────────────────────────────────────────

/// Returns 12 vertices of a unit icosahedron.
fn ico_verts() -> [Vec3; 12] {
    let phi = (1.0 + (5.0f32).sqrt()) / 2.0;
    [
        Vec3::new(-1.0,  phi,  0.0).normalize(),
        Vec3::new( 1.0,  phi,  0.0).normalize(),
        Vec3::new(-1.0, -phi,  0.0).normalize(),
        Vec3::new( 1.0, -phi,  0.0).normalize(),
        Vec3::new( 0.0, -1.0,  phi).normalize(),
        Vec3::new( 0.0,  1.0,  phi).normalize(),
        Vec3::new( 0.0, -1.0, -phi).normalize(),
        Vec3::new( 0.0,  1.0, -phi).normalize(),
        Vec3::new( phi,  0.0, -1.0).normalize(),
        Vec3::new( phi,  0.0,  1.0).normalize(),
        Vec3::new(-phi,  0.0, -1.0).normalize(),
        Vec3::new(-phi,  0.0,  1.0).normalize(),
    ]
}

/// Returns 20 icosahedron faces (CCW viewed from outside).
fn ico_faces() -> [[usize; 3]; 20] {
    [
        [0, 11,  5], [0,  5,  1], [0,  1,  7], [0,  7, 10], [0, 10, 11],
        [11, 4,  5], [5,  9,  1], [1,  8,  7], [7,  6, 10], [10, 2, 11],
        [4, 11,  2], [9,  5,  4], [8,  1,  9], [6,  7,  8], [2, 10,  6],
        [4,  2,  3], [9,  4,  3], [8,  9,  3], [6,  8,  3], [2,  6,  3],
    ]
}

/// Subdivides one triangle into 4 at midpoints (n=1 subdivision).
fn subdivide_tri(v0: Vec3, v1: Vec3, v2: Vec3, out: &mut Vec<Vec3>, faces: &mut Vec<[u32; 3]>) {
    let m01 = ((v0 + v1) / 2.0).normalize();
    let m12 = ((v1 + v2) / 2.0).normalize();
    let m20 = ((v2 + v0) / 2.0).normalize();

    let base = out.len() as u32;
    out.push(v0);
    out.push(v1);
    out.push(v2);
    out.push(m01);
    out.push(m12);
    out.push(m20);

    // 4 sub-triangles, CCW
    faces.push([base,     base + 3, base + 5]);
    faces.push([base + 3, base + 1, base + 4]);
    faces.push([base + 5, base + 4, base + 2]);
    faces.push([base + 3, base + 4, base + 5]);
}

/// Subdivides icosahedron by n levels.
pub fn subdivide(n: u32) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let raw_verts = ico_verts();
    let raw_faces = ico_faces();

    let mut verts: Vec<Vec3> = raw_verts.iter().copied().collect();
    let mut faces: Vec<[u32; 3]> = Vec::new();

    if n == 0 {
        faces = raw_faces.map(|f| [f[0] as u32, f[1] as u32, f[2] as u32]).to_vec();
        return (verts, faces);
    }

    // Level 1
    for face in &raw_faces {
        subdivide_tri(raw_verts[face[0]], raw_verts[face[1]], raw_verts[face[2]], &mut verts, &mut faces);
    }

    // Further levels
    for _ in 2..=n {
        let mut new_faces: Vec<[u32; 3]> = Vec::with_capacity(faces.len() * 4);
        for face in &faces {
            subdivide_tri(verts[face[0] as usize], verts[face[1] as usize], verts[face[2] as usize], &mut verts, &mut new_faces);
        }
        faces = new_faces;
    }

    (verts, faces)
}

// ─────────────────────────────────────────────────────────────────────────────
// DUAL GRAPH — vertex → [face centers]
// ─────────────────────────────────────────────────────────────────────────────

/// For each vertex: list of face centroids (on sphere) that share it, sorted CCW.
fn build_vertex_rings(verts: &[Vec3], faces: &[[u32; 3]]) -> Vec<Vec<Vec3>> {
    // Map vertex index → list of (face_centroid, angle)
    let mut ring_map: HashMap<u32, Vec<(Vec3, f32)>> = HashMap::new();

    for face in faces {
        let c = ((verts[face[0] as usize] + verts[face[1] as usize] + verts[face[2] as usize]) / 3.0).normalize();
        for &vi in face {
            let normal = verts[vi as usize].normalize();
            let up = if normal.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
            let t = up.cross(normal).normalize();
            let b = normal.cross(t).normalize();
            let angle = (c - normal).dot(b).atan2((c - normal).dot(t));
            ring_map.entry(vi).or_default().push((c, angle));
        }
    }

    // Sort each ring by angle
    let mut rings: Vec<Vec<Vec3>> = Vec::new();
    for vi in 0..verts.len() {
        let mut ring = ring_map.remove(&((vi as u32))).unwrap_or_default();
        ring.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        rings.push(ring.into_iter().map(|(c, _)| c).collect());
    }
    rings
}

// ─────────────────────────────────────────────────────────────────────────────
// PRISM BUILDER
// ─────────────────────────────────────────────────────────────────────────────

fn build_prism(
    center: Vec3,
    boundary: &[Vec3],
    height: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
) {
    let n = boundary.len();
    let normal = center.normalize();
    let outer_center = center + normal * height;
    let outer: Vec<Vec3> = boundary.iter().map(|&p| p + normal * height).collect();

    // Top face (outward)
    let top_c = positions.len() as u32;
    positions.push(outer_center.to_array());
    normals.push(normal.to_array());
    let top_ring = positions.len() as u32;
    for &v in &outer {
        positions.push(v.to_array());
        normals.push(normal.to_array());
    }
    for i in 0..n {
        indices.push(top_c);
        indices.push(top_ring + i as u32);
        indices.push(top_ring + ((i + 1) % n) as u32);
    }

    // Bottom face (inward)
    let bot_c = positions.len() as u32;
    positions.push(center.to_array());
    normals.push((-normal).to_array());
    let bot_ring = positions.len() as u32;
    for &v in boundary {
        positions.push(v.to_array());
        normals.push((-normal).to_array());
    }
    for i in 0..n {
        indices.push(bot_c);
        indices.push(bot_ring + ((i + 1) % n) as u32);
        indices.push(bot_ring + i as u32);
    }

    // Side faces
    for i in 0..n {
        let next = (i + 1) % n;
        let edge_mid = (boundary[i] + boundary[next]) * 0.5;
        let side_n = (edge_mid - center).normalize();
        let base = positions.len() as u32;
        positions.push(boundary[i].to_array()); normals.push(side_n.to_array());
        positions.push(boundary[next].to_array()); normals.push(side_n.to_array());
        positions.push(outer[next].to_array()); normals.push(side_n.to_array());
        positions.push(outer[i].to_array()); normals.push(side_n.to_array());
        indices.push(base);     indices.push(base + 1); indices.push(base + 2);
        indices.push(base);     indices.push(base + 2); indices.push(base + 3);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PUBLIC API
// ─────────────────────────────────────────────────────────────────────────────

pub fn create_hexel_sphere_mesh(
    _verts: &[Vec3],
    _faces: &[[u32; 3]],
    hexel_height: f32,
    subdivision_level: u32,
) -> Mesh {
    // Re-subdivide from scratch at the given level
    let (verts, faces) = subdivide(subdivision_level);
    let rings = build_vertex_rings(&verts, &faces);

    let mut all_pos: Vec<[f32; 3]> = Vec::new();
    let mut all_nrm: Vec<[f32; 3]> = Vec::new();
    let mut all_idx: Vec<u32> = Vec::new();

    for (vi, ring) in rings.iter().enumerate() {
        if ring.len() < 3 {
            continue;
        }
        let center = verts[vi].normalize();

        // ── Build flat polygon from ring ────────────────────────────────────
        // Lerp from vertex center toward each face centroid.
        // `size` controls how big the hexel is (0.5 = golfball dimples).
        let size = 0.5;
        let polygon: Vec<Vec3> = ring.iter().map(|&fc| {
            let lerped = center.lerp(fc, size);
            lerped.normalize()
        }).collect();

        // ── Project polygon vertices slightly outward for the prism base ─────────
        // The boundary vertices of the prism are at the same radius as center
        let offset = all_pos.len() as u32;
        let mut pos = Vec::new();
        let mut nrm = Vec::new();
        let mut idx = Vec::new();
        build_prism(center, &polygon, hexel_height, &mut pos, &mut nrm, &mut idx);
        all_pos.extend_from_slice(&pos);
        all_nrm.extend_from_slice(&nrm);
        all_idx.extend(idx.into_iter().map(|i| i + offset));
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, all_pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, all_nrm)
        .with_inserted_indices(Indices::U32(all_idx))
}
