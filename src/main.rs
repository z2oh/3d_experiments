use winit::{
    event,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

mod simplex;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

fn vertex(pos: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_cuboid(offset_x: f32, offset_y: f32, offset_z: f32, width: f32, length: f32, height: f32, index_offset: u32) -> ([Vertex; 24], [u32; 36]) {
    ([
        // top (0, 0,
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [0.0, 0.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [1.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [1.0, 1.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [0.0, 1.0]),
        // bottom (0.0, 0.0, -1)
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [1.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [0.0, 0.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [0.0, 1.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [1.0, 1.0]),
        // right (1.0, 0.0, 0)
        vertex([1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [0.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [1.0, 0.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [1.0, 1.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [0.0, 1.0]),
        // left (-1.0, 0.0, 0)
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [1.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [0.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [0.0, 1.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [1.0, 1.0]),
        // front (0.0, 1.0, 0)
        vertex([1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [1.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, -1.0*height+offset_z], [0.0, 0.0]),
        vertex([-1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [0.0, 1.0]),
        vertex([1.0*width+offset_x, 1.0*length+offset_y, 1.0*height+offset_z], [1.0, 1.0]),
        // back (0.0, -1.0, 0)
        vertex([1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [0.0, 0.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, 1.0*height+offset_z], [1.0, 0.0]),
        vertex([-1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [1.0, 1.0]),
        vertex([1.0*width+offset_x, -1.0*length+offset_y, -1.0*height+offset_z], [0.0, 1.0]),
    ],
    [
        0+index_offset, 1+index_offset, 2+index_offset, 2+index_offset, 3+index_offset, 0+index_offset, // top
        4+index_offset, 5+index_offset, 6+index_offset, 6+index_offset, 7+index_offset, 4+index_offset, // bottom
        8+index_offset, 9+index_offset, 10+index_offset, 10+index_offset, 11+index_offset, 8+index_offset, // right
        12+index_offset, 13+index_offset, 14+index_offset, 14+index_offset, 15+index_offset, 12+index_offset, // left
        16+index_offset, 17+index_offset, 18+index_offset, 18+index_offset, 19+index_offset, 16+index_offset, // front
        20+index_offset, 21+index_offset, 22+index_offset, 22+index_offset, 23+index_offset, 20+index_offset, // back
    ])
}

fn create_vertices(amplitude: f64, frequency: f32) -> (Vec<Vertex>, Vec<u32>) {
    let prng_base = simplex::Simplex::with_seed(0);
    let mut index_offset = 0;
    let mut vertex_accum: Vec<Vertex> = Vec::with_capacity(100 * 100 * 24);
    let mut index_accum: Vec<u32> = Vec::with_capacity(100 * 100 * 36);
    for y in 0..100 {
        for x in 0..100 {
            let x = (x as f32) * 2.0;
            let y = (y as f32) * 2.0;
            let z = (prng_base.get2d((x / frequency) as f64, (y / frequency) as f64) * amplitude as f64) as f32;
            let (vertices_next, indices_next) = create_cuboid(x, y, z, 1.0, 1.0, 8.0, index_offset);
            index_offset += 24;
            vertex_accum.extend(vertices_next.iter());
            index_accum.extend(indices_next.iter());
        }
    }

    (vertex_accum, index_accum)
}

fn create_texels(_size: u32) -> Vec<u8> {
    // This only works because "white.png" happens to be exactly the right size, 256x256 in this case.
    let image = image::open("white.png").unwrap();
    image.to_rgba().into_raw()
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

fn generate_matrix(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 1000.0);
    let mx_view = cgmath::Matrix4::look_at(
        cgmath::Point3::new(300.0, 300.0, 50.0),
        cgmath::Point3::new(0.0, 0.0, 0.0),
        cgmath::Vector3::unit_z(),
    );
    let mx_correction = OPENGL_TO_WGPU_MATRIX;
    mx_correction * mx_projection * mx_view
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let surface = wgpu::Surface::create(&window);

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        },
        wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    })
    .await;

    use std::mem;

    let mut init_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // Create the vertex and index buffers
    let vertex_size = mem::size_of::<Vertex>();
    let mut amplitude = 10.0f64;
    let mut frequency = 6.0f32;
    let (vertex_data, index_data) = create_vertices(amplitude, frequency);

    let vertex_buf = device.create_buffer_with_data(
        bytemuck::cast_slice(&vertex_data),
        wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
    );

    let index_buf = device
        .create_buffer_with_data(bytemuck::cast_slice(&index_data), wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST);

    let vs = include_bytes!("../shaders/shader.vert.spv");
    let vs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());

    let fs = include_bytes!("../shaders/shader.frag.spv");
    let fs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    // Create the texture
    let size = 256u32;
    let texels = create_texels(size);
    let texture_extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        label: None,
    });
    let texture_view = texture.create_default_view();
    let temp_buf =
        device.create_buffer_with_data(texels.as_slice(), wgpu::BufferUsage::COPY_SRC);
    init_encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            bytes_per_row: 4 * size,
            rows_per_image: 0,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        texture_extent,
    );

    // Create other resources
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare: wgpu::CompareFunction::Undefined,
    });

    // Generate starting camera view.
    let mx_total = generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    let uniform_buf = device.create_buffer_with_data(
        bytemuck::cast_slice(mx_ref),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buf,
                    range: 0..mx_ref.len() as u64,
                },
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::Binding {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: None,
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: vertex_size as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 4 * 4,
                        shader_location: 1,
                    },
                ],
            }],
        },

        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    queue.submit(&[init_encoder.finish()]);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                let command_buf = resize(&sc_desc, &device, &uniform_buf);
                queue.submit(&[command_buf]);
            }
            Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_index_buffer(&index_buf, 0, 0);
                    rpass.set_vertex_buffer(0, &vertex_buf, 0, 0);
                    rpass.draw_indexed(0..index_data.len() as u32, 0, 0..1);
                }

                queue.submit(&[encoder.finish()]);
            },
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
                    amplitude *= 1.1;
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    let (vertex_data, index_data) = create_vertices(amplitude, frequency);
                    let temp_v_buf = device.create_buffer_with_data(bytemuck::cast_slice(&vertex_data), wgpu::BufferUsage::COPY_SRC);
                    let temp_i_buf = device.create_buffer_with_data(bytemuck::cast_slice(&index_data), wgpu::BufferUsage::COPY_SRC);
                    encoder.copy_buffer_to_buffer(&temp_v_buf, 0, &vertex_buf, 0, (vertex_data.len() * 24) as u64);
                    encoder.copy_buffer_to_buffer(&temp_i_buf, 0, &index_buf, 0, index_data.len() as u64);

                    queue.submit(&[encoder.finish()]);
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
                    frequency *= 1.1;
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    let (vertex_data, index_data) = create_vertices(amplitude, frequency);
                    let temp_v_buf = device.create_buffer_with_data(bytemuck::cast_slice(&vertex_data), wgpu::BufferUsage::COPY_SRC);
                    let temp_i_buf = device.create_buffer_with_data(bytemuck::cast_slice(&index_data), wgpu::BufferUsage::COPY_SRC);
                    encoder.copy_buffer_to_buffer(&temp_v_buf, 0, &vertex_buf, 0, (vertex_data.len() * 24) as u64);
                    encoder.copy_buffer_to_buffer(&temp_i_buf, 0, &index_buf, 0, index_data.len() as u64);

                    queue.submit(&[encoder.finish()]);
                },
                _ => {},
            }
            _ => {}
        }
    });
}

fn resize(
    sc_desc: &wgpu::SwapChainDescriptor,
    device: &wgpu::Device,
    uniform_buf: &wgpu::Buffer,
) -> wgpu::CommandBuffer {
    let mx_total = generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
    let mx_ref: &[f32; 16] = mx_total.as_ref();

    let temp_buf =
        device.create_buffer_with_data(bytemuck::cast_slice(mx_ref), wgpu::BufferUsage::COPY_SRC);

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_buffer(&temp_buf, 0, &uniform_buf, 0, 64);
    encoder.finish()
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    futures::executor::block_on(run(event_loop, window));
}
