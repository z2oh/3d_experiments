use crate::simplex;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

pub const VERTEX_SIZE: usize = std::mem::size_of::<Vertex>();

use bytemuck::{Pod, Zeroable};

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

pub fn vertex(pos: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1.0],
        _tex_coord: [tc[0], tc[1]],
    }
}

pub fn create_cuboid(offset_x: f32, offset_y: f32, offset_z: f32, width: f32, length: f32, height: f32, index_offset: u32) -> ([Vertex; 24], [u32; 36]) {
    ([
        // top (0, 0,
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [0.0, 0.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [1.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [1.0, 1.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [0.0, 1.0]),
        // bottom (0.0, 0.0, -1)
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [1.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [0.0, 0.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [0.0, 1.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [1.0, 1.0]),
        // right (1.0, 0.0, 0)
        vertex([1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [0.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [1.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [1.0, 1.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [0.0, 1.0]),
        // left (-1.0, 0.0, 0)
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [1.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [0.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [0.0, 1.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [1.0, 1.0]),
        // front (0.0, 1.0, 0)
        vertex([1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [1.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [0.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [0.0, 1.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [1.0, 1.0]),
        // back (0.0, -1.0, 0)
        vertex([1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [0.0, 0.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [1.0, 0.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [1.0, 1.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [0.0, 1.0]),
    ],
    [
        0+index_offset, 1+index_offset, 2+index_offset, 2+index_offset, 3+index_offset, 0+index_offset, // top
        4+index_offset, 5+index_offset, 6+index_offset, 6+index_offset, 7+index_offset, 4+index_offset, // bottom
        8+index_offset, 9+index_offset, 10+index_offset, 10+index_offset, 11+index_offset, 8+index_offset, // right
        12+index_offset, 13+index_offset, 14+index_offset, 14+index_offset, 15+index_offset, 12+index_offset, // left
        16+index_offset, 17+index_offset, 18+index_offset, 18+index_offset, 19+index_offset, 16+index_offset, // front
        20+index_offset, 21+index_offset, 22+index_offset, 22+index_offset, 23+index_offset, 20+index_offset, // back
    ])
}

pub fn create_vertices(amplitude: f64, frequency: f32) -> (Vec<Vertex>, Vec<u32>) {
    let prng_base = simplex::Simplex::with_seed(0);
    let mut index_offset = 0;
    let mut vertex_accum: Vec<Vertex> = Vec::with_capacity(100 * 100 * 24);
    let mut index_accum: Vec<u32> = Vec::with_capacity(100 * 100 * 36);
    for y in 0..100 {
        for x in 0..100 {
            let x = (x as f32) * 2.0;
            let y = (y as f32) * 2.0;
            let z = (prng_base.get2d((x / frequency) as f64, (y / frequency) as f64) * amplitude as f64) as f32;
            let (vertices_next, indices_next) = create_cuboid(x, y, z, 1.0, 1.0, 8.0, index_offset);
            index_offset += 24;
            vertex_accum.extend(vertices_next.iter());
            index_accum.extend(indices_next.iter());
        }
    }

    (vertex_accum, index_accum)
}

pub fn create_texels(_size: u32) -> Vec<u8> {
    // This only works because "white.png" happens to be exactly the right size, 256x256 in this case.
    let image = image::open("white.png").unwrap();
    image.to_rgba().into_raw()
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
