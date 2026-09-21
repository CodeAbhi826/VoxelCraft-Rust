# Round 18 research verdicts (2026-09-19, all web-verified + cross-checked)

## 1. Vanilla 1.16.5 Options screen (main) — the real layout

Sources: the reference wiki w/Options (live), wayback the reference game.fandom.com/wiki/Options
(2021 snapshot), real-screenshot VLM reads (2 independent), repo history.

```
Row 1: FOV: 70 (slider, L)        | Difficulty: Normal (in world) / Realms Notifications: ON (main menu)
Row 2: Music & Sounds... (L)      | Video Settings... (R)
Row 3: Controls... (L)            | Language... (R)
Row 4: Chat Settings... (L)       | Resource Packs... (R)
Row 5: Skin Customization... (L)  | Accessibility Settings... (R)
        Done (centered bottom)
```

- NO Music/Sound sliders, NO Mouse Sensitivity, NO View Bobbing, NO Engine
  button on the main screen (current VoxelCraft layout has all of these —
  must be restructured).
- Mouse Sensitivity lives in Controls (Mouse tab). View Bobbing lives in
  Video Settings. Music/Sound live in Music & Sounds.
- 1.16.5 difficulty starvation floors (wiki Difficulty + repo round-17):
  Easy 10 HP, Normal 1 HP, Hard never stops. Peaceful: hostile despawn/no
  spawn + passive regen.

## 2. Vanilla 1.16.5 Video Settings — the real layout

Source: real era-correct screenshot (Use VBOs present = pre-1.17; Entity
Shadows present = 1.16+), VLM row-read cross-checked twice:

```
Row 1 (full width):  Fullscreen Resolution: Current
Row 2: Graphics: Fast/Fancy (L)      | Render Distance: N chunks (slider, R)
Row 3: Smooth Lighting: OFF/MIN/MAX  | Max Framerate: N fps (slider)
Row 4: Use VBOs: ON/OFF              | View Bobbing: ON/OFF
Row 5: GUI Scale: Auto/1/2/3         | Attack Indicator: OFF/CROSSHAIR/HOTBAR
Row 6: Brightness (unlabeled slider) | Clouds: Fast/Fancy/Off
Row 7: Full Screen: ON/OFF           | Particles: All/Decreased/Minimal
Row 8: Mipmap Levels: 0-4 (slider)   | Biome Blend: OFF..9x9
Row 9: Entity Shadows: ON/OFF
        Done
```

- Biome Blend in 1.16.5 = cycle OFF/1x1/3x3/5x5/7x7/9x9 (wiki: 5x5 default
  "normal", 3x3 fast in some eras; we keep our verified pad-averaging
  behavior, cycle values per wiki).
- Use VBOs is a no-op under wgpu (always on) — shown for parity, tooltip
  discloses.
- OptiFine 1.16.5 precedent: adds a "Shaders" button to Video Settings
  (bottom, next to Done) → the Shader Packs selection screen. We follow the
  same placement for our SHADERS... button (disclosed as the OptiFine-style
  engine extra) + an ENGINE... button for our non-vanilla knobs.

## 3. OptiFine/Iris shader pack format (BSL/SEUS drop-in)

Sources: OptiFineDoc shaders.txt (821 lines, downloaded), shaders.properties
(526 lines, downloaded), shaders.properties (Iris docs site), shaderLABS wiki.

- Root of pack (zip or folder): `shaders/` folder + `shaders.properties`.
- Programs: `shaders/<program>.vsh/.fsh/.gsh/.csh` where program ∈
  gbuffers_* (terrain, water, entities, skybasic...), deferred, deferred1..99,
  composite, composite1..99, final, prepare*, shadowcomp*.
- Color attachments: colortex0..15 (gcolor, gdepth, gnormal, composite,
  gaux1..4 aliases for 0-7). Ping-pong main/alt buffers; flip per pass.
- Output targets declared via `/* DRAWBUFFERS:XYZ */` or
  `/* RENDERTARGETS: X,Y,Z */` fragment comments (1.17+ style).
- Preprocessor: #define/#undef/#ifdef/#ifndef/#if defined/#elif/#else/#endif
  (OptiFine interprets these BEFORE GLSL compile; options come from
  shaders.properties sliders/toggles → const declarations).
- shaders.properties: profiles.<name>=<option:value>..., screen.<name>=
  <items>, sliders=<ids>, <option>=<default>[:min:step:max], program toggles.
- Uniforms we can honestly bridge (documented subset): viewWidth/Height,
  aspectRatio, frameTimeCounter, frameCounter, worldTime, worldDay, sunAngle,
  rainStrength, isEyeInWater, eyeBrightness, screenBrightness, near/far,
  cameraPosition, sunPosition/moonPosition/shadowLightPosition (eye-space),
  gbufferModelView(±), gbufferProjection(±), fogColor, skyColor, moonPhase.
  NOT bridged in v1: shadow matrices, depthtex pre-shadow, entities ids.

## 4. labPBR 1.3 (NAPP resource packs)

Source: shaderlabs.org LabPBR_Material_Standard (full text captured).

- `_n` normal texture (DirectX Y- / top-down): R = normal X, G = normal Y
  (down-positive), B = material AO (0 = 100% occl, 255 = 0%), A = height
  (0 = 25% depth, 255 = surface; artists use min 1). normal.z reconstructed
  via sqrt(1 - dot(n.xy, n.xy)).
- `_s` specular: R = perceptual smoothness (roughness = (1-s)^2),
  G = 0..229 F0 (linear; 229 = ~90%), 230..254 predefined metals
  (230 Iron, 231 Gold, 232 Aluminum, 233 Chrome, 234 Copper, 235 Lead,
  236 Platinum, 237 Silver), 255 = albedo-as-F0; B = 0..64 porosity,
  65..255 subsurface scattering (65 = 0%, 255 = 100%); A = emissive
  (0..254; 255 = none/ignored).
- Emissive color comes from the albedo texture.
- POM: height from _n alpha; parallax occlusion marching against view ray
  in tangent space.

## 5. Ravines (the "huge ravines everywhere" bug)

Source: the reference wiki World generation + Ravine (repo already cites):
canyons start at levels 10..72, 85..127 long, typically < 15 wide, up to 62
deep (rare max). Frequency unpublished on the wiki (community ≈ 1 per
50-100 chunks regionally, carved-area fraction ≈ 1-2% of the surface).

Current VoxelCraft bug: RAVINE_CHANCE 0.02 + depth ALWAYS 40..62 + no depth
taper at ends → ~10% of the surface carved into full-depth canyons = "huge
ravines in most places". Fix: chance 0.005 (1/200), depth 10..62 with a
rare-max distribution (most 10..30), depth also tapers at the two ends
(vanilla canyon entrances slope), keep shape grammar wiki-cited.

## 6. NAPP / BSL runtime requirements

- BSL (capttatsu): OptiFine/Iris pack, zip with shaders/ + shaders.properties;
  "Advanced Materials Resource packs with shader maps are supported" =
  labPBR. 1.16.5-compatible versions exist (BSL v7.x era).
- NAPP: resource pack with 512x/256x labPBR normals (_n) + specular (_s)
  maps; pack_format 6 (1.16.x) builds exist. Requires a shader pack for the
  maps to be consumed; textures themselves render fine as plain albedo.
