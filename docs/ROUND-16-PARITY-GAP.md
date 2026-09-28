# ROUND-16-PARITY-GAP.md — the remaining known gaps

> 2026-09-26: the consolidated, prioritized roadmap built from this ledger +
> the test-report + the verification report now lives in **`docs/MASTER-PLAN.md`**
> — that file is the single plan of record; this ledger stays as the evidence
> trail and keeps receiving same-round updates per the standing rule.

Date: 2026-09-18 (updated; first cut 2026-09-15) · the honest ledger
of what is NOT yet vanilla-parity, with size estimates (the spec's
Round 16 [1] deliverable).

## Shipped since the 2026-09-15 cut

- **Round 10 [1]** — the vanilla integer GUI-scale model (Auto =
  max(1, min(⌊w/320⌋, ⌊h/240⌋)), live-canvas logical space, integer
  device-px-per-vanilla-px blit; the fractional 0.72/0.86/1.0 path
  removed). Items [2]–[7] (armor registry, survival inventory, armor
  HUD wiring) shipped in the prior sub-rounds 2–3.
- **Round 11** — the 11-tab creative inventory (prior session, commit
  b477d24; re-verified).
- **Round 12 [1]** — the double chest (partner scan, 54-slot merged
  screen, spill-both-halves on break).
- **Round 12b — mount storage CLOSED (2026-09-17)** — the
  per-entity MountStorage model (donkey/mule 15; llama 3×strength),
  the mount SCREEN (saddle slot + 5-wide grid / STR badge + partial
  rows), chest-equip through the use-interaction, E-while-riding, and
  the death spill (chest + contents). E2E-verified live.
- **Round 13 — station GUIs CLOSED** — anvil (Repair & Name: combine
  math, prior-work penalty, rename, Too Expensive!), grindstone
  (Repair & Disenchant + XP), beacon (power selection + payment +
  level gates). E2E-verified live 2026-09-17.
- **Round 14 (partial) + 14b** — Music & Sound (10 sliders + the
  2026-09-17 tooltip completion), Controls (rebind table), Language,
  Chat Settings (real screen), Accessibility completion (Hold/Toggle,
  Distortion/FOV Effects), Skin Customization (8 layers + Main Hand).
- **Round 15 (partial) + 15b** — biome fog; the 20-kind particle
  batch (splash/bubble/drips/portal/end_rod/firework/squid_ink/dust/
  note/villager moods/snowflake/totem/spit); the sky exactness (sun
  30×30, moon 20×20 at distance 100, pinned star density).
- **Saved hotbars CLOSED (2026-09-17)** — the 12-tab creative strip
  (Search/Hotbars/Inventory), C+n save + X+n load (the vanilla
  defaults, held-state combos), options-store persistence
  (hotbar0..8), the tab grid with placeholder rows. E2E-verified
  live (both boot logs + the tab screen).
- **Round 17 — the hunger-drain gameplay system CLOSED (2026-09-18)**
  — the standing deferral closed: the full vanilla FoodData model
  (`vc_gameplay::hunger`: foodLevel / foodSaturationLevel /
  foodExhaustionLevel / foodTickTimer), the exhaustion-accumulation
  hooks (sprint 0.1/m, swim 0.01/m, jump 0.05, sprint-jump 0.2,
  attack 0.1 landed, damage 0.1, mining 0.005, Hunger effect
  0.005/tick/level, regen 6.0/HP), the natural regeneration (80-tick
  cadence) + the Java-only saturation boost (10-tick cadence), the
  starvation branch with the per-difficulty stop thresholds (Normal
  1 HP / Hardcore Hard-class never-stops), the sprint gate (food > 6,
  mayfly bypass), the live HUD food bar (replacing the hardcoded
  20/20) + the saturation-zero hunger-bar jitter, the FoodData eat
  path (the hunger/2 direct-heal convention RETIRED — the full
  nutrition/saturation table re-verified live against the Food
  page), the respawn reset (20/5), and the cadence defect fix (the
  effects tick + lava contact damage ran per FRAME — now per 20 Hz
  sim tick). All wiki-cited live 2026-09-18
  (docs/research/round-17-hunger-audit.md).

## Deferred (with reasons)

