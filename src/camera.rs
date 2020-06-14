use crate::utils;

pub struct Camera {
    fovy: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,

    eye: cgmath::Point3<f32>,
    dir: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,

    cached_matrix: Option<cgmath::Matrix4<f32>>,
}

impl Camera {
    pub fn new(
        fovy: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
        eye: cgmath::Point3<f32>,
        dir: cgmath::Vector3<f32>
    ) -> Self {
        Self {
            fovy,
            aspect_ratio,
            z_near,
            z_far,
            eye,
            dir,
            up: cgmath::Vector3::unit_z(),
            cached_matrix: None,
        }
    }

    // Requires a mutable reference since this function has automatic memoization. This is probably okay.
    pub fn matrix(&mut self) -> cgmath::Matrix4<f32> {
        if let Some(cached_matrix) = self.cached_matrix {
            cached_matrix
        } else {
            let m = utils::OPENGL_TO_WGPU_MATRIX *
            cgmath::perspective(cgmath::Deg(self.fovy), self.aspect_ratio, self.z_near, self.z_far) *
            cgmath::Matrix4::look_at_dir(
                self.eye,
                self.dir,
                self.up,
            );

            self.cached_matrix = Some(m);
            m
        }
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        // Invalidate the cache. The next time we get the matrix, it will be recomputed.
        self.cached_matrix = None;
    }
}
