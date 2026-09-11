# VoxelCraft vs. Minecraft 1.16.5 — Comprehensive Parity Backlog & Architecture Audit

This document tracks the clean-room architectural comparison, current implementation status, and technical backlog between VoxelCraft-Rust and **vanilla Minecraft 1.16.5 (The Nether Update)**.

All mechanics, numbers, and constants documented here are verified against authoritative public documentation (`minecraft.wiki`, vendor specifications like AMD GPUOpen, or technical Minecraft community research) or explicitly flagged as `[ESTIMATED / APPROXIMATION]`.

---

## 1. Executive Parity Status Matrix

| Subsystem | Vanilla 1.16.5 Scope | VoxelCraft Status | Backlog Gap & Clean-Room Action Items |
|---|---|---|---|
| **Blocks & Collision Shapes** | 700+ block IDs, multi-part states, non-cube voxel bounds | 506 block entries, 805 states (`vc-blocks`) | Complex shapes (stairs, slabs, fences, walls, lanterns) need sub-block AABB list. Full waterlogging flag. |
| **Villager Life & Jobs** | 15 professions, work stations, sleeping, golem summoning, trading tiers | 15 professions, 5 trading tiers, restock | Daily schedule AI (work, gossip at bell, sleep in bed), field harvesting/replanting, iron golem panic summoning. |
| **Farming Systems** | Wheat, carrots, potatoes, beetroot, melons, pumpkins, sugarcane, sweet berries, cocoa, nether wart | Basic crop growth, berry bushes, sweet berries | Hydration radius (4 blocks), bone meal growth stages, bee pollination acceleration, villager farmer automation. |
| **Technical Mechanics & Exploits** | TNT duping (BUD + coral notifier), carpet duping, sand duping, update suppression | Pure tick simulation | Deliberate parity for community farm designs: piston push limit (12 blocks), slime/honey separation, falling block portal transition timing. |
| **Entity Models & Recoil** | Steve/Alex humanoid models, quadrupeds, winged, multi-jointed entities | Procedural bounding box & hand renderer | 3D multi-part box models for all mobs, walking leg/arm swing cycle, head pitch/yaw tracking, clean-room damage recoil tilt. |
| **Enchantments & Books** | 38 enchantments, anvil combining, prior work penalty, librarian book trading | 38 enchantments implemented in `vc-gameplay` | Enchanted book item rendering with glint shader, anvil level progression & XP cost curve, librarian trade rolling. |
| **Vehicles (Boats & Minecarts)** | Rowable 2-seat boats, minecarts (hopper, chest, furnace, TNT) on 4 rail types | Basic entity movement | Boat rowing oar animation + splash sounds; minecart momentum physics, curves, sloped rails, and rail clatter audio. |
| **Dimensions & World Gen** | Overworld (79 biomes), Nether (5 biomes + bastions/fortresses), End (islands + cities + ships) | 25 Overworld biomes, Nether & End basic terrain | Nether Bastion Remnant variants (bridge, housing, stables, treasure), Ruined Portals, End gateway beam teleportation, End cities & Elytra ships. |
| **UI, Input & Menus** | Mouse release activation, drag cancellation, integer GUI scaling, FSR presets | UI canvas 960×540, mouse release activation, letterbox color match | 1.16.5 button release activation parity verified; X11/Wayland input deduplicated; pillarbox clear normalized (#EF323D Intro, #38281B Loading); integer GUI scaling. |
| **Weather & Atmosphere** | Rain, thunderstorms, snow, particle splashes, sky darkness, thunder claps | Sky cycle, daylight cycle | Weather state machine (Clear 12k–180k ticks, Rain 12k–24k ticks, Thunder 3.6k–15.6k ticks), rain/snow particle streaks, sky darkening curve. |

---

## 2. Deep-Dive Subsystem Specifications & Backlog

### A. Villager Economy, Schedules & Iron Golem Summoning
*(Source: `minecraft.wiki/w/Villager`, `minecraft.wiki/w/Iron_Golem`)*

1. **Daily Diurnal Schedule (24,000 Ticks / 20 Minutes)**:
   - `0 – 2000 ticks`: Morning gathering and wandering around village boundaries.
   - `2000 – 9000 ticks`: Working at their claimed job site block (Composter, Lectern, Blast Furnace, Smoker, Barrel, Loom, Stonecutter, Smithing Table, Grindstone, Cartography Table, Brewing Stand, Cauldron, Fletching Table). Checks workstation presence every 300 ticks to perform restock (up to 2 restocks/day).
   - `9000 – 11000 ticks`: Gathering around the village Bell to gossip, share gossip tags (`minor_positive`, `major_positive`, `minor_negative`, `major_negative`), and trade food.
   - `12000 – 0 ticks`: Retiring to their claimed Bed to sleep.
2. **Farmer Auto-Harvesting**:
   - Farmer villagers seek mature crops (Wheat stage 7, Carrots stage 7, Potatoes stage 7), harvest them into inventory, and replant seeds in tilled farmland.
3. **Iron Golem Summoning**:
   - **Panic Trigger**: In Java Edition 1.16.5, 3 or more villagers within 10 blocks of each other who have slept within the last 20 minutes (24,000 ticks) and are panicked by a line-of-sight hostile mob within 16 blocks will attempt to summon an Iron Golem.
   - **Exclusion Zone**: Villagers scan every 10 seconds and will not summon if an existing Iron Golem is detected within a 16-block radius ($\pm16$ on X, Y, Z) in the last 30 seconds.
   - **Player Built**: 4 Iron Blocks in a T-shape + 1 Carved Pumpkin / Jack o'Lantern on top.

### B. Technical Mechanics, Glitches & Vanilla Farming Systems
*(Source: Technical Minecraft community consensus & `minecraft.wiki`)*

1. **Piston & Slime Block Dynamics**:
   - Piston push limit: exactly 12 blocks.
   - Slime blocks and honey blocks do not stick to each other.
   - Immovable blocks: Obsidian, Crying Obsidian, Bedrock, Extended Pistons.
2. **TNT Duplication (Java 1.13–1.16.5 Standard)**:
   - Technical machines use an unignited TNT block in a BUD-powered configuration attached to a slime block alongside a dead coral fan and a rail.
   - When the sticky piston moves the assembly, the dead coral fan emits a notifier block update immediately before the intact TNT block transitions into the moving piston tile entity state, causing the TNT to ignite a primed entity while the original block is preserved and moved.
3. **Gravity Block / Sand Duplication**:
   - When falling block entities (sand, gravel, concrete powder, anvils) cross an End Portal or Nether Portal chunk border, the dimension teleportation logic clones the falling entity at the destination coordinates while tick boundary conditions delay destruction of the source entity, generating an extra block.
4. **Zero-Tick & Rapid Farming Mechanics**:
   - Piston extension/retraction cycles causing immediate block update notifications forced instant crop growth ticks in pre-1.16 versions; in 1.16, standard piston observers trigger automated harvesting upon maturity.

### C. 3D Entity Models, Recoil & Animations
*(Source: Client damage rendering standards & clean-room behavioral specification)*

1. **Model Hierarchy (Voxel Boxes)**:
   - **Humanoid (Steve & Alex)**: Head (8×8×8), Torso (8×12×4), Left/Right Arms (4×12×4 or 3×12×4 for Alex), Left/Right Legs (4×12×4).
   - **Quadrupeds (Cow, Pig, Sheep)**: Horizontal Torso, 4 independent legs, Head, Snout.
   - **Iron Golem**: Heavy Torso (18×12×11), Long swinging arms (6×30×6), Legs, Protruding nose.
2. **Damage Recoil & Procedural Animation**:
   - **Hurt Recoil**: During the 10-tick invulnerability window (`hurt_t > 0`), the entity model or billboard tilts horizontally using a quadratic ease into a sinusoidal oscillation peaking at $\approx 14^\circ$, combined with the red damage color tint (`[Clean-room Behavioral Approximation]`).
   - **Walking Cycle**: Sinusoidal opposing pitch for left/right legs and arms ($\sin(t \cdot \omega)$).
   - **Head Tracking**: Clamped pitch $[-60^\circ, 60^\circ]$ and yaw relative to torso $[-75^\circ, 75^\circ]$ towards camera/target.
   - **Death Animation**: $90^\circ$ side roll fall + white smoke puff particles.

### D. Enchantments, Enchanted Books & Anvil System
*(Source: `minecraft.wiki/w/Anvil_mechanics`, `minecraft.wiki/w/Enchanting_mechanics`)*

1. **Enchanted Books**:
   - Books can hold any of the 38 vanilla enchantments.
   - Procedural purple/cyan glint shader pass scrolling over 2D item icon or 3D held mesh.
2. **Anvil Combining Rules**:
   - Prior Work Penalty: exponential penalty cost $2^n - 1$ levels added to each subsequent repair or combination.
   - Renaming cost: +1 level.
   - "Too Expensive!" hard limit at 40 levels in Survival mode.
3. **Enchanting Table**:
   - Requires 0 to 15 bookshelves placed in a 5×5 perimeter 1 block away.
   - Lapis lazuli cost (1–3) + XP levels deducted (1–3), requiring player level 1–30.
   - Runic glyph hover revealing one guaranteed enchantment outcome.

### E. Dimensions: Nether & The End Parity
*(Source: `minecraft.wiki/w/The_Nether`, `minecraft.wiki/w/The_End`)*

1. **The Nether (1.16.5 Parity)**:
   - **Bedrock Ceiling**: Flat bedrock ceiling at Y=127; Lava sea surface at Y=31.
   - **5 Biomes**: Nether Wastes, Crimson Forest, Warped Forest, Soul Sand Valley, Basalt Deltas.
   - **Bastion Remnants**: 4 structural layouts (Bridge, Hoglin Stables, Housing Units, Treasure Room) with Piglin Brutes and gilded blackstone chests.
   - **Nether Fortresses**: Blaze spawners, Wither Skeleton spawning on nether bricks, Nether Wart rooms.
   - **Respawn Anchor**: Charges 1–4 with Glowstone; sets spawn in Nether; explodes with strength 5 in Overworld or End.
   - **Piglin Bartering**: Gold Ingot barter loot table (ender pearls, fire resistance potions, obsidian, soul speed books).
2. **The End (1.16.5 Parity)**:
   - **Central Island**: 10 Obsidian Pillars with Ender Crystals (some protected by iron bars), Bedrock Exit Portal with Dragon Egg podium.
   - **Ender Dragon Boss**: Circling, strafing dragon breath fireballs, perching on bedrock podium, healing tethers to end crystals.
   - **End Gateway Portals**: 20 circular bedrock portals opening around perimeter after dragon defeat, teleporting player to outer islands (1000 blocks distance).
   - **Outer Islands**: Chorus fruit plants, End Cities with branching rooms, Shulkers, and End Ships containing the Elytra in an item frame.

### F. Vehicles: Boats & Minecarts
*(Source: `minecraft.wiki/w/Boat`, `minecraft.wiki/w/Minecart`)*

1. **Boats**:
   - 2-passenger capacity (player can transport villagers, pigs, hostile mobs).
   - Dual-paddle rowing animation synchronized with Left/Right turn keys.
   - Water wake and paddle splash sound effects.
   - Extreme acceleration sliding on Ice, Packed Ice, and Blue Ice.
2. **Minecarts & Rail Network**:
   - Standard Rail, Powered Rail (boost when powered, brake when unpowered), Detector Rail (emits redstone signal when cart is present), Activator Rail (shakes passengers out, primes TNT carts).
   - Dynamic 90-degree track curves and sloped incline ramps.
   - Momentum conservation around loops.
   - Metallic wheel friction hum and rail joint click-clack procedural audio.

### G. FSR 1.0 Presets, GUI Scale & Weather Systems
*(Source: `AMD GPUOpen FidelityFX-FSR 1.0 Specs`, `minecraft.wiki/w/Weather`)*

1. **FSR 1.0 Quality Modes (GPUOpen Canonical Specifications)**:
   - **Ultra Quality**: 1.3× per-dimension scale factor ($\approx 77\%$ linear render scale).
   - **Quality**: 1.5× per-dimension scale factor ($\approx 67\%$ linear render scale).
   - **Balanced**: 1.7× per-dimension scale factor ($\approx 59\%$ linear render scale).
   - **Performance**: 2.0× per-dimension scale factor ($50\%$ linear render scale).
   - Dynamic RCAS sharpening lobe scaling: softer (0.4) on Ultra Quality to crisp (0.8) on Performance mode.
2. **GUI Scale Settings**:
   - Options: `Auto`, `1x`, `2x`, `3x`, `4x`.
   - Integer scaling calculating the maximum pixel multiple that fits window bounds without stretching fonts or blurring borders.
3. **Weather Engine**:
   - **Clear**: 12,000 to 180,000 ticks (0.5 to 7.5 Minecraft days).
   - **Rain**: 12,000 to 24,000 ticks (10 to 20 minutes).
   - **Thunderstorm**: 3,600 to 15,600 ticks (3 to 13 minutes), active only during rain.
   - **Transition Intensity**: Smooth linear ramp over 100 ticks (0.01 per tick) for `rain_level` and `thunder_level`.
   - Darkening sky light curve, rain streak particles, splash particles on top block faces, spatial thunder claps.
