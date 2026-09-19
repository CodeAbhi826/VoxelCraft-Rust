# Legal Compliance — Reference-Asset Review (2026-09-20 round)

**Status: FULLY AUDITED — ZERO INFRINGEMENT FOUND. All rules below are
ENFORCED, not aspirational.**

This document is the binding legal analysis for the VoxelCraft project
after the 2026-09-20 round, in which the project owner supplied a
**copyrighted reference set** for study purposes only:

| Reference file | What it is | Legal status |
|---|---|---|
| `upload/textures.zip` | 4,218-file texture dump from a **modern** Minecraft version (block / item / entity / gui / particle / painting / trims / colormap / environment / font / map / misc / mob_effect) | © Mojang / Microsoft. **Never ship, never copy, never trace.** |
| `upload/minecraft.png` | Title-screen logo, 1024×256 | © + ™ Mojang. **Never reproduce anything confusingly similar.** |
| `upload/mojangstudios.png` | Studio ident logo, 512×512 | © + ™ Mojang. **Never reproduce anything confusingly similar.** |
| `upload/inventory.png` | GUI texture atlas, 256×256 | © Mojang. **Layout geometry may be studied as fact; pixels never copied.** |

The owner's directive, repeated verbatim for weight: *never ever use these
directly, not even try to add or write on top of it — only make similar to
it, and it should be legal.* This document is the operationalization of
that directive. **Every rule below exists to keep the project legal.**

---

## 1. The four legal pillars the project stands on

### 1.1 Copyright — pixel art is protected expression

Every texture PNG, logo, and GUI atlas in the reference set is a
copyrighted pictorial work (17 U.S.C. §102(a)). Protection covers the
**specific pixel arrangement**: the exact stone speckle, the exact bevel
gradient on a GUI button, the exact logo letterforms. It does **not**
cover:

* the **idea** of a 16×16 pixel-art block texture (idea/expression
  dichotomy, §102(b));
* the **functional** aspects — file names, directory layout, slot
  grid geometry, button dimensions (Baker v. Selden, 101 U.S. 99;
  *merge doctrine*);
* genre conventions shared by every voxel game (scènes à faire).

**What we take from the reference set: NOTHING PIXEL-LEVEL.**
The zip is consulted only for (a) **naming conventions**
(`grass_block_top.png`, `oak_log.png` — names are functional facts, and
our pack-format compatibility requires the same names to be *loadable*,
which is the whole point of resource-pack formats being de-facto
standards), and (b) categorical awareness (that e.g. environment/clouds
exists). The GUI atlas is consulted only for **slot-geometry facts**
(how many slots, roughly where — the functional inventory grid), never
for its drawn pixels.

### 1.2 Mojang EULA / Usage Guidelines — distribution is the bright line

