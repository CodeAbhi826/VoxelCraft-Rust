//! Native entry point (Windows / Linux / macOS).
//! Run with: `cargo run --release`
//! Benchmark mode (Master Spec §37/§48 Phase 0):
//!   `cargo run --release -- --benchmark [frames=600] [warmup=120]
//!                                      [seed=12648430] [json=bench.json]`

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();

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

    if let Some(bench) = voxelcraft::bench::BenchState::from_args(&args) {
        // benchmark: deterministic fixed-seed world + auto-start the game
        // (title menus are skipped; the camera is scripted in update())
        app.start_bench(bench);
    }

    use winit::event_loop::ControlFlow;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| app.handle_event(event, elwt))
        .expect("event loop run");
}
