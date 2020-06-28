/// A messy mesh generation interface. Right now this is achieved through the `MeshAccumulator`
/// type, which can accumulate some primitives in a very naive way. However, this can be fixed
/// at any time with minimal external breakage, as the output data will always be a collection
/// of vertices and indices.
///
/// In fact, we probably want a `Mesh` type to further abstract this.

use cgmath::prelude::*;
use crate::utils::{IVertex, Vertex};
use cgmath::{Point3, Rad, Vector3, Quaternion};

use bitflags::bitflags;

bitflags! {
    pub struct CuboidFaces: u8 {
        const TOP    = 0b100000;
        const BOTTOM = 0b010000;
        const FRONT  = 0b001000;
        const BACK   = 0b000100;
        const LEFT   = 0b000010;
        const RIGHT  = 0b000001;
    }
}

// TODO: This will need to be generic over vertex type, index type, and index offset type
// eventually.
pub struct MeshAccumulator {
    vertex_accum: Vec<Vertex>,
    index_accum: Vec<u32>,
    index_offset: u32,
}

#[allow(dead_code)]
impl MeshAccumulator {
    /// Create a new `MeshAccumulator`.
    pub fn new() -> Self {
        Self {
            vertex_accum: Vec::new(),
            index_accum: Vec::new(),
            index_offset: 0,
        }
    }

    /// Create a new `MeshAccumulator` with the specified capacities for the vertex and index
    /// buffers. If a size is known externally, passing it here will prevent needless reallocation
    /// of the buffer as it grows.
    pub fn with_capacities(vertex_capacity: usize, index_capacity: usize) -> Self {
        Self {
            vertex_accum: Vec::with_capacity(vertex_capacity),
            index_accum: Vec::with_capacity(index_capacity),
            index_offset: 0,
        }
    }

    // TODO: Return new `Mesh` type.
    /// Consumes the `MeshAccumulator` and returns the vertex and index buffers.
    pub fn report(self) -> (Vec<Vertex>, Vec<u32>) {
        (self.vertex_accum, self.index_accum)
    }

    /// Add an isolated cuboid to the mesh. The parameter names are self-describing.
    ///
    /// Up is assumed to be the z-axis.
    pub fn add_cuboid_quat(
        &mut self,
        center: Point3<f32>,
        orientation: Quaternion<f32>,
        width_2: f32,
        length_2: f32,
        height_2: f32,
    ) {
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

    /// Add an isolated cuboid to the mesh. The parameter names are self-describing.
    ///
    /// Up is assumed to be the z-axis.
    pub fn add_cuboid_quat_faces(
        &mut self,
        center: Point3<f32>,
        orientation: Quaternion<f32>,
        faces: CuboidFaces,
        width_2: f32,
        length_2: f32,
        height_2: f32,
    ) {
        if faces.contains(CuboidFaces::TOP) {
            let top_center = center + orientation.rotate_vector(Vector3::new(0., 0., height_2));
            let top = orientation;
            self.add_quad_quat(top_center, top, width_2, length_2);
        }
        if faces.contains(CuboidFaces::BOTTOM) {
            let bottom_center = center - orientation.rotate_vector(Vector3::new(0., 0., height_2));
            let bottom = orientation * Quaternion::from_angle_x(Rad::turn_div_2());
            self.add_quad_quat(bottom_center, bottom, width_2, length_2);
        }
        if faces.contains(CuboidFaces::RIGHT) {
            let right_center = center + orientation.rotate_vector(Vector3::new(width_2, 0., 0.));
            let right = orientation * Quaternion::from_angle_y(Rad::turn_div_4());
            self.add_quad_quat(right_center, right, height_2, length_2);
        }
        if faces.contains(CuboidFaces::LEFT) {
            let left_center = center - orientation.rotate_vector(Vector3::new(width_2, 0., 0.));
            let left = orientation * Quaternion::from_angle_y(-Rad::turn_div_4());
            self.add_quad_quat(left_center, left, height_2, length_2);
        }
        if faces.contains(CuboidFaces::FRONT) {
            let front_center = center + orientation.rotate_vector(Vector3::new(0., length_2, 0.));
            let front = orientation * Quaternion::from_angle_x(-Rad::turn_div_4());
            self.add_quad_quat(front_center, front, width_2, height_2);
        }
        if faces.contains(CuboidFaces::BACK) {
            let back_center = center - orientation.rotate_vector(Vector3::new(0., length_2, 0.));
            let back = orientation * Quaternion::from_angle_x(Rad::turn_div_4());
            self.add_quad_quat(back_center, back, width_2, height_2);
        }
    }

    /// Add an isolated quad to the mesh. The parameter names are self-describing.
    ///
    /// Up is assumed to be the z-axis.
    pub fn add_quad_quat(
        &mut self,
        center: Point3<f32>,
        orientation: Quaternion<f32>,
        width_2: f32,
        length_2: f32,
    ) {
        let i = self.index_offset;

        let pos = center.to_vec();
        let _1 = Vector3::new(-width_2, -length_2, 0.0);
        let _2 = Vector3::new(width_2, -length_2, 0.0);
        let _3 = Vector3::new(width_2, length_2, 0.0);
        let _4 = Vector3::new(-width_2, length_2, 0.0);

        let _1 = pos + orientation.rotate_vector(_1);
        let _2 = pos + orientation.rotate_vector(_2);
        let _3 = pos + orientation.rotate_vector(_3);
        let _4 = pos + orientation.rotate_vector(_4);

        let _1_n: [f32; 3] = orientation.rotate_vector(Vector3::unit_z()).normalize().into();

        self.vertex_accum.extend(&[
            Vertex::new([_1.x, _1.y, _1.z], _1_n, [0.0, 0.0]),
            Vertex::new([_2.x, _2.y, _2.z], _1_n, [1.0, 0.0]),
            Vertex::new([_3.x, _3.y, _3.z], _1_n, [1.0, 1.0]),
            Vertex::new([_4.x, _4.y, _4.z], _1_n, [0.0, 1.0]),
        ]);
        self.index_accum.extend(&[0+i, 1+i, 2+i, 2+i, 3+i, 0+i]);
        self.index_offset += 4;
    }
}

pub struct IMeshAccumulator {
    vertex_accum: Vec<IVertex>,
    index_accum: Vec<u16>,
    index_offset: u16,
}

#[allow(dead_code)]
impl IMeshAccumulator {
    /// Create a new `IMeshAccumulator`.
    pub fn new() -> Self {
        Self {
            vertex_accum: Vec::new(),
            index_accum: Vec::new(),
            index_offset: 0,
        }
    }

