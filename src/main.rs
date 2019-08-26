use rendy::{
    mesh::{Color, PosColor, Position},
    factory::{Config, Factory},
    wsi::winit::{EventsLoop, WindowBuilder, Event, WindowEvent},
};
#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;
#[cfg(feature = "metal")]
type Backend = rendy::metal::Backend;
#[cfg(feature = "vulkan")]
type Backend = rendy::vulkan::Backend;

pub const WINDOW_NAME: &str = "rust-game-engine";

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
fn main() {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_NAME)
        .with_dimensions((800, 600).into())
        .build(&events_loop)
        .expect("Window creation failed.");
    
    let config: Config = Default::default();
    let (mut factory, mut families): (Factory<Backend>, _) =
        rendy::factory::init(config).expect("Factory creation failed.");
    
    let mut running = true;
    while running 
    {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event: w, .. } => match w {
                    WindowEvent::CloseRequested => running = false,
                    _ => (),
                } 
                _ => (),
            }
        });
    }
}

// when no features aren't enabled, print error
#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
fn main() {
    println!("Please enable one of the backend features: dx12, metal, vulkan");
}