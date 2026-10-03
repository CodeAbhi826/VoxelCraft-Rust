//! Native entry point (Windows / Linux / macOS).
//! Run with: `cargo run --release`
//! Benchmark mode (Master Spec §37/§48 Phase 0):
//!   `cargo run --release -- --benchmark [frames=600] [warmup=120]
//!                                      [seed=12648430] [json=bench.json]`

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // F3 "Allocated" telemetry: process-wide counting allocator (wraps the
    // system allocator; only >=4 KiB allocations hit the atomic counters)
    #[global_allocator]
    static ALLOCATOR: voxelcraft::alloc_stats::Counting = voxelcraft::alloc_stats::Counting;

    let args: Vec<String> = std::env::args().collect();

    // --help / -h: the usage card (every flag is real — kept in sync
    // with bench.rs's --benchmark parser and the --smoke contract)
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "VoxelCraft {} — Rust/wgpu voxel engine (single file, builtin pack embedded)",
            env!("CARGO_PKG_VERSION")
        );
        println!("  (no args)     normal launch — intro → title → world");
        println!("  --verbose     the full raw diagnostic stream: [t+s][cat]");
        println!("                lines to the terminal + logs/latest.log —");
        println!("                EVERYTHING raw, even the hidden internals and");
        println!("                the minute errors. Categories: input — every");
        println!("                event: hover changes, menu clicks (widget");
        println!("                id/kind/label + hover/hit cross-check),");
        println!("                unhandled-id WARNs, screen routing; gfx —");
        println!("                boot/surface/gui-scale/canvas geometry, the");
        println!("                per-screen widget-table dump, duplicate-id +");
        println!("                overlap WARNs, 1 Hz UI/render heartbeat; sim");
        println!("                — the 1 Hz internals heartbeat (the sim");
        println!("                ticks, the scheduler depth, the entity counts,");
        println!("                the player's state flags); screen — every");
        println!("                transition; perf, world, save, f3 — the");
        println!("                steady-state stream (run with --verbose when");
        println!("                reporting bugs)");
        println!("  --smoke       CI end-to-end smoke run (boots, enters a world, exits)");
        println!("  --benchmark   [frames=600] [warmup=120] [seed=…] [json=bench.json]");
        return;
    }

    // --verbose: the full raw diagnostic stream (--help above documents
    // it) — enable BEFORE any boot line so the [t+ timestamps anchor at
    // start. THE flag for the raw log (the old --debug name is retired —
    // the same stream, everything raw).
    let debug_on = args.iter().any(|a| a == "--verbose");
    if debug_on {
        vc_render::render::set_verbose(true);
        vc_render::render::report_boot_log(
            "raw debug stream enabled (--verbose) — lines: [t+s][category] message; \
             categories: input screen sim world perf save f3 gfx (gfx = the UI/render pipeline: \
             surface/gui-scale/canvas geometry, widget-table dumps, 1 Hz heartbeat; \
             sim = the 1 Hz internals heartbeat)",
        );
    }

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

    // CI smoke contract (linux-game.yml): boot headless, run the REAL boot
    // path (intro → title panorama), then enter a world through the real
    // pipeline (world-entry loading gate → gameplay) and exit(0) — proves
    // the single-file binary runs end-to-end (pack embed → adapter →
    // pipelines → world streaming → loading gate → gameplay) on a clean
    // machine (lavapipe software Vulkan under Xvfb).
    if args.iter().any(|a| a == "--smoke") {
        app.smoke = true;
    }

    use winit::event_loop::ControlFlow;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| app.handle_event(event, elwt))
        .expect("event loop run");
}
