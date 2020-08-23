use bevy::{
    prelude::*,
    render::{
        mesh::{VertexAttribute, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};

pub fn calculate_normals(vertices: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::default(); vertices.len()];
    let num_faces = indices.len() / 3;
    {
        for face in 0..num_faces {
            let i0 = face * 3;
            let i1 = i0 + 1;
            let i2 = i0 + 2;
            let a = vertices[indices[i0] as usize];
            let b = vertices[indices[i1] as usize];
            let c = vertices[indices[i2] as usize];
            let n = (b - a).cross(c - a);
            normals[indices[i0] as usize] += n;
            normals[indices[i1] as usize] += n;
            normals[indices[i2] as usize] += n;
        }
    }
    normals.into_iter().map(|n| n.normalize()).collect()
}

pub fn get_hex_vertices(center: Vec3, normal: Vec3, up: Vec3, size: f32) -> Vec<Vec3> {
    // `up` and `normal` should both be normalized and perpedicular (cross product should also be normalized)
    // get the a vec perpendicular to both up and normal
    let start = up.cross(normal);

    // rotate 60deg 6 times along the normal to correspond to each vertex of hex
    (0..6)
        .map(|rot| (rot as f32 * 60.0).to_radians())
        .map(|angle| Quat::from_axis_angle(normal, angle))
        .map(|quat| (quat.mul_vec3(start) * size) + center)
        .collect()
}

pub fn mesh_hex_outline(center: Vec3, normal: Vec3, up: Vec3, size: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);

    // compute vertices
    let vertices = get_hex_vertices(center, normal, up, size);
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Position".into(),
        values: VertexAttributeValues::Float3(
            vertices.iter().map(|v| [v.x(), v.y(), v.z()]).collect(),
        ),
    });

    // compute normals
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Normal".into(),
        values: VertexAttributeValues::Float3(
            vertices
                .iter()
                .map(|_| [normal.x(), normal.y(), normal.z()])
                .collect(),
        ),
    });

    // compute UVs
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Uv".into(),
        values: VertexAttributeValues::Float2(vertices.iter().map(|_| [0.0, 0.0]).collect()),
    });

    // indices
    mesh.indices = Some(vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 0]);
    mesh
}

pub fn mesh_hex_plane(center: Vec3, normal: Vec3, up: Vec3, size: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // compute vertices
    let vertices = get_hex_vertices(center, normal, up, size);
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Position".into(),
        values: VertexAttributeValues::Float3(
            vertices.iter().map(|v| [v.x(), v.y(), v.z()]).collect(),
        ),
    });

    // compute normals
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Normal".into(),
        values: VertexAttributeValues::Float3(
            vertices
                .iter()
                .map(|_| [normal.x(), normal.y(), normal.z()])
                .collect(),
        ),
    });

    // compute UVs
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Uv".into(),
        values: VertexAttributeValues::Float2(vertices.iter().map(|_| [0.0, 0.0]).collect()),
    });

    // indices
    mesh.indices = Some(vec![5, 0, 1, 2, 3, 4, 5, 1, 2, 2, 4, 5]);
    mesh
}

pub fn mesh_hex_voxel(top: Vec3, bottom: Vec3, normal: Vec3, up: Vec3, size: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // compute vertices
    let vertices: Vec<_> = get_hex_vertices(top, normal, up, size)
        .into_iter()
        .chain(get_hex_vertices(bottom, normal, up, size).into_iter())
        .collect();

    let triangles = vec![
        5, 0, 1, 2, 3, 4, 5, 1, 2, 2, 4, 5, 5, 11, 6, 0, 5, 6, 0, 6, 7, 1, 0, 7, 1, 7, 8, 2, 1, 8,
        2, 8, 9, 3, 2, 9, 3, 9, 10, 4, 3, 10, 4, 10, 11, 5, 4, 11,
    ];

    let normals = calculate_normals(&vertices, &triangles);

    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Position".into(),
        values: VertexAttributeValues::Float3(
            vertices.iter().map(|v| [v.x(), v.y(), v.z()]).collect(),
        ),
    });

    // indices
    mesh.indices = Some(triangles);

    // compute normals
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Normal".into(),
        values: VertexAttributeValues::Float3(
            normals.iter().map(|n| [n.x(), n.y(), n.z()]).collect(),
        ),
    });

    // compute UVs
    mesh.attributes.push(VertexAttribute {
        name: "Vertex_Uv".into(),
        values: VertexAttributeValues::Float2(vertices.iter().map(|_| [0.0, 0.0]).collect()),
    });

    mesh
}
