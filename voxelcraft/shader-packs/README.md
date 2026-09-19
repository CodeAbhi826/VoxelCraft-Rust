# Shader Packs

Drop **Iris-format** shader packs here as plain folders (a
`shaders.properties` file plus a `shaders/` program directory). The
engine scans this folder whenever you open **Video Settings →
SHADERS...** — no restart needed.

## What the engine does with a pack

The scan runs the Iris **structure analysis** (the documented, published
interface — no GLSL is executed):

* parse `shaders.properties` (profiles, screens, sliders, directives)
* walk the `shaders/` program layout and map every stage to its phase
  (gbuffers → composite → final)
* read the `/* RENDERTARGETS: n */` / `DRAWBUFFERS` directives
* report the documented uniforms the pack expects

Every pack row on the Shaders screen carries its honest tier label
(`IRIS-STRUCTURE-VALIDATED`) — meaning the pack was detected and fully
parsed, **not** that it renders yet. The GLSL → WGSL translation lives
in the separate vc-iris sister project; until it registers, selecting
a pack stores your choice (persisted in options.txt as `shaderpack`)
and reports the analysis, while the render pipeline stays vanilla.

## What may legally live here

This folder is for **packs you legally obtained** — BSL, SEUS,
complementary-style packs downloaded from their own distribution, or
anything you wrote yourself. The engine never ships, hosts, or
redistributes any third-party shader pack (see the repo's
`docs/LEGAL-COMPLIANCE.md`). What you drop here stays on your machine.

## Also: labPBR materials

The Shaders screen has a **LABPBR MATERIALS** toggle. When ON, resource
packs that provide labPBR 1.3 companion maps (`stone_n.png` normal maps
and `stone_s.png` specular maps next to the albedo texture, the
NAPP-style convention) are detected and decoded, and the count is
reported. OFF (the default) keeps the pure vanilla look.

Example layout:

```
shader-packs/
  BSL-v8.2/               <- the pack folder (Iris format)
    shaders.properties
    shaders/
      gbuffers_terrain.vsh / .fsh
      composite.fsh
      final.fsh
```

Selecting **(none)** on the screen returns to the vanilla pipeline —
the default, and always available.