| Gap | Reason | Estimate |
|---|---|---|
| Round 13 loom / stonecutter / cartography / smithing / fletching | The SPEC'S OWN deferral clauses: banners, map items, the stone-family recipes, netherite gear and arrows are not registered (audit §4 verdicts) | blocked on their subsystems |
| Container-panel family chrome (+~7 vanilla-eq-px vs the 176-wide panels) | A full panel-chrome rework touches every container screen (disclosed in the round-12 audit) | ½ round |
| Saved-hotbar counts in the tab grid | The creative grid is the u16 item grid; the X+n load preserves counts but the tab view does not render them | trivial, cosmetic |
| The in-game nether portal (Phase 12A of the master plan) | Newly identified in the 2026-09-18 master-plan review: the Nether dimension itself is COMPLETE (five biomes, fortresses, 8:1 coords, respawn anchors, piglin/hoglin content, nether fog) and reachable through the §28 travel pipeline + the E2E `dim:` command, but the obsidian-frame portal block + the flint-and-steel item + the walk-in trigger do not exist (the End portals are the only in-game dimension path; the fire block exists via lightning only). A portal round needs: the PORTAL block registration + frame validation + flint-and-steel item + portal particles/fog overlay + the walk-in travel + spawn-frame building on the far side | 1 round |
| Hunger NBT persistence (foodLevel/foodSaturationLevel/foodExhaustionLevel in the player block) | The engine's level.dat player NBT carries no health/xp either — every world entry is a fresh 20 HP / 20 food (the wiki's own world-creation semantics). Riding the existing convention this round; a future persistence round carries the whole vital-stat set | with the persistence round |
| Peaceful / Easy difficulty rules | The engine has no difficulty selector (worlds run Normal-class starvation; Hardcore runs Hard-class per the modes doc). The two unreachable rule rows stay disclosed | with a difficulty selector round |

## Known issues (observed, not fixed this pass)

- (none new — the three latent defects from the interrupted commits
  were fixed 2026-09-17: the name-pool underflow, the R13 state
  decode window, the missing music-slider tooltips; the frame-cadence
  effects/lava defect was fixed 2026-09-18 with the round-17
  per-sim-tick block)

## Legal audit notes (Round 16 [2])

- Grep for "the reference game/the original publisher/the original author/Creeper" in user-facing strings: the
  mob display name "Creeper Spawn Egg" (blocks.rs) is a pre-existing
  naming choice from the earlier audit16 rounds (vanilla's own item
  name, shown on VLM-verified screens); flagged for a dedicated
  string-freeze pass rather than a mid-round rename.
- The wasm bundle pair (voxelcraft.js + voxelcraft_bg.wasm) is the
  matched 2026-09-18 rebuild (wasm-bindgen 0.2.127, glue
  patched, packs rsynced from builtin-pack/ +
  builtin-packs/classic-art/ — procedural PNGs only).
- README disclaimer: the standing clean-room notice, unchanged.

## 2026-09-26 full-codebase gap audit (pre-next-round check)

Grep-level audit of the actual crates (not the older docs) to re-baseline
what exists vs what is still missing. Several "backlog" entries are STALE —
the features shipped under later round names without this ledger being
updated:

**Exists (verified in code, ledger was stale):**
- Weather state machine — `vc-gameplay/src/weather.rs` (Clear/Rain/Thunder
  with vanilla tick windows, lightning `can_strike`, `sleep_reset`,
  `sky_factor` wired into `sim.sky_factor`, rain loop `weather/rain` in the
  sound registry, thunderstorm mob flag). 
- Redstone §25 core incl. pistons — `vc-sim/src/redstone.rs` (1,825 lines:
  PISTON/STICKY_PISTON blocks, PISTON_PUSH_LIMIT=12, unpushable table,
  break-on-push family, repeater/comparator tiles).
- Ender dragon fight + exit-portal + dragon-egg sequence — `game.rs` Phase
  E1 (ray-vs-dragon-AABB, End_Crystal power 6, `dragon_defeated` gate) +
  dragon/wither boss bars (HUD sweep).
- Multi-box 3D mob models — 10-box spider rig class, limb-swing accumulator
  (`mobs.rs`); the old "procedural billboard only" backlog row is obsolete.
- Structures beyond villages: mineshafts (MINESHAFT_CHANCE), dungeon rooms,
  Nether fortresses (432×432 regions, Phase E1).
- Creative HUD hides status rows (B-2) — pinned by
  `creative_hud_hides_all_status_rows`.
- Item entities + XP orbs (attraction + pickup), villager gossip (incl.
  cure major_positive), mounts w/ MountStorage, spawn-egg items.

**Still missing (newly confirmed or re-confirmed, ranked):**
1. Weather PRESENTATION — no rain/snow streak particles, no surface-splash
   particles, no spatial thunder-clap event (strike logic + loop sound
   exist). Small round: a weather particle layer + one audio event.
2. TNT block + primed-TNT entity — explosions exist only via creeper
   blasts; the block/entity/fuse/chain-reaction are absent (blocks.rs has
   only a state-bracket comment). Medium round.
3. Beds — no BED block at all ("beds are deferred" per mobs.rs): no
   sleep-to-skip-night, no spawn-point set, phantoms not sleep-gated (the
   sleep STATISTIC exists). Medium round, needs 2-state multi-part block.
4. Vehicles — boats and minecarts/rails (all 4 rail types, curves, powered
   boost) have zero code. Large round; pairs naturally with piston-movable
   entities.
5. Bone Meal item — absent (bee pollination does the stage-advance role);
   the crafting recipe exists (bone meal → white concrete powder is a
   different crafting use — no, that was bone-meal-as-ingredient check;
   the ITEM itself is not usable on crops). Small round once items carry
   use-actions.
6. Waterlogging as a general blockstate — only the contact-form rule in
   fluids.rs; stairs/slabs/fences cannot hold water. Medium, touches the
   state registry.
