use cgmath::prelude::*;

//use crate::simplex;

/// A bit of a hacky type to allow a Matrix4 to be treated as an owned collection of f32s by the
/// `ManagedBuffer` type. This is the common `newtype` pattern.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Matrix4(cgmath::Matrix4<f32>);

unsafe impl Pod for Matrix4 {}
unsafe impl Zeroable for Matrix4 {}

impl From<cgmath::Matrix4<f32>> for Matrix4 {
    fn from(matrix: cgmath::Matrix4<f32>) -> Self {
        Matrix4(matrix)
    }
}

impl AsRef<[f32]> for Matrix4 {
    fn as_ref(&self) -> &[f32] {
        let array_ref: &[f32; 16] = self.0.as_ref();
        array_ref.as_ref()
    }
}

/// A bit of a hacky type to allow a Matrix4 to be treated as an owned collection of f32s by the
/// `ManagedBuffer` type. This is the common `newtype` pattern, with some additional padding since
/// dynamic offsets must be 256-byte aligned. This is temporary.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PaddedMatrix4(cgmath::Matrix4<f32>, [u8; 192]);

unsafe impl Pod for PaddedMatrix4 {}
unsafe impl Zeroable for PaddedMatrix4 {}

impl From<cgmath::Matrix4<f32>> for PaddedMatrix4 {
    fn from(matrix: cgmath::Matrix4<f32>) -> Self {
        PaddedMatrix4(matrix, [0; 192])
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pos: [f32; 4],
    normal: [f32; 3],
    tc: [f32; 2],
}

impl Vertex {
    pub fn new(pos: [f32; 3], normal: [f32; 3], tc: [f32; 2]) -> Vertex {
        Vertex {
            pos: [pos[0], pos[1], pos[2], 1.0],
            normal,
            tc,
        }
    }
}

pub const VERTEX_SIZE: usize = std::mem::size_of::<Vertex>();

use bytemuck::{Pod, Zeroable};

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}


pub fn create_vertices(prev_f: f32) -> (Vec<Vertex>, Vec<u32>) {
    // TODO: Simplex noise will make a return...
    //let prng_base = simplex::Simplex::with_seed(0);
    let mut mesh_accumulator = crate::mesh::MeshAccumulator::new();

    let base = cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Rad(0.0)).normalize();

    let origin = cgmath::Point3::origin();
    for y in 0..64 {
        for x in 0..64 {
            let y = y - 32;
            let x = x - 32;
            // Calculate the polar coordinates.
            let _theta = f32::atan2(y as f32, x as f32);
            let r = ((x*x + y*y) as f32).sqrt();

            // If we are too far away, throw away the cuboid.
            if r > 32.0 {
                continue;
            }

            let z = (r + prev_f).sin();

            let pos = cgmath::Vector3::new(x as f32, y as f32, z);
            mesh_accumulator.add_cuboid_quat(origin + (pos), base, 0.5, 0.5, 0.5);
        }
    }
    mesh_accumulator.report()
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

#[macro_export]
macro_rules! benchmark {
    ($label:expr, $body:expr) => {{
        let before = ::std::time::Instant::now();
        let res = $body;
        let after = ::std::time::Instant::now();
        log::info!("[TIME] {:?} took {:?}", $label, after - before);
        res
    }}
}
