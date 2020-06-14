use winit::event::VirtualKeyCode;

use crate::render_context;

pub fn handle_key(render_context: &mut render_context::RenderContext, keycode: VirtualKeyCode) {
    match keycode {
        VirtualKeyCode::J => {
            render_context.set_amplitude(render_context.amplitude() * 1.1);
            render_context.set_dirty();
        },
        VirtualKeyCode::L => {
            render_context.set_frequency(render_context.frequency() * 1.1);
            render_context.set_dirty();
        },
        // Ignore other keys.
        _ => {},
    }
}
