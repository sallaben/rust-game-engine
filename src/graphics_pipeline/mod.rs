use failure::Error;

use gfx_hal::{
    format::Format,
    pso::{DepthStencilDesc, Element, ElemStride, VertexInputRate},
};

use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::{Factory},
    mesh::{PosColor, AsVertex},
    graph::{
        render::{SimpleGraphicsPipelineDesc, SimpleGraphicsPipeline, PrepareResult},
        GraphContext, NodeBuffer, NodeImage
    },
    memory::{Dynamic},
    resource::{Handle, Escape, DescriptorSetLayout, Buffer, BufferInfo},
    shader::{ShaderSet},
};

use crate::{
    VERTEX_DATA, SHADERS
};

#[cfg(feature = "spirv-reflection")]
use crate::{
    SHADER_REFLECTION
};

#[derive(Debug, Default)]
pub struct TriangleRenderPipelineDesc;

#[derive(Debug)]
pub struct TriangleRenderPipeline<B> where B: gfx_hal::Backend
{
    vertex_buffer: Option<Escape<Buffer<B>>>,
}

impl<B, T> SimpleGraphicsPipelineDesc<B, T> for TriangleRenderPipelineDesc
where
    B: gfx_hal::Backend,
    T: ?Sized,
{
    type Pipeline = TriangleRenderPipeline<B>;

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &T) -> ShaderSet<B> 
    {
        SHADERS.build(factory, Default::default()).expect("Shader set load failed!")
    }

    fn depth_stencil(&self) -> Option<DepthStencilDesc> { None }

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, ElemStride, VertexInputRate)> 
    {
        #[cfg(feature = "spirv-reflection")]
        return vec![SHADER_REFLECTION
            .attributes_range(..)
            .expect("Spir-V reflection vertex retrieval failed!")
            .gfx_vertex_input_desc(gfx_hal::pso::VertexInputRate::Vertex)];
        
        #[cfg(not(feature = "spirv-reflection"))]
        return vec![PosColor::vertex().gfx_vertex_input_desc(gfx_hal::pso::VertexInputRate::Vertex)];
    }

    fn build
    (
        self,
        _ctx: &GraphContext<B>,
        _factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &T,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, Error> 
    {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert!(set_layouts.is_empty());

        Ok(TriangleRenderPipeline { vertex_buffer: None })
    }
}

impl<B, T> SimpleGraphicsPipeline<B, T> for TriangleRenderPipeline<B>
where
    B: gfx_hal::Backend,
    T: ?Sized,
{
    type Desc = TriangleRenderPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &T,
    ) -> PrepareResult
    {
        if self.vertex_buffer.is_none() {
            println!("Creating vertex buffer!");

            #[cfg(feature = "spirv-reflection")]
            let vbuf_size = SHADER_REFLECTION.attributes_range(..).expect("Shader attribute range retrieval for buffer failed!").stride as u64 * VERTEX_DATA.len() as u64;

            #[cfg(not(feature = "spirv-reflection"))]
            let vbuf_size = PosColor::vertex().stride as u64 * VERTEX_DATA.len() as u64;

            let buf_info = BufferInfo {
                size: vbuf_size,
                usage: gfx_hal::buffer::Usage::VERTEX,
            };

            println!("{:?}", buf_info);

            let mut vertex_buffer = factory
                .create_buffer(
                    buf_info,
                    Dynamic,
                ).expect("Vertex buffer creation failed!");
            
            println!("Uploading vertex buffer!");
            unsafe {
                factory
                    .upload_visible_buffer(&mut vertex_buffer, 0, &VERTEX_DATA)
                    .expect("Vertex data upload failed!");
            }

            self.vertex_buffer = Some(vertex_buffer);
        }
        PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        _layout: &<B as gfx_hal::Backend>::PipelineLayout,
        mut encoder: RenderPassEncoder<B>,
        _index: usize,
        _aux: &T,
    )
    {
        let vb = self.vertex_buffer.as_ref().unwrap();
        unsafe {
            encoder.bind_vertex_buffers(0, Some((vb.raw(), 0)));
            encoder.draw(0..3, 0..1);
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &T){}
}
