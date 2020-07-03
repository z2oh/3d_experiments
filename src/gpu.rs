/// This struct is the base level abstraction to the GPU. It is responsible for managing the render
/// surface, the swap chain, the device, and the device queue.
pub struct GpuContext {
    // TODO: does this field need to be here?
    #[allow(dead_code)]
    instance: wgpu::Instance,

    surface: wgpu::Surface,
    // TODO: does this field need to be here?
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

#[derive(Debug)]
pub enum GpuContextError {
    RequestAdapterError,
    RequestDeviceError(wgpu::RequestDeviceError),
    SwapChainError(wgpu::SwapChainError),
}

impl std::fmt::Display for GpuContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuContextError::RequestAdapterError =>
                write!(f, "Adapter request failed! `wgpu` returned `None`."),
            GpuContextError::RequestDeviceError(wgpu_err) =>
                write!(f, "Device request failed! `wgpu` error is: {:?}", wgpu_err),
            GpuContextError::SwapChainError(wgpu_err) =>
                write!(f, "Swap chain operation failed! `wgpu` error is: {:?}", wgpu_err),
        }
    }
}

impl std::error::Error for GpuContextError {}

impl GpuContext {
    /// Create a new `GpuContext` on the provided window.
    pub async fn create(window: &winit::window::Window) -> Result<GpuContext, GpuContextError> {
        let size = window.inner_size();

        // TODO: These features will eventually need to be parameters.
        let unsafe_features = wgpu::UnsafeFeatures::disallow();
        let required_features = wgpu::Features::empty();

        // Create the wgpu instance.
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        // Create the wgpu surface.
        let surface = unsafe { instance.create_surface(window) };

        // Create the wgpu adapter.
        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                },
                unsafe_features,
            )
            .await
            .ok_or(GpuContextError::RequestAdapterError)?;

        let adapter_features = adapter.features();

        // Create the device handle and the command queue handle for that device.
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            features: adapter_features & required_features,
            limits: wgpu::Limits::default(),
            shader_validation: true,
        }, None)
        .await
        .map_err(|e| GpuContextError::RequestDeviceError(e))?;

        // Create our swapchain. The swapchain is an abstraction over a buffered pixel array which corresponds directly
        // to the image which is rendered onto the display.
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
        })
    }

    //
    // Functions with logic.
    //

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.swap_chain_desc.width = size.width;
        self.swap_chain_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

    /// N.B. This function will panic if `bytes` is invalid SPIR-V bytecode.
    pub fn create_shader_module_from_bytes(&self, bytes: &[u8]) -> wgpu::ShaderModule {
        let spirv = wgpu::util::make_spirv(bytes);
        self.device.create_shader_module(spirv)
    }

    pub fn get_next_frame(&mut self) -> Result<wgpu::SwapChainFrame, GpuContextError> {
        self.swap_chain.get_next_frame().map_err(|e| GpuContextError::SwapChainError(e))
    }

    pub fn submit_command_encoder(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(Some(encoder.finish()));
    }

    //
    // Forwarding functions.
    //

    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> wgpu::Texture {
        self.device.create_texture(desc)
    }

    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        self.device.create_sampler(desc)
    }

    pub fn create_bind_group_layout(&self, desc: &wgpu::BindGroupLayoutDescriptor) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(desc)
    }

    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> wgpu::BindGroup {
        self.device.create_bind_group(desc)
    }

    pub fn create_pipeline_layout(&self, desc: &wgpu::PipelineLayoutDescriptor) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(desc)
    }

    pub fn create_render_pipeline(&self, desc: &wgpu::RenderPipelineDescriptor) -> wgpu::RenderPipeline {
        self.device.create_render_pipeline(desc)
    }

    pub fn create_buffer_with_data(&self, data: &[u8], usage: wgpu::BufferUsage) -> wgpu::Buffer {
        self.device.create_buffer_with_data(data, usage)
    }

    //
    // Unknown.
    //

    // TODO: Determine what functionality is needed from Queue and provide interfaces here, rather
    // than provide raw queue access.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Gets the aspect ratio of the current swap chain.
    // TODO: This feels weird to have here...
    pub fn aspect_ratio(&self) -> f32 {
        self.swap_chain_desc.width as f32 / self.swap_chain_desc.height as f32
    }

    /// Gets the dimensions of the current swap chain.
    // TODO: This feels weird to have here...
    pub fn size(&self) -> (u32, u32) {
        (self.swap_chain_desc.width, self.swap_chain_desc.height)
    }
}
