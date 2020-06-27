#[allow(dead_code)]
/// This error type is wholly unused, but I am leaving it her for potential future use.
pub enum ManagedBufferError {
    VertexBufError,
    IndexBufError,
}

// TODO: Keep track of 'gpu life, since this manages a raw buffer.
// The `host_data` field was previously a Vec<T> to avoid dealing with this generic type parameter,
// but sometimes we don't want a heap allocated type (like when dealing when small buffers, like for
// shaders), and so we are forced to be generic. This results in a kind of ugly type specifier,
// where the type of T needs to be given twice. Is there some way to avoid this?
pub struct ManagedBuffer<T, Own: AsRef<[T]>>
  where T: bytemuck::Pod + bytemuck::Zeroable
{
    /// This flag is checked on render to see if the buffer needs to be recopied to GPU.
    dirty: bool,
    /// A managed pointer to the data in CPU memory.
    host_data: Own,
    /// The wgpu *handle* to the underlying raw buffer.
    raw: wgpu::Buffer,

    // We don't actually use T in any of the fields, so we need this zero-size field to placate the
    // compiler.
    _type: std::marker::PhantomData<T>,
}

impl<'cpu, Own: AsRef<[T]>, T: bytemuck::Pod + bytemuck::Zeroable> ManagedBuffer<T, Own> {
    #[inline(always)]
    pub fn t_size(&self) -> usize {
        std::mem::size_of::<T>()
    }

    /// Replaces the data in the CPU memory of the ManagedBuffer. This function will not trigger a
    /// write into the GPU, but it will set the `dirty` flag. This flag should be checked in any
    /// place where an up-to-date buffer needs to be used, and if the buffer is dirty, it may be
    /// flushed using the `enqueue_copy_write_command` function to write the data into the GPU.
    pub fn replace_data(&mut self, new_data: Own) -> Option<Own> {
        if new_data.as_ref().len() == self.host_data.as_ref().len() {
            self.dirty = true;
            Some(std::mem::replace(&mut self.host_data, new_data))
        } else {
            None
        }
    }

    /// Create a new vertex buffer with some provided input data. This object manages the data on
    /// both the CPU and the GPU. This buffer is `COPY_DST`, so it can be written to. If the desired
    /// buffer is immutable, this is not the function to use.
    pub fn new_vertex_buf_with_data(
        device: &wgpu::Device,
        host_data: Own,
    ) -> Result<ManagedBuffer<T, Own>, ManagedBufferError> {
        let raw = device.create_buffer_with_data(
            bytemuck::cast_slice(host_data.as_ref()),
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        );
        Ok(ManagedBuffer {
            dirty: true,
            host_data,
            raw,
            _type: std::marker::PhantomData,
        })
    }

    /// Create a new index buffer with some provided input data. This object manages the data on
    /// both the CPU and the GPU. This buffer is `COPY_DST`, so it can be written to. If the desired
    /// buffer is immutable, this is not the function to use.
    pub fn new_index_buf_with_data(
        device: &wgpu::Device,
        host_data: Own,
    ) -> Result<ManagedBuffer<T, Own>, ManagedBufferError> {
        let raw = device.create_buffer_with_data(
            bytemuck::cast_slice(host_data.as_ref()),
            wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
        );
        Ok(ManagedBuffer {
            dirty: true,
            host_data,
            raw,
            _type: std::marker::PhantomData,
        })
    }

    /// Create a new uniform buffer with some provided input data. This object manages the data on
    /// both the CPU and the GPU. This buffer is `COPY_DST`, so it can be written to. If the desired
    /// buffer is immutable, this is not the function to use.
    pub fn new_uniform_buf_with_data(
        device: &wgpu::Device,
        host_data: Own,
    ) -> Result<ManagedBuffer<T, Own>, ManagedBufferError> {
        let raw = device.create_buffer_with_data(
            bytemuck::cast_slice(host_data.as_ref()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        Ok(ManagedBuffer {
            dirty: true,
            host_data,
            raw,
            _type: std::marker::PhantomData,
        })
    }

    /// Returns a wgpu::BufferSlice for portion of the buffer specified by the bounds.
    pub fn slice<S>(&self, bounds: S) -> wgpu::BufferSlice
        where S: std::ops::RangeBounds<wgpu::BufferAddress>
    {
        self.raw.slice(bounds)
    }

    // Some convenience accessors.

    /// Returns the length of the host data. This is measured in number of `T`s, *not* number of
    /// bytes.
    pub fn len(&self) -> usize {
        self.host_data.as_ref().len()
    }

    /// Returns true if the buffer is dirty and needs to be flushed to GPU.
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Enqueues a command onto the encoder to copy the buffer from CPU to GPU. This will not issue
    /// a write unless the buffer is dirty, and so can safely be called in a render loop.
    ///
    /// Calling this function will reset the dirty flag. Be sure that you finish the command encoder
    /// and submit it on a queue.
    pub fn enqueue_copy_command(
        &mut self, device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if !self.dirty { return }

        let stage_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(self.host_data.as_ref()),
            wgpu::BufferUsage::COPY_SRC
        );

        encoder.copy_buffer_to_buffer(
            &stage_buffer, 0, &self.raw, 0, (self.host_data.as_ref().len() * self.t_size()) as u64
        );

        // We are setting the dirty flag to false here trusting that the caller will actually
        // finish the command encoder and submit it on the queue!
        self.dirty = false;
    }
}
