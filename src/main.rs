use winit::{
    event,
    event::{Event, WindowEvent},
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => render_context.render(),

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => render_context.resize(size),
            // Handle requests to close the window...
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: event::KeyboardInput {
                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                state: event::ElementState::Pressed, ..
            }, .. }, .. } => *control_flow = ControlFlow::Exit,

            // Other keypresses go to the input handler.
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: event::KeyboardInput {
                virtual_keycode: Some(keycode),
                state: event::ElementState::Pressed, ..
            }, .. }, .. } => input_context.handle_key(&mut render_context, keycode),

            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } =>
                input_context.handle_cursor_moved(&mut render_context, position),
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    futures::executor::block_on(run(event_loop, window));
}
