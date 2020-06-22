use winit::{
    event,
    event::{Event, DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

mod camera;
mod input;
mod render_context;
mod simplex;
mod utils;

use render_context::RenderContext;

async fn run(event_loop: EventLoop<()>, window: Window) {
    env_logger::init();
    // Initialize the render context.
    let mut render_context = RenderContext::create(&window).await.unwrap();
    let mut input_context = input::InputContext::new();

    // Start focused by default, assuming the application was executed with the intention of using it straight away.
    let mut window_focused: bool = true;
    let six_ms = std::time::Duration::from_millis(6);
    // Move our starting time back by the time between frame requests so that we request the first frame right away.
    let mut prev_frame = std::time::Instant::now() - six_ms;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                if now - prev_frame > six_ms {
                    render_context.set_mesh_dirty();
                    prev_frame = now;
                }
                render_context.render();
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => render_context.resize(size),
            // Handle requests to close the window...
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } |
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: event::KeyboardInput {
                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                state: event::ElementState::Pressed, ..
            }, .. }, .. } => {
                *control_flow = ControlFlow::Exit;
                window.set_cursor_grab(false).unwrap();
                window.set_cursor_visible(true);
            },

            // Other keypresses go to the input handler.
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: event::KeyboardInput {
                virtual_keycode: Some(keycode),
                state: event::ElementState::Pressed, ..
            }, .. }, .. } => input_context.handle_key(&mut render_context, keycode),

            // We track if the window has focus so that we can ignore device events when focus is lost.
            Event::WindowEvent { event: WindowEvent::Focused(b), .. } => window_focused = b,

            Event::WindowEvent { event: WindowEvent::CursorEntered { .. }, .. } => {
                window.set_cursor_grab(true).unwrap();
                window.set_cursor_visible(false);
            },
            Event::WindowEvent { event: WindowEvent::CursorLeft { .. }, .. } => {
                window.set_cursor_grab(false).unwrap();
                window.set_cursor_visible(true);
            },

            // Ignore all device events if the window does not have focus.
            Event::DeviceEvent { .. } if !window_focused => {}

            // Handle mouse motion.
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta, .. }, .. } => {
                input_context.handle_cursor_moved(&mut render_context, delta);
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    futures::executor::block_on(run(event_loop, window));
}
