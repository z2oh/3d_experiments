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
    cached_right: Option<cgmath::Vector3<f32>>,
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
            cached_right: None,
        }
    }

    // Requires a mutable reference since this function has automatic memoization. This might be okay.
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

    // Requires a mutable reference since this function has automatic memoization. This might be okay.
    fn right(&mut self) -> cgmath::Vector3<f32> {
        if let Some(cached_cross) = self.cached_right {
            cached_cross
        } else {
            let v = self.dir.cross(self.up);
            self.cached_right = Some(v);
            v
        }
    }

    /// Invalidate the cache. The next time we get the matrix, it will be recomputed.
    fn invalidate_cache(&mut self) {
        self.cached_matrix = None;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.invalidate_cache();
        self.aspect_ratio = aspect_ratio;
    }

    pub fn move_forward(&mut self, mag: f32) {
        self.invalidate_cache();
        self.eye += self.dir * mag;
    }

    pub fn move_backward(&mut self, mag: f32) {
        self.invalidate_cache();
        self.eye -= self.dir * mag;
    }

    pub fn move_right(&mut self, mag: f32) {
        self.invalidate_cache();
        let r = self.right();
        self.eye += r * mag;
    }

    pub fn move_left(&mut self, mag: f32) {
        self.invalidate_cache();
        let r = self.right();
        self.eye -= r * mag;
    }

}
