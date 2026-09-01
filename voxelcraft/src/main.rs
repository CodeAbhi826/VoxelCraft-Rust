//! Native entry point (Windows / Linux / macOS).
//! Run with: `cargo run --release`

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("event loop");
    let window = winit::window::WindowBuilder::new()
        .with_title("VoxelCraft — Rust + wgpu voxel engine")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .with_resizable(true)
        .build(&event_loop)
        .expect("window");
    // single-window app: process-lifetime reference (also gives the wgpu
    // surface a 'static lifetime without cloning)
    let window: &'static winit::window::Window = Box::leak(Box::new(window));

    let mut app = pollster::block_on(voxelcraft::game::GameApp::new(window));

    use winit::event_loop::ControlFlow;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| app.handle_event(event, elwt))
        .expect("event loop run");
}
