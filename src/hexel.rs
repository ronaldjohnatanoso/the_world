use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────
// HASH (fast deduplication)
// ─────────────────────────────────────────────────────────────

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct Key(i32, i32, i32);

fn quantize(v: Vec3) -> Key {
    let scale = 10000.0;
    Key(
        (v.x * scale).round() as i32,
        (v.y * scale).round() as i32,
        (v.z * scale).round() as i32,
    )
}

// ─────────────────────────────────────────────────────────────
// MESH BUILDER
// ─────────────────────────────────────────────────────────────

struct MeshBuilder {
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    map: HashMap<Key, u32>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            positions: vec![],
            indices: vec![],
            map: HashMap::new(),
        }
    }

    fn add_vertex(&mut self, v: Vec3) -> u32 {
        let k = quantize(v);
        if let Some(&i) = self.map.get(&k) {
            return i;
        }
        let i = self.positions.len() as u32;
        self.positions.push(v);
        self.map.insert(k, i);
        i
    }

    fn add_triangle(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        let i0 = self.add_vertex(a);
        let i1 = self.add_vertex(b);
        let i2 = self.add_vertex(c);
        self.indices.extend([i0, i1, i2]);
    }
}

// ─────────────────────────────────────────────────────────────
// ICOSAHEDRON
// ─────────────────────────────────────────────────────────────

fn icosahedron() -> (Vec<Vec3>, Vec<[usize; 3]>) {
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;

    let mut v = vec![
        Vec3::new(-1.,  t,  0.),
        Vec3::new( 1.,  t,  0.),
        Vec3::new(-1., -t,  0.),
        Vec3::new( 1., -t,  0.),
        Vec3::new( 0., -1.,  t),
        Vec3::new( 0.,  1.,  t),
        Vec3::new( 0., -1., -t),
        Vec3::new( 0.,  1., -t),
        Vec3::new( t,  0., -1.),
        Vec3::new( t,  0.,  1.),
        Vec3::new(-t,  0., -1.),
        Vec3::new(-t,  0.,  1.),
    ];

    for p in &mut v {
        *p = p.normalize();
    }

    let f = vec![
        [0,11,5],[0,5,1],[0,1,7],[0,7,10],[0,10,11],
        [1,5,9],[5,11,4],[11,10,2],[10,7,6],[7,1,8],
        [3,9,4],[3,4,2],[3,2,6],[3,6,8],[3,8,9],
        [4,9,5],[2,4,11],[6,2,10],[8,6,7],[9,8,1],
    ];

    (v, f)
}

// ─────────────────────────────────────────────────────────────
// SUBDIVISION (correct + cached)
// ─────────────────────────────────────────────────────────────

fn midpoint(
    cache: &mut HashMap<(usize, usize), usize>,
    verts: &mut Vec<Vec3>,
    a: usize,
    b: usize,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };

    if let Some(&i) = cache.get(&key) {
        return i;
    }

    let m = (verts[a] + verts[b]).normalize();
    let i = verts.len();
    verts.push(m);
    cache.insert(key, i);
    i
}

fn subdivide(
    mut verts: Vec<Vec3>,
    mut faces: Vec<[usize; 3]>,
    level: usize,
) -> (Vec<Vec3>, Vec<[usize; 3]>) {
    for _ in 0..level {
        let mut new_faces = vec![];
        let mut cache = HashMap::new();

        for [a, b, c] in faces {
            let ab = midpoint(&mut cache, &mut verts, a, b);
            let bc = midpoint(&mut cache, &mut verts, b, c);
            let ca = midpoint(&mut cache, &mut verts, c, a);

            new_faces.extend([
                [a, ab, ca],
                [b, bc, ab],
                [c, ca, bc],
                [ab, bc, ca],
            ]);
        }

        faces = new_faces;
    }

    (verts, faces)
}

// ─────────────────────────────────────────────────────────────
// HEXASPHERE
// ─────────────────────────────────────────────────────────────

fn build_hexasphere(radius: f32, divisions: usize, hex_size: f32) -> Mesh {
    let (base_verts, base_faces) = icosahedron();
    let (verts, faces) = subdivide(base_verts, base_faces, divisions);

    // vertex → faces
    let mut vertex_faces: Vec<Vec<usize>> = vec![vec![]; verts.len()];
    for (i, f) in faces.iter().enumerate() {
        for &vi in f {
            vertex_faces[vi].push(i);
        }
    }

    let mut builder = MeshBuilder::new();

    for (vi, pos) in verts.iter().enumerate() {
        let center = pos.normalize();

        // collect centroids
        let mut points = vec![];
        for &fi in &vertex_faces[vi] {
            let [a, b, c] = faces[fi];
            let centroid = (verts[a] + verts[b] + verts[c]) / 3.0;
            points.push(centroid.normalize());
        }

        // sort around normal — project to tangent plane, compute angle from center
        let up = if center.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let tangent = up.cross(center).normalize();
        let bitangent = center.cross(tangent).normalize();

        points.sort_by(|a, b| {
            let da = *a - center;
            let db = *b - center;
            let angle_a = da.dot(tangent).atan2(da.dot(bitangent));
            let angle_b = db.dot(tangent).atan2(db.dot(bitangent));
            angle_a.partial_cmp(&angle_b).unwrap()
        });

        // build polygon
        let poly: Vec<Vec3> = points
            .iter()
            .map(|p| center.lerp(*p, hex_size).normalize() * radius)
            .collect();

        // triangulate (fan)
        for i in 1..poly.len() - 1 {
            builder.add_triangle(poly[0], poly[i], poly[i + 1]);
        }
    }

    let positions: Vec<[f32; 3]> =
        builder.positions.iter().map(|v| v.to_array()).collect();

    let normals: Vec<[f32; 3]> =
        builder.positions.iter().map(|v| v.normalize().to_array()).collect();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(builder.indices))
}

// ─────────────────────────────────────────────────────────────
// PUBLIC API
// ─────────────────────────────────────────────────────────────

pub fn create_hexel_sphere_mesh(subdivision_level: u32) -> Mesh {
    build_hexasphere(1.0, subdivision_level as usize, 0.95)
}