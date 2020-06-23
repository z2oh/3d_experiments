use winit::window::Window;

use crate::benchmark;
use crate::camera;
use crate::utils;

#[cfg(debug_assertions)]
mod debug_pass;

/// A `RenderContext` stores any state that is required for rendering a frame. This may include:
///
/// - camera position
/// - asset handles
/// - gpu buffer handles
/// - cached geometry
/// - render pipeline descriptions
/// - shader modules
/// - bind groups and layouts
///
/// Eventually an additional layer should be introduced to abstract all interfacing with the GPU.
#[allow(dead_code)]
pub struct RenderContext {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    next_frame_encoder: wgpu::CommandEncoder,

    amplitude: f64,
    frequency: f32,

    vertex_buf: crate::managed_buffer::ManagedBuffer<utils::Vertex>,
    index_buf: crate::managed_buffer::ManagedBuffer<u32>,

    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,

    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_sampler: wgpu::Sampler,

    depth_buffer: wgpu::Texture,
    depth_buffer_view: wgpu::TextureView,
    depth_buffer_sampler: wgpu::Sampler,

    camera: camera::Camera,

    uniform_buf: wgpu::Buffer,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,

    mesh_dirty: bool,
    uniform_dirty: bool,
}

impl RenderContext {
    // TODO: `Option` -> `Result`.
    pub async fn create(window: &Window) -> Option<RenderContext> {
        let size = window.inner_size();

        // Create the wgpu instance.
        let instance = wgpu::Instance::new();

        // Create the wgpu surface.
        let surface = unsafe { instance.create_surface(window) };

        // Create the wgpu adapter.
        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                },
                wgpu::UnsafeExtensions::disallow(),
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();

