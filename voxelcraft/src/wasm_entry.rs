//! WASM entry point: boots the game onto the #game canvas with WebGPU
//! (falls back to WebGL2 when no WebGPU adapter is available).
//! Served by the browser build (`wasm-bindgen --target web`).

use wasm_bindgen::prelude::*;
use winit::platform::web::EventLoopExtWebSys;
use winit::platform::web::WindowBuilderExtWebSys;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let doc = window.document().ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = doc
        .get_element_by_id("game")
        .ok_or_else(|| JsValue::from_str("missing #game canvas"))?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into()?;
    // focus for keyboard events
    if let Some(c) = canvas.dyn_ref::<web_sys::HtmlElement>() {
        let _ = c.focus();
    }

    wasm_bindgen_futures::spawn_local(async move {
        let event_loop = winit::event_loop::EventLoop::new().expect("event loop");
        let window = winit::window::WindowBuilder::new()
            .with_canvas(Some(canvas))
            .build(&event_loop)
            .expect("window");
        // page-lifetime reference (wgpu surface needs 'static)
        let window: &'static winit::window::Window = Box::leak(Box::new(window));

        let mut app = crate::game::GameApp::new(window).await;

        use winit::event_loop::ControlFlow;
        event_loop.set_control_flow(ControlFlow::Poll);
        // NOTE: `run()` throws a JS exception on wasm to satisfy its `!` return
        // type; unwinding through this async task would drop the event loop.
        // `spawn()` registers the handler and returns normally.
        event_loop.spawn(move |event, elwt| app.handle_event(event, elwt));
    });

    Ok(())
}