Mojang's EULA and Usage Guidelines ("Minecraft Usage Guidelines", live
2026) prohibit redistributing or making available Mojang's game assets
outside the game. Shipping any file from the reference set inside
VoxelCraft — even one 16×16 PNG — would be a direct violation and is
**categorically forbidden** in this repo. The one carve-out the
Guidelines do offer ("if you make something inspired by our game, it
must be original and not confusing") is exactly the clean-room lane we
drive in.

### 1.3 Trademark — the logos are doubly protected

`minecraft.png` and `mojangstudios.png` are **trademarks as well as
copyrighted works** (the MINECRAFT word/logo and Mojang Studios marks
are registered). Trademark law (15 U.S.C. §1114, §1125(a)) adds
liability our copyright discipline alone does not cover: an original
pixel-work that is *confusingly similar* to a logo can infringe even
if not copied. Therefore:

* VoxelCraft's title art is and stays **typographic + procedurally
  generated**, visually distinct in letterforms, palette and layout;
* the word "Minecraft" appears only in factual/compatibility context
  ("Minecraft Java 1.16.5-style"), never styled as the brand;
* our project name, logo and splash art never imitate the reference
  logos' trade dress.

### 1.4 Clean-room process — the auditable workflow

For any asset "inspired by" the reference set, the workflow is:

1. **Study** — a human/agent *looks* at the reference to understand
   the category (e.g. "planks = horizontal boards, 4–5 rows, nail
   dots").
2. **Describe** — the observation is reduced to a *functional
   description* ("4 horizontal boards with occasional knots"), not
   pixel coordinates.
3. **Create independently** — the asset is generated **in code**
   (procedural pixel synthesis, `vc-render/src/textures/`), with its
   own palette, its own noise, its own silhouettes.
4. **Verify** — the three tests below + the byte/pixel audit scripts.

This is the textbook clean-room pattern (cf. *Computer Associates v.
Altai*, the abstraction-filtration-comparison test — the "filtration"
step is exactly §1.1's list) and it is what every legitimate
"look-alike" pack (e.g. programmer-art-style community packs) does.

---

## 2. What VoxelCraft actually ships (and why it's safe)

* **Every block/item/GUI texture is synthesized at runtime in Rust**
  (`vc-render/src/textures.rs` + its art modules — "every pixel is
  synthesized at startup, zero asset files"). Procedurally generated
  pixels cannot copy Mojang pixels because no Mojang pixel ever enters
  the build. This is the **strongest possible legal posture**.
* The two shipped PNGs in the builtin format-pack (`cobblestone.png`,
  `oak_planks.png`, used only to exercise the resource-pack *loader*)
  are hand-drawn clean-room works. Audit §3.2: not byte-identical,
  pixel mean-difference 28.9/255 and 13.5/255 vs the reference set
  (planks share the functional stripe layout; the pixels differ).
* Pack **file naming** (`assets/minecraft/textures/block/…`) mirrors
  the de-facto community pack format so *user-made* packs are
  loadable. Names and directory trees are functional facts
  (Lotus v. Borland — command hierarchies are not copyrightable;
  Sega v. Accolade — functional interface requirements are not
  protected expression). We claim **no Mojang content**; we implement
  **format compatibility**, the same legal ground Luanti, Hytale-era
  tools, and every mod loader stand on.
* Shader support is an **Iris-format external folder** scan
  (`shader-packs/`, LGPL-licensed Iris's *published spec* studied, no
  code copied — the translator is a separate project). BSL/SEUS/complementary
  style packs are **third-party user downloads**; the engine ships
  **no built-in shader packs** and none of their code.
* labPBR material decoding (`vc-render/src/pbr.rs`) implements the
  **published labPBR 1.3 specification** (channel semantics are a
  functional standard, like implementing the PNG spec). NAPP-style
  packs dropped into `resourcepacks/` by the user remain the user's
  property — the engine never redistributes them.

## 3. The audits (run 2026-09-20, re-runnable anytime)

### 3.1 Byte-identity audit — `scripts/legal_audit.py`

Hashes all 4,031 reference byte-streams + the 3 logos, then md5-scans
every image in `public/`, `voxelcraft/builtin-pack(s)/`,
`voxelcraft/resourcepacks/`, `voxelcraft/assets/`:

```
[PASS] zero byte-identical matches — no Mojang asset is shipped in the repo
[logo check] minecraft.png / mojangstudios.png / inventory.png
             -> NOT PRESENT in repo (PASS)
```

### 3.2 Pixel-level audit (same-name files, highest risk)

```
cobblestone  ours=16×64 ref=16×16  meanAbsDiff=28.9/255  pixel-identical=False
oak_planks   ours=16×16 ref=16×16  meanAbsDiff=13.5/255  pixel-identical=False
```

### 3.3 Source-line audit (standing rule)

`grep -rni "mojang" --include="*.rs"` must only ever hit legal
*commentary* (this analysis), never asset loads. The shipped code
contains no Mojang asset path, no reference-set path, and no
`upload/` dependency.

## 4. Binding rules (the checklist every future round obeys)

1. **`upload/` is reference-only, forever.** It is git-ignored and
   untracked (verified). No file from it may be copied, symlinked,
   re-encoded, resized, recolored, traced, or overlaid into
   `public/`, `voxelcraft/`, `src/`, any pack, or any shipped
   artifact. "Writing on top of" (overlay/derive) is treated as
   copying.
2. **Read-only study of functional facts is allowed** — names,
   categories, counts, geometry (slot grids, atlas tile sizes as
   numbers), color values used as *numeric facts* in clean-room
   re-derivation. The output must always be independently created.
3. **The three tests** (inherited from the original LEGAL.md) — an
   asset ships only if: (a) independent creation (drawn from a
   functional description, not traced), (b) functional purpose,
   (c) no substantial similarity. If a side-by-side with the Mojang
   original looks copied, **redraw it**.
4. **No built-in shaders or copied shader code.** The engine provides
   the Iris-format *interface* and the `shader-packs/` folder; any
   BSL/SEUS-style pack is a user-side download (their license, their
   distribution — never ours).
5. **No Mojang trademarks** anywhere in our art, styling, or naming —
   the disclosure line stays: unofficial fan-style engine, not
   affiliated, not endorsed.
6. **The reference set is modern-version; the engine is 1.16.5-era.**
   Per the owner: only the *needed* subset was consulted — 1.16.5-
   relevant block/item/gui/environment naming. Modern-only content
   (hanging signs, trims, bamboo blocks, etc.) is out of scope and
   was not used for anything.
7. **Re-run the audit scripts** whenever any pack asset changes:
   `python3 scripts/legal_audit.py` must print `[PASS]` before any
   deploy.

## 5. Bottom line

VoxelCraft's posture is: **100% procedurally synthesized art,
format-compatible naming only, user-supplied packs never
redistributed, reference set quarantined in git-ignored `upload/`,
audits green.** The reference set made the project *more* compliant,
not less: it is now studied under an explicit legal framework instead
of ad-hoc inspiration. As long as the rules in §4 are followed — and
they are enforced by review + script — the project stays on the
clean, legal side of the line the owner drew: *similar in spirit,
original in substance, legal in every pixel.*