7. Bastion remnants — not generated (gilded blackstone + obsidian traces
   are the disclosed stand-in). Large structure round.
8. End tail — End cities/ships, shulkers, end-gateway teleportation and
   the elytra are absent (exit portal + dragon egg exist). Large round.
9. Villager daily schedule — workstation claiming + timed restock exist in
   trading, but no work/bell/sleep schedule AI, no farmer auto-harvest.
   Large AI round.
10. Wither boss — wither skeleton + boss-bar painter exist, but no wither
    entity or soul-sand/skull build ritual (audit found no code; verify
    before planning). Medium-large.

Unchanged from the table above: the in-game nether portal round, hunger
NBT persistence, difficulty selector. Shader/resource packs on disk need
no plan item — see the shader-packs README; the §34.2 v1 scope note (no
texture/depth access for packs) remains the disclosed boundary.
## Round A — input & camera parity (2026-09-28, executed per MASTER-PLAN.md Part III)

**Shipped (all verified through the real input path, native-first):**
- **F5 third-person camera** — vanilla 3-state cycle (first → third-behind →
  third-front → first; `next_camera_mode`, unit-tested), eye offset 4.0 back /
  1.5 front, 0.25-step ray clamp against solid terrain, mode-2 looks back at
  the player (yaw+π, −pitch). Player body renders from the multi-box rig via
  the new clean-room skin tile (`TILE_MOB_PLAYER` = 793, `TILE_MAX` bumped;
  `player_skin_art()` painter + atlas arm; `EntityModel::player()` =
  humanoid(tile, arms_forward=false)); walk-cycle anim from `sample_anim`,
  hurt tint, 1.8/32 auto-scale; first-person view-model gated to mode 0
  (vanilla's body-replaces-hand behavior).
- **F2 screenshot** — request flag consumed INSIDE `render()` before
  present (readback of the live swapchain: pad-to-256 rows, BGRA→RGBA swap
  keyed on the surface format), PNG encoded in vc-render (owns `image`),
  saved to `screenshots/yyyy-MM-dd_HH-mm-ss-mmm.png` (millisecond tail:
  same-second captures distinct, lexicographic = chronological), toast +
  boot log. `COPY_SRC` added to the surface usage.
- **F3+G chunk borders** — `debug_chunks` flag → `render(chunk_borders)`
  draws 5×5 columns of unit-cube edge instances (yellow line pipe,
  pre-scaled 16×128 VB), player-chunk `div_euclid(16)`.
- **F11 fullscreen toggle** — was unbound (test-report gap); now toggles
  `settings.fullscreen` + `apply_fullscreen()` (same control as the Video
  Settings row — one control, one state).
- **B-5 Targeted Block → vanilla bottom-left** — extracted
  `f3_targeted_lines()` (block + id + state props + fluid); `debug_full` /
  `debug_canvas_full` painters pin the group above the F3 footer at
  `targeted_base_y(live_h, n)` (unit-tested at 720p/1080p/2160p); all three
  F3 call sites (live, F3_DUMP, F3_DUMP2) go through the *_full painters —
  the dumps show what the game paints.
- **E2E_FKEYS CI leg** — `linux-game.yml` run3: F1 HUD toggle/restore, F5
  full cycle → third-behind, F3+G on→off (chord via real F3 hold/release),
  F11 on→off, F2 arms capture; then a 3-stage PULL-based ladder (stage 0
  waits for world entry, 1/2 consume the PNG only after the readback
  lands) saves BOTH F5 views (`e2e_fkeys_behind_*.png` /
  `e2e_fkeys_front_*.png`, non-trivial size asserted) and exits 0/1.
  Headless xvfb quirks handled explicitly: no-WM Focused(false) auto-pause
  is held off while the ladder is pending, and the plain-smoke early exits
  (with and without F3_DUMP) defer to the ladder. CI greps
  "e2e: fkeys keys ok", "FKEY CONTRACT OK", both view-saved lines.
- **B-8 verified already-fixed** — wasm default RD is 6 in Rust
  (Settings::default cfg, game.rs:362); no JS wrapper downgrade exists
  (grep of public/voxelcraft.html + play.html + wasm_entry: no
  renderDistance override). Ledgered per the nothing-skipped policy.
- **B-4 re-verified bound** — `creative_picker: KeyCode::KeyB` (default)
  handled at key_action:3758; Help card accurate.
- **README test-count fix** — 818 → **861** (actual count from
  `cargo test --release --no-default-features --workspace`: 861 passed,
  0 failed, 2 ignored; full run green locally before this commit).
- New unit tests: F5 cycle contract, screenshot-stamp shape/ordering,
  targeted-group bottom-left pin, FKEYS stage machine. Plus the stray
  `title_layout_is_vanilla_stack` test re-attributed (dead-code warning).

**Honest status:** the E2E_FKEYS ladder was executed locally up to the keys
stage (all key assertions green, "e2e: fkeys keys ok"); the two-capture
finish + CI integration is verified by GitHub Actions on this push (no
heavy local builds per user directive).
