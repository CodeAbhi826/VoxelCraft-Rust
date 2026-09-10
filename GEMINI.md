# VoxelCraft-Rust — Agent Guidelines & Rules

## 1. Hardware & Compilation Guardrails (STRICT)
- **Host CPU/GPU**: Intel Celeron N4000 (dual-core 1.10 GHz, UHD Graphics 600).
- **NEVER run `cargo build --release` or compile release binaries locally on the user's host machine**. Local release compilation risks thermal throttling and host freezing.
- **Local machine usage is strictly restricted to**:
  - `cargo check -p <crate> --lib`
  - `cargo test -p <crate> --lib`
  - Fast single-file or non-release inspection commands
  - Git operations
- **All production executables, release packages, and standalone release binaries must be compiled and distributed via GitHub Actions CI**.

## 2. Clean-Room Legal Compliance
- **Zero Copied Assets**: Do NOT extract, copy, or bundle proprietary Minecraft .jar assets, textures, sounds, or Mojang code.
- **Procedural Art & Textures**: All 16x16 tiles, fonts, and particle textures must be synthesized procedurally at boot or generated cleanly via non-infringing math.
- **Independent Clean-Room Architecture**: Re-implement vanilla mechanics using clean-room specifications and open architectural comparisons with Luanti (Minetest).

## 3. Visual & Rendering Standards
- **Seamless Tiling**: Block faces must tile continuously without artificial border insets, clamping gaps, or "chocolate bar" seams.
- **Explicit Atlas Gradients**: Atlas UV gradients must be explicitly scaled to the 32x32 atlas dimensions (`dpdx(in.uv) / 32.0`) to avoid LOD explosion and coarse mipmap bleeding across tile seams.
- **Minecraft 1.16.5 Fidelity**: Smooth lighting curve, ambient occlusion, water flow/transparency, 15x15 crosshair, and lower-right first-person 3D player arm with walk bobbing and attack swing animation.

## 4. UI & Menu Layout Parity
- **Creative Inventory**: 12 top/bottom category tabs, 3D player preview avatar tracking the cursor, working armor & offhand slots, 9x5 block grid with vertical scrollbar, live search filter box, 9-slot hotbar, and trash slot.
- **Video Settings**: Exact 19-option 1.16.5 layout with 2 top wide sliders, 2-column 16-option grid, and bottom Done button.
- **World Selection**: Darkened dirt background, top search box, scrollable world cards, and bottom action bar.
- **F3 Debug Screen**: Two-column layout with real-time FPS, coordinates, biome, chunk cache, and Targeted Block/Fluid properties.
