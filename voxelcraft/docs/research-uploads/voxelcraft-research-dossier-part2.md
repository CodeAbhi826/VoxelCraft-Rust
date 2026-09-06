# VoxelCraft-Rust → Minecraft Replication: Research Dossier — PART 2
*Continues from Part 1 (voxelcraft-research-dossier.md). Additive only — nothing here duplicates Part 1. Hand both files over when ready to draft the master prompt.*

---

## 10. Live visual reference — Minecraft 26.2 "Chaos Cubed" (current release, June 16, 2026)

- **Versioning change**: Minecraft moved to year-based versioning in 2026 (26.1 "Tiny Takeover" → March 24, 2026; 26.2 "Chaos Cubed" → June 16, 2026). Bedrock runs parallel numbers (26.30 for this drop).
- **Sulfur Caves** — new Overworld underground biome: sulfur + cinnabar terrain bands, stalactite/stalagmite "sulfur spikes," noxious sulfur pools (inflict Nausea on contact), surface geysers (Potent Sulfur over magma, erupt at random intervals, can launch entities into the air).
- **Sulfur Cube** — new passive mob, slime-like, absorbs nearby blocks and changes behavior/appearance based on what it ate (TNT → explosive variant, magma → damaging "hot" variant, stone → slower bouncing variant, blue ice → fast chaotic movement).
- **Two new block families**: Cinnabar (reddish) and Sulfur (yellow), each with the standard stairs/slabs/walls/polished/brick/chiseled variant set.
- **Cave spiders now spawn naturally in the wild** for the first time (previously spawner-only).
- **Experimental Vulkan renderer (Java)** — Mojang moving away from the OpenGL pipeline used since 2009. Validates VoxelCraft's original wgpu/Vulkan-first architecture choice.
- **Friends List system** added directly in-game menus (Java & Bedrock) — not relevant to VoxelCraft's singleplayer-only scope, noted for completeness only.
- **Framing reminder**: the *concept* (themed hazard biome + material-reactive mob) is freely usable; exact colors/textures/model are Mojang's expression and must be independently designed if VoxelCraft builds something similar in spirit.

---

## 11. Farm-enabling mechanics / historically-bug-origin features — status and source pointer

Legal note: these are *emergent behavior from mechanical rules*, not designed expression — if anything, further from "copyrightable" than intentional features, since they're side effects of correctly implementing the underlying rules.

**Verified with real sourcing (not memory) this round:**
- **Quasi-connectivity** — redstone signal propagating one layer above its source without a direct connection. Originated as a bug; Mojang's own Java tech lead (slicedlime) has publicly confirmed it was deliberately left in once the community began relying on it.
- **TNT duping** — exploits entity-death/item-drop timing desync so a primed TNT entity is created without consuming the source block. Also publicly acknowledged by Mojang's tech lead as intentionally preserved, specifically because the technical-Minecraft community relies on it for tree farms and large-scale terrain removal ("world eaters").
- **Raid farms** — killing an illager "captain" (found at pillager outposts, woodland mansions, patrols) outside raid range grants Bad Omen (stacks to level V from multiple captains in Java Edition). Entering any village-recognized chunk with Bad Omen triggers a raid: 5–7 escalating waves of pillagers/vindicators/evokers/witches/ravagers. Players farm this by killing captains near an elevated artificial village, funneling raiders into a kill chamber for Totems of Undying, emeralds, enchanted gear.
- **Basic mob-farm spawn mechanics** (not a bug, but the mechanical basis for nearly every grinder design): hostile mobs require light level 0 to spawn; there's a per-player mob cap; dark enclosed rooms + water/trapdoor funnels are the standard exploit of these two rules.

**Source pointer for the full catalog (confirmed to exist, not yet pulled in full):**
`minecraft.wiki`'s `Category:Tutorials` / `Tutorials/` namespace contains individual documented pages for exactly this category, including explicitly bug/exploit-framed ones: `Tutorials/Zero-ticking`, `Tutorials/Block and item duplication`, `Tutorials/Breaking bedrock`, `Tutorials/Instant repeaters`, `Tutorials/Headless pistons`, `Tutorials/Indestructible ender crystals`, `Tutorials/Villager farming`, plus a general `Farming` category page. **A dedicated pull through this namespace is the legitimate way to build the full "100+" list — flagged as a specific future research task, not attempted by enumeration from memory.**

---

## 12. World creation / seed system

### Real Minecraft's world-creation flow
- **World Type** options: Normal · Superflat (fully customizable: ordered list of `{block, height}` layers, pure JSON/mechanical, e.g. `generator-settings={"biome":"minecraft:plains","layers":[...]}`) · Large Biomes (biomes scaled up, commonly cited ~4×) · Amplified (extreme terrain to Y=256, CPU-heavy, shown with a warning in-game) · Single Biome/buffet (entire world is one biome) · Debug Mode (fixed grid of every block state, for testing/mapmaking).
- **Seed input** — random or custom text/number seed (text seeds are hashed to a number).
- **Bonus chest** toggle, **world border** setting, reduced debug info — all set at creation time.
- `level-type` values (server-side, same underlying system): `minecraft:normal`, `minecraft:flat`, `minecraft:large_biomes`, `minecraft:amplified`, `minecraft:single_biome_surface`. Legacy pre-1.13 names still referenced in some tooling: `buffet`, `default_1_1`, `customized`.

### Confirmed gap in VoxelCraft (checked against actual code this round)
**There is no "New World" creation screen or flow at all.** `GameApp::new()` calls `World::random_seed()` automatically; on native, it restores whatever single save already exists (`World::new(meta.seed)`) if one is found. No seed-input field, no world-type choice, no world naming, no bonus chest, no border setting exist anywhere in the UI code. This is more foundational than the settings/options gaps already logged in Part 1 — there's currently no player-facing way to start a *new*, *named*, *seeded* world at all, only an implicit single continuous save.

---

## 13. Files to gather from the user's own legally-owned Minecraft install

| File/data | Location | Legal status |
|---|---|---|
| `options.txt` / `optionsof.txt` / `optionsshaders.txt` (more variants) | Win: `%appdata%\.minecraft\` · Mac: `~/Library/Application Support/minecraft/` · Linux: `~/.minecraft/` | Safe — already have 3, more welcome |
| Generated data reports (`blocks.json`, `registries.json`, `commands.json`) | Run `java -jar server.jar --reports` once with a downloaded vanilla server jar → `generated/reports/` | Safe — official Mojang tool-developer export |
| Vanilla datapack JSON (recipes/loot_tables/tags/advancements) | Inside client/server jar (it's a zip) at `data/minecraft/...` | Safe — mechanical data |
| `sounds.json` | Inside client jar, `assets/minecraft/sounds.json` | Safe — event-name/category schema only, not audio |
| `level.dat` from an existing world | `saves/<world name>/level.dat` (binary NBT) | Safe — schema/tag structure reference |
| Textures / models / `.ogg` audio | Inside client jar, `assets/minecraft/textures\|models\|sounds/` | **Reference only** — never extract-and-reuse, redraw/re-record independently |

---

## 14. Open items carried forward (not yet resolved)

- Full farm/glitch catalog — needs the dedicated `minecraft.wiki Tutorials/` namespace pull (§11).
- World-creation/seed UI — confirmed as a real, foundational gap (§12), not yet in any phase list.
- All open decisions from Part 1 §8 still stand (license choice, phase priority order, Iris sister-project confirmed but not detailed, numeric ground truth still pending the generated-reports pass).
