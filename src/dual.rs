//! Dual graph module — shows hexel boundary pattern on geodesic sphere
//!
//! Computes the dual of the subdivided triangular mesh.
//! The dual connects face centers, forming hex/pent cells (the "baseball seam" pattern).

use std::collections::{HashMap, HashSet};

/// Face center + adjacency list for dual graph construction
#[derive(Clone)]
struct FaceCenter {
    center: [f32; 3],
    neighbors: Vec<u32>, // indices into face list
}

/// Adjacent faces share an edge (2 vertices in common)
fn faces_are_adjacent(a: &[u32; 3], b: &[u32; 3]) -> bool {
    let mut shared = 0;
    for &av in a {
        for &bv in b {
            if av == bv {
                shared += 1;
            }
        }
    }
    shared == 2 // adjacent if they share exactly 2 vertices (an edge)
}

/// Builds the dual graph of a triangular mesh.
/// Returns (face_centers, adjacency_edges) where edges connect neighbor face centers.
pub fn build_dual_graph(
    vertices: &[Vec3],
    faces: &[[u32; 3]],
) -> (Vec<[f32; 3]>, Vec<([f32; 3], [f32; 3])>) {
    let n = faces.len();
    let mut centers: Vec<[f32; 3]> = Vec::with_capacity(n);
    let mut edges: Vec<([f32; 3], [f32; 3])> = Vec::new();

    // Compute face centers
    for face in faces {
        let p0 = vertices[face[0] as usize];
        let p1 = vertices[face[1] as usize];
        let p2 = vertices[face[2] as usize];
        let center = [(p0.x + p1.x + p2.x) / 3.0,
                      (p0.y + p1.y + p2.y) / 3.0,
                      (p0.z + p1.z + p2.z) / 3.0];
        centers.push(center);
    }

    // Find adjacent face pairs and connect centers
    let mut seen: HashSet<u64> = HashSet::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if faces_are_adjacent(&faces[i], &faces[j]) {
                let key = ((i as u64) << 32) | (j as u64);
                if !seen.contains(&key) {
                    seen.insert(key);
                    edges.push((centers[i], centers[j]));
                }
            }
        }
    }

    (centers, edges)
}

/// Face center index + which original face it came from
#[derive(Clone)]
pub struct DualFace {
    pub center: Vec3,
    pub vertex_indices: [u32; 3],
    pub adjacent_centers: Vec<Vec3>,
}

/// Full dual graph data for a subdivided sphere
pub struct DualGraph {
    pub faces: Vec<DualFace>,
    pub edges: Vec<([f32; 3], [f32; 3])>,
}

impl DualGraph {
    /// Build dual graph from subdivided icosahedron data
    pub fn from_subdivided(vertices: &[Vec3], faces: &[[u32; 3]]) -> Self {
        let n = faces.len();
        let mut dual_faces: Vec<DualFace> = Vec::with_capacity(n);
        let mut centers: Vec<Vec3> = Vec::with_capacity(n);

        // Compute centers
        for face in faces {
            let p0 = vertices[face[0] as usize];
            let p1 = vertices[face[1] as usize];
            let p2 = vertices[face[2] as usize];
            centers.push((p0 + p1 + p2) / 3.0);
        }

        // Build adjacency and dual faces
        for i in 0..n {
            let mut adjacent_centers = Vec::new();
            for j in 0..n {
                if i != j && faces_are_adjacent(&faces[i], &faces[j]) {
                    adjacent_centers.push(centers[j]);
                }
            }
            dual_faces.push(DualFace {
                center: centers[i],
                vertex_indices: faces[i],
                adjacent_centers,
            });
        }

        // Build edge list
        let mut edge_set: HashSet<u64> = HashSet::new();
        let mut edges: Vec<([f32; 3], [f32; 3])> = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                if faces_are_adjacent(&faces[i], &faces[j]) {
                    let key = ((i as u64) << 32) | (j as u64);
                    if !edge_set.contains(&key) {
                        edge_set.insert(key);
                        edges.push((centers[i].to_array(), centers[j].to_array()));
                    }
                }
            }
        }

        DualGraph { faces: dual_faces, edges }
    }
}
