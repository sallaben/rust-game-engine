use rendy::{
    command::{Families},
    mesh::{Color, PosColor, Position},
    factory::{Config, Factory},
    wsi::winit::{EventsLoop, WindowBuilder, Event, WindowEvent},
    graph::{
        render::{SimpleGraphicsPipeline, RenderGroupBuilder},
        GraphBuilder, Graph
    },
    shader::{ShaderKind, SourceLanguage, SpirvShader, SourceShaderInfo},
};

#[cfg(feature = "spirv-reflection")]
use rendy::shader::SpirvReflection;


use lazy_static;

mod graphics_pipeline;

#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;
#[cfg(feature = "metal")]
type Backend = rendy::metal::Backend;
#[cfg(feature = "vulkan")]
type Backend = rendy::vulkan::Backend;
#[cfg(feature = "gl")]
type Backend = gfx_backend_gl::Backend;

//type Backend = gfx_backend_vulkan::Backend;


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
    static ref SHADER_REFLECTION: SpirvReflection = SHADERS.reflect().unwrap();
}

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan", feature = "gl"))]
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

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan", feature = "gl"))]
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
        graphics_pipeline::TriangleRenderPipeline::builder()
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
#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan", feature = "gl")))]
fn main() 
{
    println!("Please enable one of the backend features: dx12, metal, vulkan");
}
