/// This type manages the lifetimes of its constituent fields. It will clean them up in `drop`.
#[allow(dead_code)]
struct DebugPassContext<'device> {
    device: &'device wgpu::Device,
    vertex_buf: wgpu::Buffer,
    vertex_buf_len: usize,

    index_buf: wgpu::Buffer,
    index_buf_len: usize,
}

/*
impl<'device> DebugPassContext {
    fn new<'device>(
        device: &'device wgpu::Device,
        debug_pass_type: DebugPassType,
    ) -> Self {

    }
}
*/

#[allow(dead_code)]
pub enum DebugPassType<'host> {
    /// Draws a slice of vector-position pairs. Once the data is copied onto the GPU via the
    /// construction of a `DebugPassContext`, the `'host` vector-position pairs may be dropped.
    DrawVectors(&'host [crate::utils::Vertex]),
}

/*
impl DebugPassType {
    fn build_context<'device>(&self, device: &'device wgpu::Device) -> DebugPassContext<'device> {
        match self {
            DebugPassType::DrawVectors(vectors) => {
                for (v, p) in vectors {
                }
            }
        }
    }
}
*/
