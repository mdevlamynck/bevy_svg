use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, Mesh},
        pipeline::PrimitiveTopology,
    },
};
use lyon_geom::{Point, Transform};
use lyon_tessellation::{
    self, FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor,
};

/// A vertex with all the necessary attributes to be inserted into a Bevy
/// [`Mesh`](bevy::render::mesh::Mesh).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Vertex {
    position: [f32; 3],
    color:    [f32; 4],
}

/// The index type of a Bevy [`Mesh`](bevy::render::mesh::Mesh).
pub(crate) type IndexType = u32;

/// Lyon's [`VertexBuffers`] generic data type defined for [`Vertex`].
pub(crate) type VertexBuffers = lyon_tessellation::VertexBuffers<Vertex, IndexType>;

/// Zero-sized type used to implement various vertex construction traits from
/// Lyon.
pub(crate) struct VertexConstructor {
    pub(crate) color: Color,
}

/// Enables the construction of a [`Vertex`] when using a `FillTessellator`.
impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y, 0.0],
            color:    [
                self.color.r(),
                self.color.g(),
                self.color.b(),
                self.color.a(),
            ],
        }
    }
}

/// Enables the construction of a [`Vertex`] when using a `StrokeTessellator`.
impl StrokeVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y, 0.0],
            color:    [
                self.color.r(),
                self.color.g(),
                self.color.b(),
                self.color.a(),
            ],
        }
    }
}

pub(crate) fn apply_transform(buffer: &mut VertexBuffers, transform: Transform<f32>) {
    for mut vertex in buffer.vertices.iter_mut() {
        let pos = transform.transform_point(Point::new(vertex.position[0], vertex.position[1]));

        vertex.position[0] = pos.x;
        vertex.position[1] = pos.y;
    }
}

pub(crate) fn merge_buffers(buffers: Vec<VertexBuffers>) -> VertexBuffers {
    let mut buffer = VertexBuffers::new();
    let mut offset = 0;

    for buf in buffers.iter() {
        buffer.vertices.extend(&buf.vertices);
        buffer
            .indices
            .extend(buf.indices.iter().map(|i| i + offset));

        offset += buf.vertices.len() as u32;
    }

    buffer
}

pub(crate) fn to_mesh(buffer: VertexBuffers) -> Mesh {
    let mut positions = Vec::with_capacity(buffer.vertices.len());
    let mut colors = Vec::with_capacity(buffer.vertices.len());

    buffer.vertices.iter().for_each(|v| {
        positions.push(v.position);
        colors.push(v.color);
    });

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(buffer.indices.clone())));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    mesh
}
