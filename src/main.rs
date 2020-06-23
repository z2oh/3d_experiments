use std::thread;
use std::sync::mpsc::channel;

use winit::{
    event,
    event::{Event, DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

mod camera;
mod input;
mod managed_buffer;
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

    let (control_send, control_receive) = channel();
    let (data_send, data_receive) = channel();
    thread::spawn(move || {
        let start = std::time::Instant::now();
        loop {
            match control_receive.recv() {
                Ok(0) => {
                    let duration = std::time::Instant::now() - start;
                    let mod_time = (duration.as_millis() % (1 << 16)) as f32;
                    // This will block on the creation of the mesh.
                    let mesh = benchmark!("Mesh gen took", crate::utils::create_vertices(mod_time / 1000.0));
                    if let Err(_) = data_send.send(mesh) {
                        break
                    }
                },
                Ok(1) => break,
                _ => break,
            }
        }
    });
    // Get things going...
    if let Err(_) = control_send.send(0) {
        return
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                if now - prev_frame > six_ms {
                    prev_frame = now;
                }
                render_context.set_mesh_dirty();
                // See if our next mesh data is ready yet...
                let new_data = match data_receive.try_recv() {
                    Ok(data) => {
                        // Request more data!
                        if let Err(_) = control_send.send(0) {
                            *control_flow = ControlFlow::Exit;
                        }
                        Some(data)
                    }
                    _ => None,
                };
                render_context.render(new_data);
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => render_context.resize(size),
            // Handle requests to close the window...
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } |
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: event::KeyboardInput {
                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                state: event::ElementState::Pressed, ..
            }, .. }, .. } => {
                *control_flow = ControlFlow::Exit;

                // Tell the thread to stop and then block on it.
                control_send.send(1).unwrap();

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
    window.set_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
    futures::executor::block_on(run(event_loop, window));
}
