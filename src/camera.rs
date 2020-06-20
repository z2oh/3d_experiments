use crate::utils;
use cgmath::prelude::*;

pub struct Camera {
    position: cgmath::Point3<f32>,
    view: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,

    /// Current pitch of the camera, in the range [-pi/2, pi/2].
    // TODO: This does not really belong here. It should be in some `MouseControllableCamera` type
    // which wraps the more general `Camera` type.
    pitch: cgmath::Rad<f32>,

    aspect_ratio: f32,
    fovy: f32,
    z_near: f32,
    z_far: f32,

    cached_matrix: Option<cgmath::Matrix4<f32>>,
    cached_right: Option<cgmath::Vector3<f32>>,
}

impl Camera {
    /// Initializes a new `Camera` struct. This function will normalize any input vectors.
    pub fn new(
        position: cgmath::Point3<f32>,
        view: cgmath::Vector3<f32>,
        up: cgmath::Vector3<f32>,
        aspect_ratio: f32,
        fovy: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        // Initialize our normalization invariants.
        let view = view.normalize();
        let up = up.normalize();

        // Calculate the current pitch of the camera, given our view vector. This is just the angle
        // between the view vector and the xy plane.
        let mut pitch = view.angle(view - (view.project_on(cgmath::Vector3::unit_z())));
        // If we were looking down, invert the calculated angle.
        if view.z < 0.0 {
            pitch = -pitch;
        }

        Self {
            position,
            view,
            up,

            pitch,

            aspect_ratio,
            fovy,
            z_near,
            z_far,

            cached_matrix: None,
            cached_right: None,
        }
    }

    // Requires a mutable reference since this function caches its results. This might be okay.
    pub fn matrix(&mut self) -> cgmath::Matrix4<f32> {
        if let Some(cached_matrix) = self.cached_matrix {
            cached_matrix
        } else {
            let m =
                utils::OPENGL_TO_WGPU_MATRIX *
                cgmath::perspective(cgmath::Deg(self.fovy), self.aspect_ratio, self.z_near, self.z_far) *
                cgmath::Matrix4::look_at_dir(self.position, self.view, self.up);

            self.cached_matrix = Some(m);
            m
        }
    }

    // Requires a mutable reference since this function caches its results. This might be okay.
    fn right(&mut self) -> cgmath::Vector3<f32> {
        if let Some(cached_cross) = self.cached_right {
            cached_cross
        } else {
            let v = (self.view).cross(self.up);
            self.cached_right = Some(v);
            v
        }
    }

    /// Invalidate the cache. The next time we get the matrix, it will be recomputed.
    fn invalidate_cache(&mut self) {
        self.cached_matrix = None;
        self.cached_right = None;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.invalidate_cache();

        self.aspect_ratio = aspect_ratio;
    }

    pub fn move_forward(&mut self, mag: f32) {
        self.invalidate_cache();

        self.position += self.view * mag;
    }

    pub fn move_backward(&mut self, mag: f32) {
        self.invalidate_cache();

        self.position -= self.view * mag;
    }

    pub fn move_right(&mut self, mag: f32) {
        self.invalidate_cache();

        let r = self.right();
        self.position += r * mag;
    }

    pub fn move_left(&mut self, mag: f32) {
        self.invalidate_cache();

        let r = self.right();
        self.position -= r * mag;
    }

    pub fn rotate_by_x_y(
        &mut self,
        delta: cgmath::Vector2<f32>,
        x_sensitivity: f32,
        y_sensitivity: f32,
    ) {
        self.invalidate_cache();

        // Left and right rotation, so rotate around the z-axis, *not* the up axis.
        let rot_axis_x = cgmath::Vector3::new(0.0, 0.0, 1.0);
        let x_theta = cgmath::Rad(delta.x / x_sensitivity);
        let x_quat = cgmath::Quaternion::from_axis_angle(rot_axis_x, x_theta);

        // Up and down rotation, so rotate around the cross product of up and the view direction.
        let rot_axis_y = self.view.cross(rot_axis_x).normalize();
        let y_theta = {
            let y_theta = cgmath::Rad(delta.y / y_sensitivity);
            // Normalize pitch to [-pi/2, pi/2].
            let new_pitch = self.pitch + y_theta;
            // If our new pitch would have exceeded the pitch range ([-pi/2, pi/2]), then we just
            // don't pitch the camera any more. This could cause some strange behavior for
            // pathological events (like if the mouse sensitivity is extremely high), but for now
            // this is good enough.
            if new_pitch > cgmath::Rad(std::f32::consts::PI * 0.5) {
                cgmath::Rad(0.0)
            } else if new_pitch < cgmath::Rad(-std::f32::consts::PI * 0.5) {
                cgmath::Rad(0.0)
            } else {
                self.pitch = new_pitch;
                y_theta
            }
        };
        let y_quat = cgmath::Quaternion::from_axis_angle(rot_axis_y, y_theta);

        self.view = y_quat.rotate_vector(self.view).normalize();
        self.view = x_quat.rotate_vector(self.view).normalize();
    }
}
