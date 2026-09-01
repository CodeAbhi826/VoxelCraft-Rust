# VoxelCraft-Rust — Deep Bug Audit & Fix Brief

> **Repo audited:** https://github.com/CodeAbhi826/VoxelCraft-Rust (commit d18de16)
> **Method:** Every file read line-by-line. Every code path traced. Every wgpu/egui API call verified against the actual 0.20/0.28 docs in the cargo registry.
>
> **Result:** 7 bugs found. 3 are CRITICAL (game crashes or can't progress). 4 are HIGH (visible glitches or missing features the engine was supposed to have).

---

## BUG SUMMARY TABLE

| # | Severity | Title | File | Line |
|---|----------|-------|------|------|
| 1 | 🔴 CRITICAL | Missing texture bind group → crash on first chunk draw | `renderer/mod.rs` | 199 |
| 2 | 🔴 CRITICAL | Loading screen never builds meshes → stuck at 0% for 8s | `main.rs` | 192-202 |
| 3 | 🔴 CRITICAL | Loading/Settings/Logger states try to render 3D world with identity matrix → triggers Bug #1 crash | `main.rs` | 234-242 |
| 4 | 🟠 HIGH | Parallel meshing (`build_meshes_parallel`) NEVER called — game uses slow sync path | `main.rs` | 274 |
| 5 | 🟠 HIGH | `process_mesh_uploads` drains ALL pending in one frame → stutter spikes | `main.rs` | 260 |
| 6 | 🟠 HIGH | Place block doesn't check player intersection → player gets stuck inside placed block | `main.rs` | 144-150 |
| 7 | 🟡 MEDIUM | StartScreen doesn't build meshes → terrain invisible when menu appears | `main.rs` | 203-206 |

---

## 🔴 CRITICAL BUG #1 — Missing texture bind group (THE crash you saw)

### Location
**File:** `src/renderer/mod.rs`
**Line:** 199

### The code (broken)
```rust
render_pass.set_pipeline(&self.world_pipeline.pipeline);
render_pass.set_bind_group(0, &self.world_pipeline.bind_group, &[]);  // ← uniforms only
// ❌ bind group 1 (texture atlas) is NEVER set
```

### Why it's broken
The pipeline layout (in `pipeline.rs` line 58) declares TWO bind group layouts:
```rust
bind_group_layouts: &[&bind_group_layout, texture_layout],  // [0=uniforms, 1=texture]
```

The WGSL shader (`world.wgsl` lines 11-13) expects both:
```wgsl
@group(0) @binding(0) var<uniform> u: WorldUniforms;
@group(1) @binding(0) var tex_array: texture_2d_array<f32>;
@group(1) @binding(1) var tex_sampler: sampler;
```

But the render pass only binds group 0 (uniforms). When `draw_indexed` is called, wgpu validates that all bind groups referenced by the pipeline layout are bound. Group 1 is missing → validation error → crash:
```
Incompatible bind group at index 1 in the current render pipeline
note: Should be compatible with an explicit bind group layout with label = `Texture Bind Layout`
```

### The fix
In `src/renderer/mod.rs`, find line 199 and add ONE line after it:

**Before:**
```rust
            render_pass.set_pipeline(&self.world_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.world_pipeline.bind_group, &[]);
```

**After:**
```rust
            render_pass.set_pipeline(&self.world_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.world_pipeline.bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_atlas.bind_group, &[]);  // ← ADD THIS LINE
```

### Why this works
`self.texture_atlas` is a field of `Renderer` (line 26) and has a `bind_group` field (created in `texture.rs` line 103, labeled "Texture Bind Group"). Its layout matches `texture_layout` passed to the pipeline. Binding it to slot 1 satisfies the pipeline's requirement.

---

## 🔴 CRITICAL BUG #2 — Loading screen never builds meshes (stuck at 0%)

### Location
**File:** `src/main.rs`
**Lines:** 192-202 (the `GameState::Loading` block)

### The code (broken)
```rust
GameState::Loading => {
    if !loading_started { world.update_player_position(0, 0, settings.render_distance); loading_started = true; }
    let (total, ready) = world.chunk_stats();
    let progress = if loading_total > 0 { ready as f32 / loading_total as f32 } else { 0.0 };
    if progress > 0.6 || loading_start.elapsed().as_secs() > 8 {
        state = GameState::StartScreen;
        // ...
    }
    render_loading_screen(&egui_ctx, progress, ready, loading_total);
}
```

### Why it's broken
1. `update_player_position` calls `generate_chunks_parallel`, which inserts chunks with `dirty = true` (the `Chunk::new` constructor sets `dirty = true`, and `WorldGenerator::generate` also sets `dirty = true`).
2. `chunk_stats()` returns `ready = count of chunks where dirty == false`.
3. Since `dirty` is never cleared during Loading (no `build_meshes_parallel` call), `ready` stays 0 forever.
4. Progress = 0 / total = 0%. The 0.6 threshold is never reached.
5. The ONLY escape is the 8-second timeout. After 8 seconds, the game proceeds to StartScreen — but NO meshes have been built, so the terrain is invisible.

### The fix
In `src/main.rs`, in the `GameState::Loading` block, add a call to `build_meshes_parallel` BEFORE checking progress. Replace the entire Loading block (lines 192-202) with:

```rust
GameState::Loading => {
    if !loading_started {
        world.update_player_position(0, 0, settings.render_distance);
        loading_started = true;
    }
    // Build meshes for generated chunks (parallel, uses all CPU cores)
    world.build_meshes_parallel(0, 0, 4);  // mesh up to 4 chunks per frame
    // Upload finished meshes to GPU
    process_mesh_uploads(&world, &mut renderer);

    let (total, ready) = world.chunk_stats();
    let progress = if loading_total > 0 { ready as f32 / loading_total as f32 } else { 0.0 };
    if progress > 0.6 || loading_start.elapsed().as_secs() > 8 {
        state = GameState::StartScreen;
        let spawn_y = Player::find_spawn(&world, 0, 0);
        player.position = Vec3::new(0.5, spawn_y, 0.5);
    }
    render_loading_screen(&egui_ctx, progress, ready, loading_total);
}
```

### Why this works
`build_meshes_parallel` meshes dirty chunks in parallel (using rayon) and queues the results. It also clears the `dirty` flag on meshed chunks. Now `ready` increments as chunks get meshed, progress rises from 0% to 60%, and the game transitions to StartScreen with terrain already visible.

---

## 🔴 CRITICAL BUG #3 — UI-only states try to render 3D world (triggers Bug #1 crash)

### Location
**File:** `src/main.rs`
**Lines:** 234-242

### The code (broken)
```rust
let (view, proj) = if matches!(state, GameState::Playing | GameState::Paused | GameState::StartScreen) {
    let eye = player.eye_position();
    let view = Mat4::look_to_rh(eye, player.forward_vector(), Vec3::Y);
    let proj = Mat4::perspective_rh_gl(...);
    (view, proj)
} else { (Mat4::IDENTITY, Mat4::IDENTITY) };

let fog_color = sky_color_for_time(0.3);
let _ = renderer.render(view, proj, player.eye_position(), fog_color, 0.3, &mut egui_renderer, primitives, &screen_descriptor);
```

### Why it's broken
For `Loading`, `Settings`, and `Logger` states, `view` and `proj` are `Mat4::IDENTITY`. But `renderer.render()` is still called — it runs the world render pass, iterates `chunk_meshes`, and calls `draw_indexed` with the identity MVP. This draws chunks at world origin with a broken camera, which triggers the bind group validation (Bug #1) and crashes.

Even AFTER Bug #1 is fixed, rendering chunks with an identity matrix produces garbage visuals (everything at clip space origin).

### The fix
Add a `render_ui_only` method to `Renderer` that skips the world pass entirely. Then guard the render call in main.rs.

**Step 1:** In `src/renderer/mod.rs`, add this new method (after the existing `render` method, before the `compute_frustum_planes` function):

```rust
/// Render ONLY the egui UI (no 3D world). Used for Loading/Settings/Logger states.
pub fn render_ui_only(
    &mut self,
    egui_renderer: &mut Option<egui_wgpu::Renderer>,
    egui_primitives: Vec<egui::ClippedPrimitive>,
    screen_descriptor: &egui_wgpu::ScreenDescriptor,
    clear_color: [f32; 4],
) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view_tex = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("UI Only Encoder"),
    });

    // egui buffer upload
    let mut egui_cmds: Vec<wgpu::CommandBuffer> = Vec::new();
    if let Some(renderer) = egui_renderer.as_mut() {
        egui_cmds = renderer.update_buffers(
            &self.device, &self.queue, &mut encoder,
            &egui_primitives, screen_descriptor,
        );
    }

    // Clear screen + draw egui (no world pass)
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view_tex,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        if let Some(renderer) = egui_renderer.as_ref() {
            renderer.render(&mut render_pass, &egui_primitives, screen_descriptor);
        }
    }

    let main_cmd = encoder.finish();
    egui_cmds.push(main_cmd);
    self.queue.submit(egui_cmds);
    output.present();
    Ok(())
}
```

**Step 2:** In `src/main.rs`, replace lines 234-242 with:

```rust
let fog_color = sky_color_for_time(0.3);
// Only render the 3D world for states that need it.
// Loading/Settings/Logger are pure UI — skip world rendering.
if matches!(state, GameState::Playing | GameState::Paused | GameState::StartScreen) {
    let eye = player.eye_position();
    let view = Mat4::look_to_rh(eye, player.forward_vector(), Vec3::Y);
    let proj = Mat4::perspective_rh_gl(
        settings.fov.to_radians(),
        renderer.config.width as f32 / renderer.config.height as f32,
        0.05, 1000.0,
    );
    let _ = renderer.render(view, proj, player.eye_position(), fog_color, 0.3, &mut egui_renderer, primitives, &screen_descriptor);
} else {
    let _ = renderer.render_ui_only(&mut egui_renderer, primitives, &screen_descriptor, fog_color);
}
```

### Why this works
UI-only states (Loading, Settings, Logger) now skip the world render pass entirely. No chunk drawing = no bind group validation = no crash. The screen is cleared to the sky color and egui is drawn on top.

---

## 🟠 HIGH BUG #4 — Parallel meshing never used (game is single-threaded)

### Location
**File:** `src/main.rs`
**Line:** 274 (inside `rebuild_dirty_chunks`)

### The code (broken)
```rust
fn rebuild_dirty_chunks(world: &World, player: &Player, budget: usize) {
    // ...
    if let Some(chunk) = world.get_chunk(cx, cz) {
        if chunk.read().dirty {
            if let Some(mesh) = world.build_mesh(cx, cz) {  // ← synchronous, one chunk at a time
                world.queue_mesh(cx, cz, mesh);
                // ...
            }
        }
    }
}
```

### Why it's broken
The `World` struct has a `build_meshes_parallel` method (world.rs line 135) that uses `rayon::par_iter()` to mesh multiple chunks across all CPU cores. But `rebuild_dirty_chunks` in main.rs NEVER calls it — it uses `world.build_mesh(cx, cz)` (synchronous, single chunk) in a loop. This means:
- Only 1 CPU core is used for meshing
- Chunk loading is slow → stutter when flying fast
- The `rayon` dependency is wasted

### The fix
Replace the entire `rebuild_dirty_chunks` function (lines 262-285) with a call to `build_meshes_parallel`:

```rust
fn rebuild_dirty_chunks(world: &World, player: &Player, budget: usize) {
    // Use parallel meshing — meshes multiple chunks across all CPU cores.
    world.build_meshes_parallel(
        player.position.x as i32,
        player.position.z as i32,
        budget,
    );
}
```

Then in the `GameState::Playing` block, ensure `process_mesh_uploads` is called after (it already is on line 210). The flow becomes:
1. `build_meshes_parallel` meshes dirty chunks in parallel, queues results
2. `process_mesh_uploads` drains the queue and uploads to GPU

### Why this works
`build_meshes_parallel` already does everything `rebuild_dirty_chunks` did (find dirty chunks, sort by distance, build meshes, clear dirty flag) but in parallel across all CPU cores. The function signature matches: it takes player X/Z (for distance sorting) and a budget.

---

## 🟠 HIGH BUG #5 — Upload budget missing (stutter when many chunks finish at once)

### Location
**File:** `src/main.rs`
**Line:** 260

### The code (broken)
```rust
fn process_mesh_uploads(world: &World, renderer: &mut Renderer) {
    for p in world.drain_pending_meshes() { renderer.upload_chunk_mesh(p.cx, p.cz, p.mesh); }
}
```

### Why it's broken
`drain_pending_meshes()` returns ALL pending meshes. If 50 chunks finished meshing in the same frame (e.g., after flying to a new area), all 50 upload in one frame → GPU stalls → frame time spikes from 16ms to 200ms → visible stutter.

### The fix
Add a per-frame upload budget. Replace `process_mesh_uploads` (lines 259-261) with:

```rust
fn process_mesh_uploads(world: &World, renderer: &mut Renderer) {
    // Limit uploads per frame to avoid stutter.
    // Each chunk upload is ~1-2ms; 4 uploads = ~4-8ms = stays under 16ms frame budget.
    const MAX_UPLOADS_PER_FRAME: usize = 4;
    let pending = world.drain_pending_meshes();
    for p in pending.into_iter().take(MAX_UPLOADS_PER_FRAME) {
        renderer.upload_chunk_mesh(p.cx, p.cz, p.mesh);
    }
}
```

Note: `drain_pending_meshes` takes ownership of the entire Vec. The remaining (non-uploaded) meshes are lost. To fix this properly, change `drain_pending_meshes` to not drain, or re-queue the leftovers. The simplest fix is to change the World method to peek instead of drain:

**In `src/world/world.rs`**, add a new method:

```rust
/// Take up to `max` pending meshes, leaving the rest for next frame.
pub fn take_pending_meshes(&self, max: usize) -> Vec<PendingMesh> {
    let mut pending = self.pending_meshes.lock();
    let take = max.min(pending.len());
    pending.drain(..take).collect()
}
```

Then in main.rs:
```rust
fn process_mesh_uploads(world: &World, renderer: &mut Renderer) {
    const MAX_UPLOADS_PER_FRAME: usize = 4;
    for p in world.take_pending_meshes(MAX_UPLOADS_PER_FRAME) {
        renderer.upload_chunk_mesh(p.cx, p.cz, p.mesh);
    }
}
```

### Why this works
Only 4 chunks upload per frame (max ~8ms). The rest stay in the queue for next frame. Frame time stays smooth at 16ms. Chunks still load — just 4 at a time instead of 50.

---

## 🟠 HIGH BUG #6 — Place block doesn't check player intersection

### Location
**File:** `src/main.rs`
**Lines:** 144-150

### The code (broken)
```rust
MouseButton::Right if pressed && state == GameState::Playing => {
    if let Some((pos, normal)) = player.raycast(&world, 6.0) {
        let place = pos + normal;
        let block = hotbar[selected_slot];
        world.set_block(place.x, place.y, place.z, block);  // ← no intersection check
    }
}
```

### Why it's broken
If the player looks down at their feet and right-clicks, the block is placed inside the player's AABB. The player gets stuck inside the block and can't move. This is a classic voxel game bug.

### The fix
Add an AABB intersection check before placing. Replace lines 144-150 with:

```rust
MouseButton::Right if pressed && state == GameState::Playing => {
    if let Some((pos, normal)) = player.raycast(&world, 6.0) {
        let place = pos + normal;
        let block = hotbar[selected_slot];
        // Don't place inside the player's AABB
        let p_min = player.position - Vec3::new(0.3, 0.0, 0.3);
        let p_max = player.position + Vec3::new(0.3, 1.8, 0.3);
        let b_min = Vec3::new(place.x as f32, place.y as f32, place.z as f32);
        let b_max = b_min + Vec3::ONE;
        let intersects = p_min.x < b_max.x && p_max.x > b_min.x
                       && p_min.y < b_max.y && p_max.y > b_min.y
                       && p_min.z < b_max.z && p_max.z > b_min.z;
        if !intersects {
            world.set_block(place.x, place.y, place.z, block);
        }
    }
}
```

### Why this works
The player AABB is `[-0.3, 0, -0.3]` to `[+0.3, 1.8, +0.3]` relative to position. The block AABB is `[x, y, z]` to `[x+1, y+1, z+1]`. If they overlap on all 3 axes, the placement is rejected. The player can still place blocks adjacent to themselves — just not inside themselves.

---

## 🟡 MEDIUM BUG #7 — StartScreen doesn't build meshes (terrain invisible)

### Location
**File:** `src/main.rs`
**Lines:** 203-206

### The code (broken)
```rust
GameState::StartScreen => {
    if render_start_screen(&egui_ctx, true) { state = GameState::Playing; lock_mouse(&window, &mut mouse_locked); }
    process_mesh_uploads(&world, &mut renderer);
}
```

### Why it's broken
StartScreen calls `process_mesh_uploads` (uploads pending meshes) but NEVER calls `rebuild_dirty_chunks` or `build_meshes_parallel`. If the Loading state finished before all chunks were meshed (e.g., the 8-second timeout fired), the remaining dirty chunks never get meshed on StartScreen. The terrain stays invisible.

### The fix
Add `rebuild_dirty_chunks` to the StartScreen block. Replace lines 203-206 with:

```rust
GameState::StartScreen => {
    if render_start_screen(&egui_ctx, true) { state = GameState::Playing; lock_mouse(&window, &mut mouse_locked); }
    rebuild_dirty_chunks(&world, &player, 2);  // ← ADD: keep meshing during menu
    process_mesh_uploads(&world, &mut renderer);
}
```

### Why this works
While the player is on the Start Screen, the game continues meshing dirty chunks (2 per frame) and uploading them. By the time the player clicks PLAY, all nearby chunks have meshes and the terrain is visible immediately.

---

## IMPLEMENTATION ORDER

Apply fixes in THIS order. After each, rebuild and test.

1. **Bug #1** (bind group) — fixes the crash. 1 line.
2. **Bug #3** (render_ui_only) — fixes crash on Loading screen. Add method + guard.
3. **Bug #2** (loading builds meshes) — fixes stuck-at-0% loading. Add 2 lines.
4. **Bug #7** (StartScreen builds meshes) — fixes invisible terrain on menu. 1 line.
5. **Bug #4** (parallel meshing) — fixes slow chunk loading. Replace function.
6. **Bug #5** (upload budget) — fixes stutter. Add method + replace function.
7. **Bug #6** (place block intersection) — fixes getting stuck. Add check.

---

## VERIFICATION TESTS

After applying ALL fixes, run these checks:

### Test 1 — Compiles
```bash
cargo build --release
```
**PASS:** 0 errors, 0 warnings (or only unused-variable warnings).

### Test 2 — Engine tests still pass
```bash
cargo test
```
**PASS:** 30 tests pass (22 unit + 8 integration). If any fail, you broke the engine — revert.

### Test 3 — Loading screen progresses
```bash
cargo run --release
```
**PASS:**
- Loading screen appears with "VOXELCRAFT" + "GENERATING WORLD..."
- Progress bar fills from 0% to 60% over 2-5 seconds
- Chunk counter increments ("45 / 169 chunks (27%)")
- After 60%, transitions to Start Screen
- **FAIL:** Stuck at 0% for 8 seconds, then jumps to Start Screen

### Test 4 — Start Screen shows terrain
**PASS:**
- Start Screen appears with "VOXELCRAFT" title + "PLAY" button
- Terrain (grass, trees, water) is visible in the background
- **FAIL:** Only blue sky + text, no terrain

### Test 5 — Click PLAY works
**PASS:**
- Click PLAY → mouse locks, cursor disappears
- Player is in the world, can look around
- Terrain renders correctly (grass green, water blue, trees)
- **FAIL:** Crash, or no terrain, or mouse doesn't lock

### Test 6 — No crash on Settings/Logger
**PASS:**
- Press ESC → Pause menu appears
- Click Settings → settings panel appears, no crash
- Press ESC → back to Start Screen
- Press L → logger panel appears, no crash
- Press ESC → back to Start Screen
- **FAIL:** Crash when opening Settings or Logger

### Test 7 — Place block doesn't trap player
**PASS:**
- Look down at feet, right-click → block is NOT placed (intersection rejected)
- Look at adjacent block, right-click → block IS placed
- **FAIL:** Player gets stuck after placing block at feet

### Test 8 — No stutter when flying fast
**PASS:**
- Press F (fly mode), fly forward fast for 10 seconds
- Frame rate stays smooth (no obvious stutter)
- Chunks load in ahead of you without freezing
- **FAIL:** Game freezes for 0.5-2 seconds when crossing chunk borders

---

## ROOT CAUSE ANALYSIS

The 3 critical bugs form a chain:
1. **Bug #2** (loading doesn't mesh) means chunks stay dirty → progress stuck at 0%
2. The 8-second timeout escapes Loading → enters StartScreen with no meshes
3. **Bug #3** (UI states render 3D) tries to draw chunks during Loading/Settings/Logger
4. **Bug #1** (missing bind group) crashes when ANY chunk is drawn

So the crash you saw was: Loading state tries to render chunks (Bug #3) → no texture bound (Bug #1) → crash.

Fixing only Bug #1 would stop the crash, but the game would still be broken (stuck at 0% loading, no terrain). All 3 critical bugs must be fixed for a working game.

---

**End of audit.** Apply all 7 fixes in order, run the 8 verification tests, commit after each pass.