        // Create the device handle and the command queue handle for that device.
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: true,
        }, None)
        .await
        .unwrap();

        // Create the command encoder used during initialization.
        let init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Create our initial mesh.
        // These variables control the simplex noise generation of the voxel heightmap.
        let amplitude = 10.0f64;
        let frequency = 6.0f32;

        // Build the mesh; these are heap allocated `Vec`s.
        let (vertex_data, index_data) = utils::create_vertices(frequency);

        // Now we create the vertex buffer and index buffer on the GPU.
        let vertex_buf = crate::managed_buffer::ManagedBuffer::new_vertex_buf_with_data(
            &device,
            vertex_data,
        ).ok()?;
        let index_buf = crate::managed_buffer::ManagedBuffer::new_index_buf_with_data(
            &device,
            index_data,
        ).ok()?;

        // Load the vertex and fragment shaders.
        let vs = include_bytes!("../../shaders/shader.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());

        let fs = include_bytes!("../../shaders/shader.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        // Create our swapchain. The swapchain is an abstraction over a buffered pixel array which corresponds directly
        // to the image which is rendered onto the display.
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // Create our texture and write it into a GPU buffer. Right now the texture is just a white image, but the
        // infrastructure is already in place to make better use of this data.
        let size = 256u32;
        let texels = utils::create_texels(size);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });
        let texture_view = texture.create_default_view();
        // Place the texture data into a temporary copy buffer, and then immediately request a copy of it into a texture
        // buffer on the GPU. We wrap this in a lexical scope to avoid reusing `temp_buf`.
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &texels,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * size,
                rows_per_image: 0,
            },
            texture_extent,
        );

        // Create our depth buffer.
        let depth_buffer_size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            size: depth_buffer_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC,
            label: None,
        });
        let depth_buffer_view = depth_buffer.create_default_view();

        // Create the samplers.
        let depth_buffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create the camera and initialize it with sane defaults.
        let aspect_ratio = sc_desc.width as f32 / sc_desc.height as f32;
        // This needs to be mutable because the camera has a matrix cache.
        // TODO: Can this be fixed? RefCell? Do we need an Arc? :(
        let mut camera = camera::Camera::new(
            // Start out at a nice vantage point looking toward the origin.
            cgmath::Point3::new(32.0, 32.0, 32.0),
            cgmath::Vector3::new(-1.0, -1.0, -1.0),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
            aspect_ratio,
            80.0,
            1.0,
            1000.0,
        );
        let camera_matrix = camera.matrix();
        let camera_matrix_ref: &[f32; 16] = camera_matrix.as_ref();

        // Create the GPU buffer where we will store our shader uniforms.
        let uniform_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(camera_matrix_ref),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Set up our bind groups; this binds our data to named locations which are referenced in the shaders.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    ..Default::default()
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                    ..Default::default()
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    ..Default::default()
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView {
                        view: &texture_view,
                        read_only_depth_stencil: false,
                    },
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: None,
        });

        // Set up our central render pipeline.
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
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: utils::VERTEX_SIZE as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 4 * 4,
                            shader_location: 1,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: 4 * 4 + 4 * 3,
                            shader_location: 2,
                        },
                    ],
                }],
            },

            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        // Flush the initialization commands on the command queue.
        queue.submit(Some(init_encoder.finish()));

        let next_frame_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Some(Self {
            surface,
            adapter,
            device,
            queue,
            next_frame_encoder,
            amplitude,
            frequency,
            vertex_buf,
            index_buf,
            vs_module,
            fs_module,
            sc_desc,
            swap_chain,
            texture,
            texture_view,
            texture_sampler,
            depth_buffer,
            depth_buffer_view,
            depth_buffer_sampler,
            camera,
            uniform_buf,
            bind_group_layout,
            bind_group,
            pipeline_layout,
            render_pipeline,
            mesh_dirty: false,
            uniform_dirty: false,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Update our swap chain description with the new width and height and then create the new swap chain.
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        // Our aspect ratio might have changed, so we update our camera.
        self.camera.set_aspect_ratio(self.sc_desc.width as f32 / self.sc_desc.height as f32);
    }

    pub fn render(&mut self) {
        let frame = match self.swap_chain.get_next_frame() {
            Ok(frame) => frame,
            Err(_) => panic!("Failed to acquire next swap chain texture!"),
        };

        // If we need to regenerate the mesh, do so now, and write the data into the CPU side of
        // our managed buffers.
        if self.mesh_dirty {
            let (vertex_data, index_data) =
                benchmark!("Regenerating mesh", utils::create_vertices(self.frequency));
            self.vertex_buf.replace_data(vertex_data);
            self.index_buf.replace_data(index_data);
            self.mesh_dirty = false;
        }

        // This looks weird, but picture the future: a loop over some collection of buffers,
        // potentially flushing each one.
        if self.vertex_buf.dirty() {
            self.vertex_buf.enqueue_copy_command(&self.device, &mut self.next_frame_encoder);
        }
        if self.index_buf.dirty() {
            self.vertex_buf.enqueue_copy_command(&self.device, &mut self.next_frame_encoder);
        }

        if self.uniform_dirty {
            let camera_matrix = self.camera.matrix();
            let camera_matrix_ref: &[f32; 16] = camera_matrix.as_ref();
            let temp_buf =
                self.device.create_buffer_with_data(bytemuck::cast_slice(camera_matrix_ref), wgpu::BufferUsage::COPY_SRC);
            self.next_frame_encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.uniform_buf, 0, 64);
            self.uniform_dirty = false;
        }

        // Go ahead and pull out the command encoder we have been using to build up this frame. We set up the next
        // frame's encoder at the same time.
        let mut next_frame_encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        std::mem::swap(&mut self.next_frame_encoder, &mut next_frame_encoder);

        {
            let mut render_pass = next_frame_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_buffer_view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    depth_read_only: false,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    stencil_read_only: false,
                    clear_stencil: 0,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_index_buffer(self.index_buf.slice(..));
            render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            render_pass.draw_indexed(0..self.index_buf.len() as u32, 0, 0..1);
        }

        self.queue.submit(Some(next_frame_encoder.finish()));
    }

    // Expose raw mutation for some of the basic state variables.
    pub fn set_mesh_dirty(&mut self) {
        self.frequency += 0.05;
        self.mesh_dirty = true;
    }
    pub fn set_uniform_dirty(&mut self) {
        self.uniform_dirty = true;
    }
    pub fn amplitude(&self) -> f64 {
        self.amplitude
    }
    pub fn set_amplitude(&mut self, amplitude: f64) {
        self.amplitude = amplitude
    }
    pub fn frequency(&self) -> f32 {
        self.frequency
    }
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency
    }

    #[allow(dead_code)]
    pub fn camera(&self) -> &camera::Camera {
        &self.camera
    }
    pub fn camera_mut(&mut self) -> &mut camera::Camera {
        self.set_uniform_dirty();
        &mut self.camera
    }
}
