use winit::window::Window;

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
    gpu_context: crate::gpu::GpuContext,

    next_frame_encoder: wgpu::CommandEncoder,

    world_geometry_manager: crate::world_geometry::WorldGeometryManager,

    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,

    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_sampler: wgpu::Sampler,

    depth_buffer: wgpu::Texture,
    depth_buffer_view: wgpu::TextureView,
    depth_buffer_sampler: wgpu::Sampler,

    camera: camera::Camera,
    // For now, this only stores the camera's matrix.
    uniform_buf: crate::managed_buffer::ManagedBuffer<f32, utils::Matrix4>,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,

    camera_dirty: bool,
}

impl RenderContext {
    // TODO: `Option` -> `Result`.
    pub async fn create(window: &Window) -> Option<RenderContext> {
        let gpu_context = crate::gpu::GpuContext::create(window).await.unwrap();

        // Create the command encoder used during initialization.
        let init_encoder = gpu_context.create_command_encoder();

        let world_geometry_manager = crate::world_geometry::WorldGeometryManager::new(&gpu_context)?;

        // Load the vertex and fragment shaders.
        let vs_module = gpu_context.create_shader_module_from_bytes(include_bytes!("../../shaders/shader.vert.spv"));
        let fs_module = gpu_context.create_shader_module_from_bytes(include_bytes!("../../shaders/shader.frag.spv"));

        // Create our texture and write it into a GPU buffer. Right now the texture is just a white image, but the
        // infrastructure is already in place to make better use of this data.
        let size = 64u32;
        let texels = utils::load_image_bytes("texture.png");
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };
        let texture = gpu_context.create_texture(&wgpu::TextureDescriptor {
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
        gpu_context.queue().write_texture(
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

        let (sc_width, sc_height) = gpu_context.size();
        // Create our depth buffer.
        let depth_buffer_size = wgpu::Extent3d {
            width: sc_width,
            height: sc_height,
            depth: 1,
        };
        let depth_buffer = gpu_context.create_texture(&wgpu::TextureDescriptor {
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
        let depth_buffer_sampler = gpu_context.create_sampler(&wgpu::SamplerDescriptor {
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

        let texture_sampler = gpu_context.create_sampler(&wgpu::SamplerDescriptor {
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
        let aspect_ratio = gpu_context.aspect_ratio();
        // This needs to be mutable because the camera has a matrix cache.
        // TODO: Can this be fixed? RefCell? Do we need an Arc? :(
        let mut camera = camera::Camera::new(
            // Start out at a nice vantage point looking toward the origin.
            cgmath::Point3::new(32.0, 32.0, 32.0),
            cgmath::Vector3::new(-1.0, -1.0, -1.0),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
            aspect_ratio,
            70.0,
            0.5,
            1000.0,
        );
        let camera_matrix: crate::utils::Matrix4 = camera.matrix().into();

        // Create the GPU buffer where we will store our shader uniforms.
        let uniform_buf = crate::managed_buffer::ManagedBuffer::new_uniform_buf_with_data(
            &gpu_context,
            camera_matrix,
        ).ok()?;

        // Set up our bind groups; this binds our data to named locations which are referenced in the shaders.
        let bind_group_layout = gpu_context.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                // Our 0th bind group is for small global data shared between all invocations of the shader. Currently,
                // this is the camera matrix. We set this bind group only once per frame
                wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::VERTEX,
                    wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                ),
                // Our 1st bind group is for texture data, which will be passed as an atlas. This may change a few times
                // per frame if we need to render from multiple atlases. TODO: are texture atlases the right way to do
                // this?
                wgpu::BindGroupLayoutEntry::new(
                    1,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
                // Our 2nd bind group is the sampler for the above texture. This is likely to change only when the
                // texture changes. TODO: is this true?
                wgpu::BindGroupLayoutEntry::new(
                    2,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::Sampler { comparison: false },
                ),
                // Our last bind group is a per object buffer, holding any data needed for an individual abstract object
                // being rendered. This might be a transform matrix, for instance.
                wgpu::BindGroupLayoutEntry::new(
                    3,
                    wgpu::ShaderStage::VERTEX,
                    wgpu::BindingType::UniformBuffer {
                        dynamic: true,
                        min_binding_size: wgpu::BufferSize::new(256),
                    },
                ),
            ],
        });

        let bind_group = gpu_context.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(world_geometry_manager.transforms_buf.slice(..)),
                },
            ],
            label: None,
        });

        // Set up our central render pipeline.
        let pipeline_layout = gpu_context.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let render_pipeline = gpu_context.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: utils::IVERTEX_SIZE as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Int3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Int3,
                            offset: 4*3,
                            shader_location: 1,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: 4*3 + 4*3,
                            shader_location: 2,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Uint,
                            offset: 4*3 + 4*3 + 4*2,
                            shader_location: 3,
                        },
                    ],
                }],
            },

            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        // Flush the initialization commands on the command queue.
        gpu_context.queue().submit(Some(init_encoder.finish()));

        let next_frame_encoder =
            gpu_context.create_command_encoder();

        Some(Self {
            gpu_context,
            next_frame_encoder,
            world_geometry_manager,
            vs_module,
            fs_module,
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
            camera_dirty: false,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Update our GPU context with the new width and height.
        self.gpu_context.resize(size);

        // Our aspect ratio might have changed, so we update our camera.
        self.camera.set_aspect_ratio(self.gpu_context.aspect_ratio());
    }

    pub fn render(&mut self) {
        let frame = self.gpu_context.get_next_frame().unwrap();

        // If the camera moved, we have to write the camera's data into the uniform buffer. We write
        // the data into the CPU side of our managed uniform buffer here.
        if self.camera_dirty {
            self.uniform_buf.replace_data(self.camera.matrix().into());
        }

        // This looks weird, but picture the future: a loop over some collection of buffers,
        // potentially flushing each one.
        if self.world_geometry_manager.vertex_buf.dirty() {
            self.world_geometry_manager.vertex_buf.enqueue_copy_command(&self.gpu_context, &mut self.next_frame_encoder);
        }
        if self.world_geometry_manager.index_buf.dirty() {
            self.world_geometry_manager.index_buf.enqueue_copy_command(&self.gpu_context, &mut self.next_frame_encoder);
        }
        if self.uniform_buf.dirty() {
            self.uniform_buf.enqueue_copy_command(&self.gpu_context, &mut self.next_frame_encoder);
        }


        {
            let mut render_pass = self.next_frame_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    }
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_buffer_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_index_buffer(self.world_geometry_manager.index_buf.slice(..));
            render_pass.set_vertex_buffer(0, self.world_geometry_manager.vertex_buf.slice(..));
            for i in 0..self.world_geometry_manager.chunks.len() as u32 {
                let chunk = &self.world_geometry_manager.chunks[i as usize];
                render_pass.set_bind_group(
                    0,
                    &self.bind_group,
                    &[(chunk.transform_index * self.world_geometry_manager.transforms_buf.t_size()) as u32],
                );
                render_pass.draw_indexed(chunk.index_offset as u32..(chunk.index_offset + chunk.index_count) as u32, chunk.vertex_offset as i32, 0..1);
            }
        }

        // Pull out the command encoder we have been using to build up this frame. We set up the next frame's encoder
        // at the same time.
        let final_encoder = std::mem::replace(
            &mut self.next_frame_encoder,
            self.gpu_context.create_command_encoder(),
        );

        self.gpu_context.submit_command_encoder(final_encoder);
    }

    // Expose raw mutation for some of the basic state variables.
    fn set_camera_dirty(&mut self) {
        self.camera_dirty = true;
    }

    #[allow(dead_code)]
    pub fn camera(&self) -> &camera::Camera {
        &self.camera
    }
    pub fn camera_mut(&mut self) -> &mut camera::Camera {
        // This is aggressive; we set the camera to dirty any time someone takes a mutable reference
        // to the camera; they do not have to mutate the camera. This is a negligible performance
        // hit.
        self.set_camera_dirty();
        &mut self.camera
    }
}
