use winit::event::VirtualKeyCode;

use crate::render_context;

pub fn handle_key(render_context: &mut render_context::RenderContext, keycode: VirtualKeyCode) {
    match keycode {
        VirtualKeyCode::J => {
            render_context.set_amplitude(render_context.amplitude() * 1.1);
            render_context.set_mesh_dirty();
        },
        VirtualKeyCode::L => {
            render_context.set_frequency(render_context.frequency() * 1.1);
            render_context.set_mesh_dirty();
        },
        VirtualKeyCode::F | VirtualKeyCode::Up => {
            render_context.camera_mut().move_forward(1.0);
        },
        VirtualKeyCode::S | VirtualKeyCode::Down => {
            render_context.camera_mut().move_backward(1.0);
        },
        VirtualKeyCode::R | VirtualKeyCode::Left => {
            render_context.camera_mut().move_left(1.0);
        },
        VirtualKeyCode::T | VirtualKeyCode::Right => {
            render_context.camera_mut().move_right(1.0);
        },
        // Ignore other keys.
        _ => {},
    }
}
