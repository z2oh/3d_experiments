use cgmath::prelude::*;

//use crate::simplex;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pos: [f32; 4],
    normal: [f32; 3],
    tc: [f32; 2],
}

pub const VERTEX_SIZE: usize = std::mem::size_of::<Vertex>();

use bytemuck::{Pod, Zeroable};

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

pub fn vertex(pos: [f32; 3], normal: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        pos: [pos[0], pos[1], pos[2], 1.0],
        normal,
        tc,
    }
}

struct MeshAccumulator {
    vertex_accum: Vec<Vertex>,
    index_accum: Vec<u32>,
    index_offset: u32,
}

#[allow(dead_code)]
impl MeshAccumulator {
    fn new() -> Self {
        Self {
            vertex_accum: Vec::new(),
            index_accum: Vec::new(),
            index_offset: 0,
        }
    }

    fn with_capacities(vertex_capacity: usize, index_capacity: usize) -> Self {
        Self {
            vertex_accum: Vec::with_capacity(vertex_capacity),
            index_accum: Vec::with_capacity(index_capacity),
            index_offset: 0,
        }
    }

    fn report(self) -> (Vec<Vertex>, Vec<u32>) {
        (self.vertex_accum, self.index_accum)
    }

    fn add_cuboid_quat(
        &mut self,
        center: cgmath::Point3<f32>,
        orientation: cgmath::Quaternion<f32>,
        width_2: f32,
        length_2: f32,
        height_2: f32,
    ) {
        use cgmath::prelude::*;
        use cgmath::{Rad, Vector3, Quaternion};
        let top_center = center + orientation.rotate_vector(Vector3::new(0., 0., height_2));
        let bottom_center = center - orientation.rotate_vector(Vector3::new(0., 0., height_2));
        let right_center = center + orientation.rotate_vector(Vector3::new(width_2, 0., 0.));
        let left_center = center - orientation.rotate_vector(Vector3::new(width_2, 0., 0.));
        let front_center = center + orientation.rotate_vector(Vector3::new(0., length_2, 0.));
        let back_center = center - orientation.rotate_vector(Vector3::new(0., length_2, 0.));

        let top = orientation;
        let front = orientation * Quaternion::from_angle_x(-Rad::turn_div_4());
        let back = orientation * Quaternion::from_angle_x(Rad::turn_div_4());
        let left = orientation * Quaternion::from_angle_y(-Rad::turn_div_4());
        let right = orientation * Quaternion::from_angle_y(Rad::turn_div_4());
        let bottom = orientation * Quaternion::from_angle_x(Rad::turn_div_2());

        self.add_quad_quat(top_center, top, width_2, length_2);
        self.add_quad_quat(bottom_center, bottom, width_2, length_2);

        self.add_quad_quat(left_center, left, height_2, length_2);
        self.add_quad_quat(right_center, right, height_2, length_2);
        self.add_quad_quat(front_center, front, width_2, height_2);
        self.add_quad_quat(back_center, back, width_2, height_2);
    }

    /// Create a quad mesh from a provided position vector and a quaternion describing the quad's
    /// orientation. A counter clockwise winding orientation is used.
    ///
    /// Up is assumed to be the z-axis.
    pub fn add_quad_quat(
        &mut self,
        center: cgmath::Point3<f32>,
        orientation: cgmath::Quaternion<f32>,
        width_2: f32,
        length_2: f32,
    ) {
        let i = self.index_offset;

        let pos = center.to_vec();
        let _1 = cgmath::Vector3::new(-width_2, -length_2, 0.0);
        let _2 = cgmath::Vector3::new(width_2, -length_2, 0.0);
        let _3 = cgmath::Vector3::new(width_2, length_2, 0.0);
        let _4 = cgmath::Vector3::new(-width_2, length_2, 0.0);

        let _1 = pos + orientation.rotate_vector(_1);
        let _2 = pos + orientation.rotate_vector(_2);
        let _3 = pos + orientation.rotate_vector(_3);
        let _4 = pos + orientation.rotate_vector(_4);

        let _1_n: [f32; 3] = orientation.rotate_vector(cgmath::Vector3::unit_z()).normalize().into();

        self.vertex_accum.extend(&[
            vertex([_1.x, _1.y, _1.z], _1_n, [0.0, 0.0]),
            vertex([_2.x, _2.y, _2.z], _1_n, [1.0, 0.0]),
            vertex([_3.x, _3.y, _3.z], _1_n, [1.0, 1.0]),
            vertex([_4.x, _4.y, _4.z], _1_n, [0.0, 1.0]),
        ]);
        self.index_accum.extend(&[0+i, 1+i, 2+i, 2+i, 3+i, 0+i]);
        self.index_offset += 4;
    }
}

pub fn create_vertices(prev_f: f32) -> (Vec<Vertex>, Vec<u32>) {
    // TODO: Simplex noise will make a return...
    //let prng_base = simplex::Simplex::with_seed(0);
    let mut mesh_accumulator = MeshAccumulator::new();

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
