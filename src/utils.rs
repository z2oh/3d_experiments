use bytemuck::{Pod, Zeroable};

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

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IVertex {
    v_pos: [i32; 3],
    b_pos: [i32; 3],
    tc: [f32; 2],
    data: [u8; 4],
}

impl IVertex {
    pub fn new(v_pos: cgmath::Vector3<i32>, b_pos: cgmath::Point3<i32>, tc: [f32; 2], face: u8) -> IVertex {
        IVertex {
            v_pos: v_pos.into(),
            b_pos: b_pos.into(),
            tc,
            data: [face, 0, 0, 0],
        }
    }
}

pub const IVERTEX_SIZE: usize = std::mem::size_of::<IVertex>();

unsafe impl Pod for IVertex {}
unsafe impl Zeroable for IVertex {}

pub fn load_image_bytes(path: &str) -> Vec<u8> {
    let image = image::open(path).unwrap();
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
