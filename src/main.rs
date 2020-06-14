use winit::{
    event,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

mod simplex;
mod render_context;
use render_context::RenderContext;
mod utils;

async fn run(event_loop: EventLoop<()>, window: Window) {
    env_logger::init();
    let mut render_context = RenderContext::create(&window).await.unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => render_context.resize(size),
            Event::RedrawRequested(_) => render_context.render(),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                } | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::J),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    render_context.set_amplitude(render_context.amplitude() * 1.1);
                    render_context.set_dirty();
                },
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::L),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    render_context.set_frequency(render_context.frequency() * 1.1);
                    render_context.set_dirty();
                },
                _ => {},
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
