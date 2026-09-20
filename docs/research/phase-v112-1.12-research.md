# Phase 1.12 research record — MC 1.12 "World of Color Update" bracket (live round, 2026-09-07)

The strict protocol's per-bracket research record: what was checked,
against which live page, and every disagreement/adaptation. Raw page
fetches: `voxelcraft/scripts/v112_page_*.json` (+ `_text.txt`
extracts), fetched live 2026-09-07 pre-implementation.

## Verified live (minecraft.wiki unless noted)

| Value | Page | Note |
|---|---|---|
| 1.12 "World of Color Update", released 2017-06-07; headline: vibrant palette, glazed terracotta, concrete, colored beds; 2 new mobs (illusioner, parrot); advancements + functions | w/Java_Edition_1.12 (changelog capture) | the bracket's scope source |
| Concrete: created when concrete powder contacts still/flowing water; hardness 1.8 | w/Concrete + changelog | |
| Concrete powder: 16 colors, gravity affected (like sand/gravel); turns into concrete when it touches water; recipe 4 sand + 4 gravel + 1 dye → 8, **shapeless** | w/Concrete_Powder + changelog §Blocks | both sources agree "shapeless"; the engine's first truly shapeless 9-slot recipe |
| Glazed terracotta: smelt any stained terracotta; 4-directional facing; hardness 1.4; smelting 0.1 XP | w/Glazed_Terracotta | the per-rotation top/bottom art is clean-room (the wiki texture is Mojang's) |
| Parrot: 6 HP, passive, tamable, spawns in jungle biomes; speed 0.2 (infobox) | w/Parrot | |
| Parrot taming: wheat/melon/pumpkin/beetroot/torchflower/pitcher seeds; **1⁄10 chance per feed**; once tamed, interacting toggles sit | w/Parrot §Taming | engine: seeds as a palette item (no crop system for the exotic seeds — disclosed) |
| Tamed parrot follows the player, **teleports at 12 blocks** distance | w/Parrot §Taming | |
| **Feeding a cookie kills a parrot** (Java: 2128 damage + Poison particles; Bedrock: Fatal Poison) | w/Parrot (the cookie paragraph) | engine adaptation: instant death, poison-free (the poison particle path is a visual the engine defers) — the lethality is the mechanic |
| Parrot drops 1–3 XP; cannot be bred; no baby parrots | w/Parrot §Drops/§Breeding | |
| Parrot spawn weight: jungle leaves+grass roll, 40/93 = 43.01% share; groups 1–2 | w/Parrot §Spawning | the page's table carries the 1.12-era weights |
| Parrot 5 variants: red, blue, green(lime), cyan, gray (Variant NBT table) | w/Parrot | uniform color pick (no biome weighting in 1.12-era data — disclosed) |
| Illusioner: 32 HP, hostile, speed 0.5; no natural spawn, **no spawn egg** (raid-only in Java 1.12+) | w/Illusioner infobox | palette-only mob: summoned by tests, never spawns |
| Casting Blindness: 20 s on first engaging a new player opponent; arm-raise tell + low-pitch sound + black mist | w/Illusioner §Casting_Blindness | regional-difficulty ≥2 gate (1.12) → ≥3 (1.12.2) not modeled — engine has no regional difficulty (disclosed; the spell always casts on first engage) |
| Illusioner defensive: Invisibility spell with 4 false duplicates refresh | w/Illusioner | engine form: 60 s invisibility + duplicate refresh cycle |
| Blindness effect: id 15, negative; "Impairs vision by adding close black fog and disables the ability to sprint and critical hit" | w/Effect §Blindness | render layer pulls fog in; sprint blocked; crits = 1.9-combat detail not modeled (disclosed) |
| Glazed terracotta smelting XP: 0.1 per block | w/Glazed_Terracotta §Smelting | furnace XP row |
| 16 dyes feed the powder recipe (any of the 16 colors) | w/Concrete_Powder §Crafting | dye ACQUISITION economy deferred — dyes are palette items (standing disclosure) |

## Disagreements / version traps caught

- **Regional-difficulty gate on Blindness**: the illusioner page says
  the spell needs regional difficulty > 2 (raised to 3 in 1.12.2).
  The engine has no regional difficulty concept; the spell casts on
  first engagement — disclosed rather than invented a fake gate.
- **The changelog's "16 new colored beds"**: out of engine scope (no
  bed block in the registry) — recorded as a formal deferral, not a
  silent skip.
- **Advancements + functions** (the update's flagship systems): no
  advancement engine and no command parser exist — formally deferred
  (both recorded in the WORKLOG with reasons).

## Deferred with reasons

- colored beds (no bed block), the recipe book / knowledge book (no
  recipe-UI system), advancements (no advancement engine), functions
  (no command parser), iron nugget (no nugget item), crafting-tweak
  gamerules (no gamerule system), dye acquisition as an economy
  (palette items — the standing disclosure), parrot imitations of
  monster sounds (no monster-sound events to imitate — disclosed),
  shoulder perching (no shoulder slot).

## Adaptations (engine-shaped forms of wiki facts)

- Cookie death: instant death replaces the 2128-damage + Poison
  particle path (same observable outcome: the parrot dies).
- Parrot wing animation + sit toggle are visual states; the flying
  steering uses the gentler vex-style seek (no per-mob physics
  tuning for parrots in the engine).
- Concrete powder solidification checks BEFORE gravity, so powder
  adjacent to water solidifies even while unsupported — matches the
  wiki's "when it touches water" wording (covers falling-into AND
  placed-next-to).
