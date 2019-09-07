use failure::Error;

use gfx_hal::{
    format::Format,
    pso::{DepthStencilDesc, Element, ElemStride, VertexInputRate},
};

use rendy::{
    command::{QueueId, RenderPassEncoder, Families},
    mesh::{Color, PosColor, Position},
    factory::{Config, Factory},
    wsi::winit::{EventsLoop, WindowBuilder, Event, WindowEvent},
    graph::{
        render::{SimpleGraphicsPipelineDesc, SimpleGraphicsPipeline, PrepareResult, RenderGroupBuilder},
        GraphContext, NodeBuffer, NodeImage, GraphBuilder, Graph
    },
    memory::{Dynamic},
    resource::{Handle, Escape, DescriptorSetLayout, Buffer, BufferInfo},
    shader::{ShaderKind, ShaderSet, SourceLanguage, SpirvShader, SourceShaderInfo},
};

#[cfg(feature = "spirv-reflection")]
use rendy::shader::SpirvReflection;

#[cfg(not(feature = "spirv-reflection"))]
use rendy::mesh::AsVertex;

use lazy_static;

#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;
#[cfg(feature = "metal")]
type Backend = rendy::metal::Backend;
#[cfg(feature = "vulkan")]
type Backend = rendy::vulkan::Backend;

pub const WINDOW_NAME: &str = "rust-game-engine";

const VERTEX_DATA: [PosColor; 3] = [
    PosColor {
        position: Position([-0.5, 0.5, 0.0]),
        color: Color([1.0, 0.0, 0.0, 1.0]),
    },
    PosColor {
        position: Position([0.0, -0.5, 0.0]),
        color: Color([0.0, 1.0, 0.0, 1.0]),
    },
    PosColor {
        position: Position([0.5, 0.5, 0.0]),
        color: Color([0.0, 0.0, 1.0, 1.0]),
    },
];

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("./shaders/vert.glsl"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/vert.glsl").into(),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().expect("Vertex shader Spir-V pre-compilation failed!");

    static ref FRAGMENT: SpirvShader = SourceShaderInfo::new(
        include_str!("./shaders/frag.glsl"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/vert.glsl").into(),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().expect("Fragment shader Spir-V pre-compilation failed!");

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
}

#[cfg(feature = "spirv-reflection")]
lazy_static::lazy_static! {
    static ref SHADER_REFLECTION: SpirvReflection = SHADERS.reflect().expect("Spir-V shader reflection failed!");
}

#[derive(Debug, Default)]
struct TriangleRenderPipelineDesc;


#[derive(Debug)]
struct TriangleRenderPipeline<B> where B: gfx_hal::Backend
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

    fn dispose(self, factory: &mut Factory<B>, aux: &T){}
}

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
fn run(
    mut events_loop: EventsLoop,
    mut factory: Factory<Backend>,
    mut families: Families<Backend>,
    graph: Graph<Backend, ()>,
) 
{
    let started = std::time::Instant::now();

    let mut frame = 0u64;
    let mut elapsed = started.elapsed();
    let mut graph = Some(graph);

    let mut running = true;
    while running 
    {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event: w, .. } => match w {
                    WindowEvent::CloseRequested => running = false,
                    _ => {},
                }, 
                _ => (),
            }
        });

        factory.maintain(&mut families);
        if let Some(ref mut graph) = graph {
            graph.run(&mut factory, &mut families, &());
            frame += 1;
        }
        elapsed = started.elapsed();
        //TODO: Add timeout?
    }
    //once done running, log time, frames, fps; dispose of graph if needed
    if graph.is_some()
    {
        let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;

        log::info!(
            "Elapsed: {:?}. Frames: {}. FPS: {}",
            elapsed,
            frame,
            frame * 1_000_000_000 / elapsed_ns
        );

        graph.take().expect("Graph disposal failed!").dispose(&mut factory, &());
    }
}

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
fn main() 
{
    env_logger::init();
    
    let config: Config = Default::default();
    
    let (mut factory, mut families): (Factory<Backend>, _) =
        rendy::factory::init(config).expect("Factory creation failed!");
    
    let events_loop = EventsLoop::new();
    
    let window = WindowBuilder::new()
        .with_title(WINDOW_NAME)
        .with_dimensions((800, 600).into())
        .build(&events_loop)
        .expect("Window creation failed.");
    
    let surface = factory.create_surface(&window);

    let mut graph_builder = GraphBuilder::<Backend, ()>::new();

    graph_builder.add_node(
        TriangleRenderPipeline::builder()
            .into_subpass()
            .with_color_surface()
            .into_pass()
            .with_surface(
                surface,
                Some(gfx_hal::command::ClearValue::Color([0.0, 0.0, 0.0, 1.0].into()))
            ),
    );

    let graph = graph_builder
        .build(&mut factory, &mut families, &())
        .expect("Graph creation failed!");

    run(events_loop, factory, families, graph);
}

// when no features aren't enabled, print error
#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
fn main() 
{
    println!("Please enable one of the backend features: dx12, metal, vulkan");
}