    /// Create a new `IMeshAccumulator` with the specified capacities for the vertex and index
    /// buffers. If a size is known externally, passing it here will prevent needless reallocation
    /// of the buffer as it grows.
    pub fn with_capacities(vertex_capacity: usize, index_capacity: usize) -> Self {
        Self {
            vertex_accum: Vec::with_capacity(vertex_capacity),
            index_accum: Vec::with_capacity(index_capacity),
            index_offset: 0,
        }
    }

    // TODO: Return new `Mesh` type.
    /// Consumes the `MeshAccumulator` and returns the vertex and index buffers.
    pub fn report(self) -> (Vec<IVertex>, Vec<u16>) {
        (self.vertex_accum, self.index_accum)
    }

    /// Add an isolated cuboid to the mesh. The parameter names are self-describing.
    ///
    /// Up is assumed to be the z-axis.
    pub fn add_cuboid_faces(
        &mut self,
        pos: Point3<i32>,
        faces: CuboidFaces,
    ) {
        if faces.contains(CuboidFaces::TOP) {
            self.add_face(pos, CuboidFaces::TOP);
        }
        if faces.contains(CuboidFaces::BOTTOM) {
            self.add_face(pos, CuboidFaces::BOTTOM);
        }
        if faces.contains(CuboidFaces::FRONT) {
            self.add_face(pos, CuboidFaces::FRONT);
        }
        if faces.contains(CuboidFaces::BACK) {
            self.add_face(pos, CuboidFaces::BACK);
        }
        if faces.contains(CuboidFaces::LEFT) {
            self.add_face(pos, CuboidFaces::LEFT);
        }
        if faces.contains(CuboidFaces::RIGHT) {
            self.add_face(pos, CuboidFaces::RIGHT);
        }
    }

    /// Add an isolated quad to the mesh. The parameter names are self-describing.
    ///
    /// Up is assumed to be the z-axis.
    pub fn add_face(
        &mut self,
        pos: Point3<i32>,
        face: CuboidFaces,
    ) {
        let i = self.index_offset;

        let [v1, v2, v3, v4] = match face {
            CuboidFaces::TOP => [
                Vector3::new(pos.x+1, pos.y+1, pos.z+1),
                Vector3::new(pos.x  , pos.y+1, pos.z+1),
                Vector3::new(pos.x  , pos.y  , pos.z+1),
                Vector3::new(pos.x+1, pos.y  , pos.z+1),
            ],
            CuboidFaces::BOTTOM => [
                Vector3::new(pos.x+1, pos.y  , pos.z),
                Vector3::new(pos.x  , pos.y  , pos.z),
                Vector3::new(pos.x  , pos.y+1, pos.z),
                Vector3::new(pos.x+1, pos.y+1, pos.z),
            ],
            CuboidFaces::FRONT => [
                Vector3::new(pos.x+1, pos.y+1, pos.z),
                Vector3::new(pos.x  , pos.y+1, pos.z),
                Vector3::new(pos.x  , pos.y+1, pos.z+1),
                Vector3::new(pos.x+1, pos.y+1, pos.z+1),
            ],
            CuboidFaces::BACK => [
                Vector3::new(pos.x,   pos.y, pos.z),
                Vector3::new(pos.x+1, pos.y, pos.z),
                Vector3::new(pos.x+1, pos.y, pos.z+1),
                Vector3::new(pos.x,   pos.y, pos.z+1),
            ],
            CuboidFaces::LEFT => [
                Vector3::new(pos.x, pos.y+1, pos.z),
                Vector3::new(pos.x, pos.y,   pos.z),
                Vector3::new(pos.x, pos.y,   pos.z+1),
                Vector3::new(pos.x, pos.y+1, pos.z+1),
            ],
            CuboidFaces::RIGHT => [
                Vector3::new(pos.x+1, pos.y,   pos.z),
                Vector3::new(pos.x+1, pos.y+1, pos.z),
                Vector3::new(pos.x+1, pos.y+1, pos.z+1),
                Vector3::new(pos.x+1, pos.y,   pos.z+1),
            ],
            _ => panic!("bad faces"),
        };

        self.vertex_accum.extend(&[
            IVertex::new(v1, pos, [0.0, 0.0], face.bits()),
            IVertex::new(v2, pos, [1.0, 0.0], face.bits()),
            IVertex::new(v3, pos, [1.0, 1.0], face.bits()),
            IVertex::new(v4, pos, [0.0, 1.0], face.bits()),
        ]);
        self.index_accum.extend(&[0+i, 1+i, 2+i, 2+i, 3+i, 0+i]);
        self.index_offset += 4;
    }
}
