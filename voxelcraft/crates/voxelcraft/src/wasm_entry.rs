//! WASM entry point: boots the game onto the #game canvas with WebGPU
//! (falls back to WebGL2 when no WebGPU adapter is available).
//! Served by the browser build (`wasm-bindgen --target web`).

use std::sync::atomic::{AtomicBool, Ordering};

use wasm_bindgen::prelude::*;
use winit::platform::web::EventLoopExtWebSys;
use winit::platform::web::WindowBuilderExtWebSys;

static READY_NOTIFIED: AtomicBool = AtomicBool::new(false);

/// Report a fatal error to the on-page boot overlay (via JS interop) so a
/// failed init shows a readable message instead of a blank screen.
pub fn boot_error(msg: &str) {
    if let Some(window) = web_sys::window() {
        let w: JsValue = window.into();
        if let Ok(f) = js_sys::Reflect::get(&w, &"voxelcraftBootError".into()) {
            if let Ok(f) = f.dyn_into::<js_sys::Function>() {
                let _ = f.call1(&w, &JsValue::from_str(msg));
            }
        }
    }
}

/// Log to the JS console (goes through JS interop — no extra web-sys features).
#[allow(dead_code)]
pub fn boot_log(msg: &str) {
    if let Some(window) = web_sys::window() {
        let w: JsValue = window.into();
        if let Ok(f) = js_sys::Reflect::get(&w, &"voxelcraftLog".into()) {
            if let Ok(f) = f.dyn_into::<js_sys::Function>() {
                let _ = f.call1(&w, &JsValue::from_str(msg));
            }
        }
    }
}

/// Tell the page the engine drew its first frame (hides the loading overlay).
fn notify_ready() {
    if READY_NOTIFIED.swap(true, Ordering::Relaxed) {
        return;
    }
    if let Some(window) = web_sys::window() {
        let w: JsValue = window.into();
        if let Ok(f) = js_sys::Reflect::get(&w, &"voxelcraftReady".into()) {
            if let Ok(f) = f.dyn_into::<js_sys::Function>() {
                let _ = f.call0(&w);
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Panic → console AND the visible boot overlay (no more silent blanks).
    std::panic::set_hook(Box::new(|info| {
        console_error_panic_hook::hook(info);
        boot_error(&info.to_string());
    }));

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let doc = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = doc
        .get_element_by_id("game")
        .ok_or_else(|| JsValue::from_str("missing #game canvas"))?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into()?;
    // focus for keyboard events
    if let Some(c) = canvas.dyn_ref::<web_sys::HtmlElement>() {
        let _ = c.focus();
    }

    wasm_bindgen_futures::spawn_local(async move {
        let event_loop = match winit::event_loop::EventLoop::new() {
            Ok(el) => el,
            Err(e) => {
                boot_error(&format!("failed to create event loop: {e:?}"));
                return;
            }
        };
        let window = match winit::window::WindowBuilder::new()
            .with_canvas(Some(canvas))
            .build(&event_loop)
        {
            Ok(w) => w,
            Err(e) => {
                boot_error(&format!("failed to create window: {e:?}"));
                return;
            }
        };
        // page-lifetime reference (wgpu surface needs 'static)
        let window: &'static winit::window::Window = Box::leak(Box::new(window));

        let mut app = crate::game::GameApp::new(window).await;
        boot_log("game app initialized — spawning event loop");

        use winit::event_loop::ControlFlow;
        event_loop.set_control_flow(ControlFlow::Poll);
        // NOTE: `run()` throws a JS exception on wasm to satisfy its `!` return
        // type; unwinding through this async task would drop the event loop.
        // `spawn()` registers the handler and returns normally.
        event_loop.spawn(move |event, elwt| {
            // after the first completed frame, let the page hide the loader
            if matches!(
                event,
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::RedrawRequested,
                    ..
                }
            ) {
                app.handle_event(event, elwt);
                notify_ready();
                return;
            }
            app.handle_event(event, elwt);
        });
    });

    Ok(())
}
