use cgmath::{Matrix4, Point3, Vector3};

use crate::simplex;
use crate::mesh::{CuboidFaces, IMeshAccumulator};
use crate::utils;

pub struct ChunkIndex {
    pub vertex_offset: usize,
    pub index_offset: usize,
    pub index_count: usize,
    pub transform_index: usize,
}

#[allow(dead_code)]
pub struct WorldGeometryManager {
    pub chunks: Vec<ChunkIndex>,

    /// This buffer holds the transforms for each of our chunks.
    pub transforms_buf: crate::managed_buffer::ManagedBuffer<utils::PaddedMatrix4, Vec<utils::PaddedMatrix4>>,

    /// Chunks are cubes of world geometry, and this value is the size of the cube.
    chunk_dim: usize,
    noise: simplex::Simplex,

    /// This buffer holds the full mesh for the world geometry.
    pub vertex_buf: crate::managed_buffer::ManagedBuffer<utils::IVertex, Vec<utils::IVertex>>,
    pub index_buf: crate::managed_buffer::ManagedBuffer<u16, Vec<u16>>,
}

impl WorldGeometryManager {
    // TODO: This shouldn't take a GpuContext. This needs another layer of abstraction around memory
    // management.
    pub fn new(gpu_context: &crate::gpu::GpuContext) -> Option<Self> {
        let chunk_dim = 16;
        let mut chunks = Vec::with_capacity(100);
        let mut chunk_transforms: Vec<utils::PaddedMatrix4> = Vec::with_capacity(100);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let noise = crate::simplex::Simplex::with_seed(0);

        for y in 0..30 {
            for x in 0..30 {
                // Generate our transform matrix for this chunk.
                let t = Matrix4::from_translation(Vector3::new((x * chunk_dim as i32) as f32, (y * chunk_dim as i32) as f32, 0.0));

                // Now generate the actual mesh for the chunk.
                let (vertices_n, indices_n) = generate_chunk_x_y(x, y, chunk_dim, &noise).report();
                let chunk_index = ChunkIndex {
                    vertex_offset: vertices.len(),
                    index_offset: indices.len(),
                    index_count: indices_n.len(),
                    transform_index: chunk_transforms.len(),
                };

                // And update our local accumulators.
                vertices.extend(vertices_n);
                indices.extend(indices_n);
                chunk_transforms.push(t.into());
                chunks.push(chunk_index);
            }
        }

        // Create the transforms buffer holding the transforms for each chunk.
        let transforms_buf = crate::managed_buffer::ManagedBuffer::new_uniform_buf_with_data(
            gpu_context,
            chunk_transforms,
        ).ok()?;

        // Now we create the vertex buffer and index buffer on the GPU.
        let vertex_buf = crate::managed_buffer::ManagedBuffer::new_vertex_buf_with_data(
            gpu_context,
            vertices,
        ).ok()?;
        let index_buf = crate::managed_buffer::ManagedBuffer::new_index_buf_with_data(
            gpu_context,
            indices,
        ).ok()?;

        // create the vertex and index buffers
        // create the chunk transforms buffer
        // create some initial chunks, centered around 0, 0
        Some(Self {
            chunks,
            transforms_buf,
            chunk_dim,
            noise,
            vertex_buf,
            index_buf,
        })
    }
}

/// Generate a chunk of world geometry given: coordinates, the chunk dimensions, and a simplex noise
/// instance.
pub fn generate_chunk_x_y(
    x_off: i32,
    y_off: i32,
    chunk_dim: usize,
    noise: &simplex::Simplex,
) -> IMeshAccumulator {
    let mut m = IMeshAccumulator::new();

    // Generate the height map for our current chunk of terrain.
    let mut height_map = Vec::with_capacity(chunk_dim * chunk_dim);
    for x_i in 0..chunk_dim {
        for y_i in 0..chunk_dim {
            let x = x_i as f32;// * 2.0;
            let y = y_i as f32;// * 2.0;
            let z1 = (noise.get2d(
                (x_off as f64 + (x / chunk_dim as f32) as f64) / 2.0,
                (y_off as f64 + (y / chunk_dim as f32) as f64) / 2.0,
            ) * 20.0 as f64) as f32;

            let mult = (noise.get2d(
                x_off as f64 + (x / chunk_dim as f32) as f64,
                y_off as f64 + (y / chunk_dim as f32) as f64,
            ) * 2.0 as f64) as f32;

            let extremes = (noise.get2d(
                (x_off as f64 + (x / chunk_dim as f32) as f64) / 10.0,
                (y_off as f64 + (y / chunk_dim as f32) as f64) / 10.0,
            ) * 10.0 as f64) as f32;

            height_map.push((z1 * mult * extremes).max(-1.0) as i32);
        }
    }

    // Generate a mesh from the heightmap.
    for x_i in 0..chunk_dim {
        for y_i in 0..chunk_dim {
            let z = height_map[x_i * chunk_dim + y_i];
            let mut faces = CuboidFaces::TOP;
            let mut z_max_delta: i32 = 1;
            if x_i == 0 || height_map[(x_i - 1) * chunk_dim + y_i] < z {
                faces |= CuboidFaces::LEFT;
                if x_i != 0 {
                    z_max_delta = z_max_delta.max(z - (height_map[(x_i - 1) * chunk_dim + y_i]));
                } else {
                    z_max_delta = z_max_delta.max(8);
                }
            }
            if x_i == (chunk_dim - 1) || height_map[(x_i + 1) * chunk_dim + y_i] < z {
                faces |= CuboidFaces::RIGHT;
                if x_i != (chunk_dim - 1) {
                    z_max_delta = z_max_delta.max(z - (height_map[(x_i + 1) * chunk_dim + y_i]));
                } else {
                    z_max_delta = z_max_delta.max(8);
                }
            }
            if y_i == 0 || height_map[x_i * chunk_dim + (y_i - 1)] < z {
                faces |= CuboidFaces::BACK;
                if y_i != 0 {
                    z_max_delta = z_max_delta.max(z - (height_map[x_i * chunk_dim + (y_i - 1)]));
                } else {
                    z_max_delta = z_max_delta.max(8);
                }
            }
            if y_i == (chunk_dim - 1) || height_map[x_i * chunk_dim + (y_i + 1)] < z {
                faces |= CuboidFaces::FRONT;
                if y_i != (chunk_dim - 1) {
                    z_max_delta = z_max_delta.max(z - (height_map[x_i * chunk_dim + (y_i + 1)]));
                } else {
                    z_max_delta = z_max_delta.max(8);
                }
            }
            for i in 0..z_max_delta {
                m.add_cuboid_faces(Point3::new(x_i as i32, y_i as i32, z - i), faces);
            }
        }
    }

    m
}
