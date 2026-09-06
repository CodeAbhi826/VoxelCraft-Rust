# VoxelCraft-Rust — Visual & Mechanical Verification Report (1.16.5 parity)

**Round:** 2026-09-06, against the post-bracket-1 state (commit d60e62f + this
round's fixes). Method: live browser E2E on WebGL2/SwiftShader (in-game,
HUD pixel measurement, F3 + container screenshots), code inspection of
`ui.rs` / `player.rs` / `combat.rs` / `furnace.rs` / `fluids.rs` / mesh
pipeline, and **live wiki verification** of every asserted number this
round (per the strict protocol — nothing taken from the old research dumps).
Environment note: the headless verifier rasterizes in software (~10 fps,
sim advancing sub-realtime), so the "10 s walk" in-game timing check was
replaced by code-constant + convergence-test verification; all pixel
measurements were taken at the 1280×720 default window.

**Two live-confirmed wrong values were fixed this round** (details at the
end): day-night cycle 10 min → **20 min** (24000 ticks @ 20 tps), wooden
slab fuel 300 → **150 ticks**.

---

## §1 Font / Text

| Item | Verdict | Evidence |
|---|---|---|
| NEAREST filtering on UI | ✅ | `render.rs` ui_samp mag/min Nearest |
| Char advance 6 px (vanilla spacing) | ✅ | `text()` 6·scale advance |
| 8 px char height (7 + descender) | ⚠️ | engine font is 5×7, **no descenders** — 'p','g','q' render like smallcaps |
| Lowercase glyphs | ❌ | lowercase → uppercase "smallcaps look" in `text()` |
| Proportional widths (vanilla: 'i' narrow, 'w' wide) | ❌ | monospace 6 px for all glyphs |
| Shadow offset +1,+1 down-right | ✅ | `set(dx+1, dy+1, …)` |
| Shadow = 25 % of glyph color (white → #3F3F3F) | ❌ | engine uses solid **black** shadow (checklist's "~50 % brightness" claim is also off — vanilla multiplies by 0.25) |
| 16 chat colors | ⚠️ | colors exist engine-side; no § code parser in the text renderer |
| §l bold / §o italic / §n underline / §r reset | ❌ | not supported |
| Monocraft substitution (legal) | ⚠️ | OFL-licensed Monocraft still planned; current engine font is a clean-room 5×7 bitmap |

## §2 HUD

| Item | Verdict | Evidence |
|---|---|---|
| Hotbar proportions 182×22 (vanilla-eq) | ✅ | `hotbar()`: 364×44 UI px = exactly 182×22 × 2 |
| 9 slots, 20 px pitch (vanilla-eq) | ⚠️ | slot pitch 40 UI = 20 vanilla-eq ✓, inner 36 UI = 18 cell ✓ — but see §4: effective on-screen scale is non-vanilla |
| Selection frame 24×23 (vanilla-eq) | ⚠️ | engine 40×40 UI = 20×20 vanilla-eq — slightly smaller than vanilla's 24×23 |
| Hearts row above hotbar, left half | ✅ | live screenshot + `status_bars()` |
| Hunger row above hotbar, right half | ✅ | live screenshot |
| 10 hearts / 10 drumsticks | ✅ | live screenshot (20 HP, 20 food) |
| Heart sprite 9×9 (vanilla) | ⚠️ | clean-room 8×6 sprite (16×12 UI) |
| XP bar 182×5 (vanilla-eq) | ⚠️ | engine 364×8 UI = 182×4 vanilla-eq (2 px fill vs vanilla 5) |
| XP level number above bar, green | ✅ | `text_outlined` green |
| Crosshair 15×15 center | ⚠️ | engine 16×16 UI = 8×8 vanilla-eq white + dark outline (vanilla: 15×15 invert-blend) |
| Held-item name above XP bar, ~2 s fade | ✅ | `held_name_t = 2.0`, fades with dt |
| Oxygen bubbles (10 × 30 air, right above hunger) | ✅ | verified live round 2026-09-05 |

## §3 Containers

| Item | Verdict | Evidence |
|---|---|---|
| Hopper height 133 (vanilla 176×133) | ✅ | `top_h 52` → panel height exactly **133** vanilla-eq; "ITEM HOPPER" label ✓ |
| Chest/furnace/crafting/brewing/enchant/trade screens | ✅ | all open + interactive (E2E paths exist) |
| Slot cells 18×18 (vanilla-eq) | ✅ | 36 UI px inner + frame |
| Panel width 176 (vanilla-eq) | ❌ | engine panels ≈ **196** vanilla-eq wide (grid_w + 28 UI) — ~20 px wider than every vanilla screen |
| Panel style #C6C6C6 light grey | ❌ | engine uses dark translucent [26,26,30,235] — deliberate custom theme, disclosed |
| Inventory: armor slots (4) | ❌ | not present |
| Inventory: player model preview | ❌ | not present |
| Inventory: 2×2 craft + output, 3×9 storage + hotbar row | ✅ | verified live |

## §4 Settings

| Item | Verdict | Evidence |
|---|---|---|
| FOV 30–110 slider | ✅ | `clamp(30,110)`, `30 + t·80` |
| Brightness Moody→Bright | ✅ | 0–100 % with MOODY label |
| Render distance 2–32 chunks | ❌ | engine clamps **2–16** (separate sim-distance 5–32 verified) |
| GUI Scale Auto/1/2/3/4 | ❌ | **not present** — UI is a fixed 960×540 canvas stretched to the window: effective scale 2.67 at 1280×720 (non-integer → uneven nearest-pixel doubling; vanilla scales are integers) |
| 10 sound sliders (Master/Music/Jukebox/Weather/Blocks/Hostile/Neutral/Players/Ambient/Voice) | ❌ | engine has 2 (VOLUME, MUSIC) |
| Graphics Fast/Fancy | ✅ | |
| Smooth lighting toggle | ✅ | |
| Clouds toggle | ✅ | |
| Mipmap levels | ⚠️ | fixed 4 (setting exists but not the 0–4 slider) |
| Max framerate | ✅ | |
| V-Sync | ✅ | |
| Sensitivity | ✅ | |
| Engine extras (shadows, FSR upscale, shader mode) | ✅ | beyond-vanilla extras, documented |

## §5 F3 debug screen

| Item | Verdict | Evidence |
|---|---|---|
| XYZ 3-decimals | ✅ | live screenshot |
| Block / Chunk with in-chunk coords | ✅ | live |
| Facing + yaw/pitch (vanilla convention) | ✅ | live (`FACING: SOUTH (TOWARDS POSITIVE Z) (45.0 / -0.6)`) |
| Client Light L (sky, blk) from real light engine | ✅ | live (15 (15 sky, 0 blk)) |
| Biome + Dimension lines | ✅ | live |
| Looking at block / fluid split | ✅ | live (1.13+ split, valid 1.16.5) |
| Left-column line count (checklist: "28") | ⚠️ | engine ~24 lines; the checklist's 28-item list was already judged unverified (version drift) — core lines all present |
| Right column (Java/mem/CPU/GPU/display) | ⚠️ | merged into a left BACKEND line (engine-adapted, disclosed in worklog) |
| F3 sub-hotkeys (F3+G chunk borders etc.) | ❌ | not implemented |

## §6 Mechanics

| Item | Verdict | Evidence |
|---|---|---|
| Tick rate 20/s | ✅ | fixed 20 Hz substeps; `simTicks` tracked (live box renders sub-realtime — environment, not engine) |
| Walk 4.317 b/s | ✅ | `WALK_SPEED` + convergence test; **live-verified** w/Walking |
| Sprint 5.612 b/s | ✅ | constant + test; **live-verified** w/Sprinting |
| Sprint-jump 7.127 b/s | ⚠️ | jump 0.42 b/t + sprint physics present, but no test asserts the emergent 7.127 figure |
| Gravity 0.08 b/t², drag 0.98, terminal 3.92 b/t | ✅ | exact `v1=(v0−0.08)×0.98`; terminal convergence test; live-verified formula |
| Jump 0.42 → 1.25-block apex | ✅ | substep phase alignment (prior round) |
| Crit ×1.5, falling + cooldown ≥ 84.8 % + not sprinting | ✅ | `combat.rs` (wiki-cited) |
| Attack cooldown `0.2 + 0.8p²`, ticks = 20/attack_speed | ✅ | `combat.rs` (wiki-cited) |
| Sweep: sword only, cooldown ≥ 84.8 % | ✅ | `combat.rs` |
| Water spread 1 lvl/5 ticks, max level 7 | ✅ | `WATER_TICK_RATE = 5`; levels 1..7 |
| Lava: Overworld 1/30 ticks spread 3, Nether 1/10 spread 7 | ❌ | **lava fluid simulation absent** (static lava blocks; live-verified the vanilla values for when it lands) |
| Smelting 200 ticks | ✅ | `COOK_TICKS = 200`; live-verified w/Smelting |
| Coal item 1600 ticks | ⚠️ | engine: COAL_ORE 800 "ore-as-fuel" stopgap (coal item doesn't exist yet) — documented deviation |
| Planks/logs 300, crafting table 300, fence 300 | ✅ | live-verified this round |
| Wooden slab 150 | ✅ | **fixed this round** (was 300) — live-verified w/Smelting |
| Day-night cycle 10 min (checklist) | ❌ | **checklist itself wrong + engine fixed this round**: vanilla = **20 min** (24000 ticks), live-verified w/Daylight_cycle |
| Drowning: 300 air, 2 HP at −20, regen 30/4 ticks | ✅ | prior live round |
| Mob fall damage = distance − 3 (MC-12357) | ✅ | prior round, Mojang-cited |

## §7 Rendering

| Item | Verdict | Evidence |
|---|---|---|
| 16×16 block textures, 256×256 atlas | ✅ | `TILE_PX = 16`, procedural clean-room |
| NEAREST base filtering | ✅ | terrain + UI samplers |
| No atlas bleeding (half-texel inset) | ✅ | `clamp(fract(uv), 0.03125, 0.96875)` + `textureSampleGrad` (fe70cd9); re-verified visually this round — clean block tiling |
| Smooth lighting + vanilla AO corner rule | ✅ | `if s1 && s2 {0} else {3−(s1+s2+corner)}` + 4-corner sky average; default on |
| Light levels 0–15 | ✅ | `LIGHT_REACH = 15` |
| Day/night = 20 min | ✅ | **fixed this round** (was 600 s → `DAY_LEN_SECS = 1200`) |
| Biome grass/foliage/water tints | ✅ | Phase 10, live-verified |
| Occlusion culling, region-loop draws, mesh budget | ✅ | live stats: 62 drawn/225 loaded, 64 draw calls, zero per-chunk binds |
| Textures seams / "connection in textures" | ✅ | resolved (fe70cd9 + 1a2bc6d); current live screenshots show no boundary artifacts |
| Render lag | ✅/⚠️ | engine-side costs addressed (prior rounds); the ~10 fps on this box is SwiftShader software rasterization, not the engine — real-GPU behavior differs |
| Wasm bundle current | ✅ | rebuilt this round from the bracket-1 tree |

---

## Fixed this round (live-confirmed corrections)

1. **Day-night cycle 600 s → 1200 s** — vanilla 1.16.5 = 24000 ticks @
   20 tps = 20 min (live: minecraft.wiki/w/Daylight_cycle, w/Tick). The
   old value traced to the checklist/research docs (which themselves
   claimed 10 min — noted as a checklist error). Regression test
   `day_cycle_is_the_vanilla_20_minutes`.
2. **Wooden-slab fuel 300 → 150 ticks** — live: minecraft.wiki/w/Smelting
   fuel table ("Wooden Slab 7.5 s / 150 ticks"). Regression tests
   `fuel_table_matches_the_live_wiki` + `slab_burns_half_as_long_as_planks`.

Suite: **342/342 green** (339 → +3), wasm32 target clean.

## Priority fix list (what the user should decide next)

**Mechanical (small, well-defined):**
1. Render-distance slider range 2–32 (currently 2–16).
2. Lava fluid simulation (OW 1/30 ticks spread 3; Nether 1/10 spread 7 —
   values live-verified and waiting in the verdicts doc).
3. Sprint-jump 7.127 b/s emergence test.
4. Coal item + 1600-tick fuel (needs the item-system milestone).

**Visual (design decisions — bigger work):**
5. GUI Scale option (Auto/1/2/3/4) + integer-scaled UI (replaces the fixed
   960×540 stretch; also fixes the non-integer pixel doubling).
6. Vanilla-style light-grey container theme (#C6C6C6) + exact 176-wide
   panels + armor slots + player model on the inventory screen.
7. Font upgrade: 8 px height with descenders, proportional widths,
   color-derived shadow (25 %), § code support — or adopt Monocraft
   (already cleared legally).
8. Vanilla-exact selection frame (24×23), XP bar height (5 vanilla-eq),
   crosshair size (15×15) — micro-adjustments to existing sprites.
9. 10-channel sound mixer (engine has 2).
10. F3 right column + F3 sub-hotkeys.

None of the visual items break gameplay; they are parity polish. The
mechanical list is small enough to fold into the next bracket (1.3–1.4).
