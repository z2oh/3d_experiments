use winit::event::VirtualKeyCode;

use crate::render_context;

pub struct InputContext {
    x_sensitivity: f32,
    y_sensitivity: f32,
}

impl InputContext {
    pub fn new() -> Self {
        Self {
            x_sensitivity: 500.0,
            y_sensitivity: 500.0,
        }
    }

    pub fn handle_key(&self, render_context: &mut render_context::RenderContext, keycode: VirtualKeyCode) {
        match keycode {
            VirtualKeyCode::F | VirtualKeyCode::Up => {
                render_context.camera_mut().move_forward(0.5);
            },
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                render_context.camera_mut().move_backward(0.5);
            },
            VirtualKeyCode::R | VirtualKeyCode::Left => {
                render_context.camera_mut().move_left(0.5);
            },
            VirtualKeyCode::T | VirtualKeyCode::Right => {
                render_context.camera_mut().move_right(0.5);
            },
            VirtualKeyCode::W => {
                render_context.camera_mut().move_up(0.5);
            },
            VirtualKeyCode::P => {
                render_context.camera_mut().move_down(0.5);
            },
            // Ignore other keys.
            _ => {},
        }
    }

    pub fn handle_cursor_moved(
        &mut self,
        render_context: &mut render_context::RenderContext,
        (delta_x, delta_y): (f64, f64),
    ) {
        let delta = cgmath::Vector2::new(-delta_x as f32, -delta_y as f32);
        render_context.camera_mut().rotate_by_x_y(delta, self.x_sensitivity, self.y_sensitivity);
    }
}
