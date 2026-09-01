//! Web input bridge (wasm-only).
//!
//! Drains the JS event queue (`window.voxelcraftEvents`, filled by the shim
//! in voxelcraft.html) once per frame and turns it into typed events.
//!
//! Why this exists: winit's web backend delivers `KeyboardInput` only when
//! its canvas holds DOM focus and gates `DeviceEvent::MouseMotion` behind
//! the same focus (default `DeviceEvents::WhenFocused`). Inside nested
//! cross-origin iframes (preview panels) that focus is lost silently and
//! both mouse-look and keyboard die. Document-level JS listeners are
//! focus-independent, so all game input flows through here on wasm.

use wasm_bindgen::JsCast;

/// Events produced by the JS input shim.
#[derive(Debug, Clone)]
pub enum WebEvent {
    Key { code: String, pressed: bool, repeat: bool },
    MouseDelta { dx: f32, dy: f32 },
    Cursor { x: f32, y: f32 },
    Button { button: i32, pressed: bool, x: f32, y: f32 },
    Wheel { dir: f32 },
    LockChange { locked: bool },
    LockError,
    Resize { w: u32, h: u32 },
    Blur,
    Visibility { hidden: bool },
}

/// Drain `window.voxelcraftEvents` (array of small arrays) → typed events.
pub fn drain_events() -> Vec<WebEvent> {
    let mut out = Vec::new();
    let Some(window) = web_sys::window() else { return out };
    let win: wasm_bindgen::JsValue = window.into();
    let Ok(arr) = js_sys::Reflect::get(&win, &"voxelcraftEvents".into()) else { return out };
    let Ok(arr) = arr.dyn_into::<js_sys::Array>() else { return out };
    if arr.length() == 0 {
        return out;
    }
    for i in 0..arr.length() {
        let Some(ev) = arr.get(i).dyn_into::<js_sys::Array>().ok() else { continue };
        let tag = ev.get(0).as_string().unwrap_or_default();
        let f = |i: u32| ev.get(i).as_f64().unwrap_or(0.0);
        let b = |i: u32| ev.get(i).as_f64().unwrap_or(0.0) > 0.5;
        match tag.as_str() {
            "k" => out.push(WebEvent::Key {
                code: ev.get(1).as_string().unwrap_or_default(),
                pressed: b(2),
                repeat: b(3),
            }),
            "m" => out.push(WebEvent::MouseDelta { dx: f(1) as f32, dy: f(2) as f32 }),
            "c" => out.push(WebEvent::Cursor { x: f(1) as f32, y: f(2) as f32 }),
            "b" => out.push(WebEvent::Button {
                button: f(1) as i32,
                pressed: b(2),
                x: f(3) as f32,
                y: f(4) as f32,
            }),
            "w" => out.push(WebEvent::Wheel { dir: f(1) as f32 }),
            "pl" => out.push(WebEvent::LockChange { locked: b(1) }),
            "ple" => out.push(WebEvent::LockError),
            "r" => out.push(WebEvent::Resize { w: f(1) as u32, h: f(2) as u32 }),
            "blur" => out.push(WebEvent::Blur),
            "vis" => out.push(WebEvent::Visibility { hidden: b(1) }),
            _ => {}
        }
    }
    arr.set_length(0);
    out
}

fn window_ref() -> Option<wasm_bindgen::JsValue> {
    web_sys::window().map(|w| w.into())
}

fn call_window_fn(name: &str) {
    if let Some(win) = window_ref() {
        if let Ok(f) = js_sys::Reflect::get(&win, &name.into()) {
            if let Ok(f) = f.dyn_into::<js_sys::Function>() {
                let _ = f.call0(&win);
            }
        }
    }
}

/// Ask the browser for pointer lock (must run inside a user-activation
/// window — button clicks qualify; the shim handles failure → drag-look).
pub fn request_pointer_lock() {
    call_window_fn("voxelcraftRequestLock");
}

/// Release pointer lock (entering pause/menu).
pub fn release_pointer_lock() {
    call_window_fn("voxelcraftRequestUnlock");
}

/// Keep the shim's notion of the current screen in sync (controls which
/// events it synthesizes vs. consumes).
pub fn set_screen(s: &str) {
    if let Some(win) = window_ref() {
        if let Ok(f) = js_sys::Reflect::get(&win, &"voxelcraftSetScreen".into()) {
            if let Ok(f) = f.dyn_into::<js_sys::Function>() {
                let _ = f.call1(&win, &wasm_bindgen::JsValue::from_str(s));
            }
        }
    }
}

/// Persist settings (semicolon-separated key=value pairs) to localStorage.
pub fn save_settings(s: &str) {
    if let Some(win) = window_ref() {
        if let Ok(f) = js_sys::Reflect::get(&win, &"voxelcraftSaveSettings".into()) {
            if let Ok(f) = f.dyn_into::<js_sys::Function>() {
                let _ = f.call1(&win, &wasm_bindgen::JsValue::from_str(s));
            }
        }
    }
}

/// Load persisted settings ("" when absent).
pub fn load_settings() -> Option<String> {
    let win = window_ref()?;
    let f = js_sys::Reflect::get(&win, &"voxelcraftLoadSettings".into()).ok()?;
    let f = f.dyn_into::<js_sys::Function>().ok()?;
    let v = f.call0(&win).ok()?;
    v.as_string()
}

/// Publish a stats object for E2E tests / in-page debugging.
pub fn publish_stats(fields: &[(&str, StatsVal)]) {
    let Some(win) = window_ref() else { return };
    let obj = js_sys::Object::new();
    for (k, v) in fields {
        let val = match v {
            StatsVal::F(x) => wasm_bindgen::JsValue::from_f64(*x as f64),
            StatsVal::S(s) => wasm_bindgen::JsValue::from_str(s),
            StatsVal::B(b) => wasm_bindgen::JsValue::from_bool(*b),
        };
        let _ = js_sys::Reflect::set(obj.as_ref(), &(*k).into(), &val);
    }
    let _ = js_sys::Reflect::set(&win, &"__vcStats".into(), obj.as_ref());
}

pub enum StatsVal {
    F(f32),
    S(String),
    B(bool),
}
