# Colour-space audit (Part 5, 5.1a)

Every sRGB/linear conversion in the renderer. VERIFIED = read in code;
TO-VERIFY = needs a targeted follow-up slice.

| # | Stage | Conversion | Mechanism | Status |
|---|---|---|---|---|
| 1 | Surface present | linear → sRGB on write | sRGB swapchain target (hardware encode) | VERIFIED (render.rs surface config + composite target) |
| 2 | Scene/post intermediates | none (linear end-to-end) | Rgba8Unorm targets; no sRGB sampling in post | VERIFIED (PostTargets + the WebGL2 corruption comment) |
| 3 | Block atlas sample | sRGB → linear on read | Rgba8UnormSrgb atlas texture | VERIFIED (atlas_tex descriptor) |
| 4 | Item icons | sRGB → linear on read | Rgba8UnormSrgb icon textures | VERIFIED (item_icon_cache descriptors) |
| 5 | GUI sheets (hearts/hotbar) | sRGB → linear on read | Rgba8UnormSrgb sheet textures | VERIFIED (gui_render descriptors) |
| 6 | Solid-white tint quad | none (linear value) | Rgba8Unorm 2×2 white texture | VERIFIED (comment + descriptor) |
| 7 | Font glyph atlas | linear upload, linear use | 1024×1024 linear atlas | PARTIAL (comment only — presentation path TO-VERIFY) |
| 8 | Panorama offscreen/dump | sRGB → linear in, linear → sRGB on dump | explicit converts in panorama.rs | VERIFIED (comments + dump fn) |
| 9 | CPU canvas fallback (no-GPU path) | ? | UiCanvas px buffer → upload format unchecked | TO-VERIFY |
| 10 | Sky/fog/cloud colors | authored space used raw? | sky gradient pow() shaping in render.rs | TO-VERIFY (values may be sRGB-authored, linear-consumed) |
| 11 | Biome tints/colormaps | ? | tint LUT + pack overlays | TO-VERIFY (Part 5 tint slices) |
| 12 | Screenshot/PNG readback | linear → sRGB? | readback encode path | TO-VERIFY |
| 13 | Lightmap | n/a (no lightmap texture — lighting is computed per-vertex/fragment) | — | VERIFIED (no lightmap code exists) |
| 14 | Shader-pack handoff | linear preserved | post_pipe_linear variant | VERIFIED |
| 15 | Tonemap/grade/vignette order | operates in linear pre-encode? | composite chain order | TO-VERIFY |

Follow-ups: one slice per TO-VERIFY row (fix or document-as-correct).
