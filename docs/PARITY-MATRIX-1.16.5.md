# PARITY-MATRIX-1.16.5.md — the 1.16.5 parity audit matrix (Phase 2)

**Purpose:** the row-by-row parity audit the overhaul executes from. Built by
five parallel audit agents (2026-10-02) against: the live reference wiki
(the reference wiki), the owner-held study-only reference set (outside the repo,
never copied — used only for fact-level counts and aggregate statistics), and the engine's code +
tests. Status: DONE / PARTIAL / MISSING / WRONG.

**Reference-registry counts (measured from the jar today):** 859 recipe JSONs,
849 loot tables, 764 blockstates, 147 tags, the NBT structures (bastion,
end_city, fossil, igloo, nether_fossils, pillager_outpost, ruined_portal,
shipwreck, underwater_ruin, village). NOTE: the jar's tags/blocks/ carries NO
mineable/ subdirectory (mineable/axe.json etc. absent — generated at runtime,
not shipped in the client jar).

**Engine headline counts (measured from the code today):** 533 blocks
(STATE_COUNT 863), 50 MobKinds, 28 biomes, 863 tests. [UPDATE 2026-10-04: STATE_COUNT 892, 66 MobKinds, the visuals round landed — see the matrix rows.] [UPDATE 2026-10-06: test count re-measured — 911 `#[test]` attributes.]

---
## Domain 10 — UI and rendering

Engine counts: particles = 20 kinds (kinds.rs KINDS, 12 with live spawn sources, 8 registered-but-inert); sound registry = 122 entries (13 material families × 6 takes + 3 water takes + 41 one-offs incl. 7 music pads); ALL procedural recipes — no .ogg loading, no vanilla music/discs.

- GUI title screen | PARTIAL | ui.rs:763 + panorama.rs | title_layout_is_vanilla_stack, title_splash_is_yellow_and_tilted | w/Title screen
- GUI world select/create/edit | DONE | ui.rs:1695/1809/1946 | ui_screens_dump | w/Create New World
- GUI pause | DONE | ui.rs:1655 + game.rs enter_pause key-state reset | ui_screens_dump | w/Pause menu
- GUI death screen | DONE | ui.rs:1964 (hardcore + score) | ui_screens_dump | w/Death screen
- GUI inventory (survival) | DONE | ui.rs:5460 + container_screen:4160 (2x2 grid, armor/offhand) | survival_inventory_layout, container_hit_box_matches_drawn_slot_rect | w/Inventory
- GUI containers | PARTIAL | ui.rs:5999 ContainerKind — 14 kinds; smoker/blast/stonecutter/loom/smithing/cartography/lectern/dispenser/dropper GUIs missing; shulker box kind absent | hopper_screen_is_five_slots_in_one_short_row, anvil/beacon/grindstone/donkey geometry tests | w/Chest, w/Donkey
- GUI creative inventory | DONE | ui.rs:5460 + CreativeGeom:5918 (tabs/search/scrollbar/trash) | creative_screen_geometry, creative_screen_search_tab | w/Creative
- Options tree (main) | PARTIAL | ui.rs:824 — DEVIATION: MUSIC|SOUND and FOV|SENSITIVITY pairs sit on the MAIN screen (vanilla hosts them in Music&Sound/Controls); ENGINE SETTINGS disclosed extra | settings_layouts_stay_centered_at_any_live_width | w/Options
- Options — Video | PARTIAL | ui.rs:933 — all 1.16.5 rows present; Max Framerate + Mipmap live on Engine Settings instead; extras: Entity Distance (1.17+), SHADERS | settings_layouts_stay_centered_at_any_live_width | w/Options §Video Settings
- Options — Engine Settings | DONE (disclosed extra) | ui.rs:1096 | (indirect) | no vanilla equivalent
- Options — Shaders | DONE (disclosed extra) | ui.rs:1039 + game.rs Screen::Shaders | ui_screens_dump (indirect) | OptiFine/Iris-modded entry
- Options — Accessibility | PARTIAL | ui.rs:1260 — Distortion/FOV Effects GRAYED (no nausea); Show Subtitles is now WIRED to the subtitle renderer | settings_layouts_stay_centered_at_any_live_width | w/Options §Accessibility
- Options — Music & Sound | DONE | ui.rs:1514 — 10 category sliders | (indirect) | w/Options §Music & Sounds
- Options — Chat Settings | PARTIAL | ui.rs:1416 — most rows GRAYED (no chat subsystem) | reduced_debug_info_hides_detail_rows | w/Options §Chat Settings
- Options — Controls | PARTIAL | ui.rs:1565 — two-column keybind editor; engine's rebindable SUBSET vs vanilla's full list | (indirect) | w/Controls
- Options — Language | PARTIAL | ui.rs:1637 — English-only, honestly labeled | none | w/Language
- Options — Skin Customization | PARTIAL | ui.rs:1322 — layer toggles + Main Hand; no visible effect until layer meshes land | (indirect) | w/Options §Skin Customization
- HUD | DONE | ui.rs:3538 status_bars, 3941 hotbar, 3451 crosshair, 3908 attack_indicator, 3783 effect_icons, 3827 boss_bar, 3885 held_item_name, 3756 damage_vignette, 3742 underwater_tint | status_bars_quad_census_and_armor_gate, oxygen_row_draws_only_when_air_is_depleted, attack_indicator_fills_then_hides_at_full_charge, xp_bar_pushes_solid_quads_with_canvas_fallback | w/Heads-up_display + w/Food
- HUD geometry (hotbar) | DONE | ui.rs:3941 — 364x44 at scale 2 = vanilla 182x22; slot pitch 40px@2x = 20px | container_hit_box_matches_drawn_slot_rect | in-code SLOT_SRC(18) x GUI_SCALE(2)
- F3 content | DONE (core) | game.rs:21561 f3_lines (two columns) + 21790 f3_targeted_lines (Targeted Block/Fluid bottom-left) | f3_overlay_is_two_columns_with_per_line_strips, targeted_group_pins_bottom_left, reduced_debug_info_hides_detail_rows | w/Debug screen
- F3 key combinations | PARTIAL | game.rs:3879-4037 — F3, F3+Q/G/H/1 present; MISSING F3+A/B/T/C/D/N/P and the F3 pie | f3_help_overlay_is_centered_box | w/Debug screen
- Particles | PARTIAL | vc-particles/src/kinds.rs — 20 of ~90 registered types; ALL flat-color billboards; no per-type sprites, no additive blend split | particle_type_registry_matches_wiki_subset | w/Particles
- Sounds/music | PARTIAL | vc-audio/src/sounds.rs — 122 entries, ALL procedural; no .ogg loading, no vanilla music, no discs/jukebox | registry_parses_and_resolves, spatial_attenuation_and_pan | w/Sounds.json
- Models/animations | PARTIAL | game.rs entity stream (jointed 3D box rigs, villager sprites, 3D items, F5 model, shadows, destroy overlay); blockstate/model JSON for BLOCKS; animated atlas tiles | entity_distance_roundtrip_and_default; vc-pack blockstate tests | render.rs:249 budget comment
- Sky/fog/clouds | DONE | render.rs (sky gradient + sun/moon/stars, fog, clouds OFF/Fast/Fancy, Nether skyless), panorama.rs, 128px cloud atlas | panorama_painter_is_deterministic_and_oriented | w/Sky, w/Cloud, w/Fog
- Lighting/AO | DONE | render.rs ao_factor in terrain+water shaders; smooth 3-state riding MeshInputs.smooth through GPU+CPU; min-light floor; face_shade +Y 1.0/-Y 0.5/±Z 0.8/±X 0.6 (exact vanilla directional match, twin copies in sync; cross 0.85 engine choice). AO curve 0.42/0.62/0.80/1.0 internally consistent, public-curve source still open. Model shade:false compiled but unconsumed (T13, behavior-neutral — no builtin model uses it) | golden_glowstone_block_light, golden_terrain_patch_hash | w/Light, w/Smooth Lighting
- Resource-pack support | DONE (textures/models) / PARTIAL (sounds/fonts) | vc-pack pack.rs (Folder/Zip/Memory/PackStack), zip.rs, game.rs pack pipeline, ui.rs resource-pack manager, PNG font override | zip roundtrip tests, model parent-chain tests | w/Resource_pack §Behavior
- Shader-pack support | DONE (composite/final) / PARTIAL (gbuffers) | shaderpack.rs (scan, properties, PackOption, GLSL→WGSL via naga), iris.rs (IrisPassChain) | bsl_style_pass_translates_to_valid_wgsl, test_warm_pack_full_chain, uniform_reference_availability_is_honest | Iris docs (in-code VERIFIED)

### Domain 10 gaps
- Particles: ~70 of ~90 types missing — S each, batch M-L; per-type sprites, size/rotation animation, block collision, additive blend split — M-L.
- Sounds: ~580 of ~700 sound events missing — L; .ogg loading from packs — L; vanilla music + 13 discs + jukebox — L; subtitle overlay — M; narrator — S.
- GUI title: Realms grayed row missing — S.
- Options main: MUSIC|SOUND + FOV|SENSITIVITY pairs on the main screen vs vanilla hosting — S (needs a 1.16.5 period capture first).
- Video: Max Framerate + Mipmap on Engine Settings instead of Video — S.
- F3 combos: F3+A/B/T/C/D/N/P + the pie missing — M.
- Accessibility: Distortion/FOV not wired — S; subtitles overlay — M.
- Chat subsystem (the grayed rows) — L.
- Controls: full vanilla keybind list + conflict detection — M.
- Language: additional languages — L.
- Skin Customization: layer meshes (cape/elytra/jacket) — M; llama carpets — S.
- Containers: smoker/blast/stonecutter/loom/smithing/cartography/lectern/dispenser/dropper GUIs — M-L; shulker kind — S.
- Entity models: vanilla-format entity model/animation loading — L.
- Resource packs: sound files from packs — M; pack-side GUI-sheet override — S.
- Shader packs: gbuffers translator seam (NoTranslator) — L.
- Citation integrity: kinds.rs:4 cites docs/research/round-15b-particles-sky-audit.md which is ABSENT — S (restore or repoint).

## Domain 4 — Player mechanics

- Movement — walk 4.317 b/s | DONE | player.rs:8 WALK_SPEED | walk_speed_converges_to_vanilla | wiki /Transportation
- Movement — sprint 5.612 b/s | DONE | player.rs:9 SPRINT_SPEED | sprint_speed_converges_to_vanilla | wiki /Transportation
- Movement — sneak 1.3 b/s | DONE | SNEAK_SPEED 1.3 / SNEAK_DIAGONAL_SPEED 1.8 (VERIFIED w/Sneaking, live 2026-10-03: "the player moves slower, at a speed of about 1.3 m/s" + "two movement keys... diagonally at a faster 1.8 m/s"; the jump height/vertical speed are unaffected); sneaking still drives water sink/vine hang/slime-magma immunity/fly descent | the sneak caps in the walk-speed selection | wiki /Transportation
- Movement — swim 2.20/1.97 b/s | DONE | player.rs:29-30 + the exact per-tick travel() substep (1158-1171) | water_slows_and_buoys | wiki /Transportation
- Movement — sprint-swim 3.918 b/s | DONE | player.rs:31 SPRINT_SWIM_SPEED | water_slows_and_buoys | wiki /Transportation
- Movement — crawl | MISSING | none — no crawl mechanic, no 1-block-gap pose | none | wiki /Swimming §Crawling
- Movement — sprint-jump 7.127 avg | DONE | player.rs:36-53 | sprint_jump_averages_vanilla_7_127 | wiki /Transportation + mcpk /Sprinting
- Movement — gravity/drag v1=(v0-0.08)x0.98, terminal 78.4 | DONE | player.rs:12,24 | gravity_drag_matches_vanilla_formula, fall_accelerates_and_terminates | wiki /Transportation
- Movement — jump 1.25-block apex | DONE | player.rs:15 JUMP_VEL 8.4 | jump_apex_is_vanilla_height | wiki /Transportation
- Movement — creative/spectator flight | PARTIAL | player.rs:10-11 FLY_SPEED 10.9/FLY_SPRINT 21.8 vs wiki 11.0/22.0 — ~1% delta, no in-code citation | none | wiki /Transportation
- Movement — spectator max speed 43.556/87.111 | MISSING | spectator reuses FLY_SPEED; no spectator cap | none | wiki /Transportation
- Movement — elytra | PARTIAL | player.rs:134-137 — cap 25/descent 2.5 vs wiki 30.0 at 0° pitch; activates only as the SELECTED item (no chest slot); a documented 10:1 adaptation | elytra_glide_ratio, elytra_gate_requires_holding_jump_and_item | wiki /Transportation
- Hunger/saturation/exhaustion | DONE | hunger.rs (food 20/sat 5 spawn, exhaustion >4.0 drain, cap 40) | spawn_state_matches_the_wiki, exhaustion_drains_saturation_before_food | wiki /Food
- Exhaustion sources | DONE | player.rs update hooks + damage() + the hunger-drain tick | taking_damage_adds_exhaustion | wiki /Food §Energy-intensive
- Health/natural regen | DONE | hunger.rs (food>=18 → 1 HP/80t costs 6.0; full sat → 1 HP/10t) | natural_regen_every_80_ticks_costs_6_exhaustion | wiki /Food
- Starvation | DONE | hunger.rs StarveRule (Easy 10/Normal 1/Hard none) | starvation_follows_the_difficulty_thresholds | wiki /Food
- Sprint gate food > 6 | DONE | hunger.rs can_sprint + player.rs sprint_allowed | sprint_gate_is_food_seven_or_above | wiki /Food
- Combat — attack cooldown | DONE | combat.rs:27 (sword 1.6 → 12.5 ticks) | cooldown_ticks_match_vanilla | wiki /Attack_cooldown
- Combat — cooldown damage scale 0.2+0.8p2 | DONE | combat.rs:34 | cooldown_scale_endpoints | wiki /Attack_cooldown
- Combat — crits x1.5 | DONE | combat.rs:44 is_critical | critical_conditions, player_fist_full_pipeline | wiki /Critical_hit
- Combat — sweep | MISSING | no sword items exist (held_attack = fists) so sweeps never fire; no sweep-hit code | none | wiki /Attack §Sweeping
- Combat — shields | PARTIAL | player.rs:413 shield_up + game.rs full-absorption adaptation (no axe-disable/partial/arrow deflection) | shield_and_elytra_registered | wiki /Java_Edition_1.9
- Combat — knockback | PARTIAL | game.rs + mobs.rs adapted impulse, not the vanilla 0.4-attribute model | none | wiki /Knockback
- Combat — armor formula | DONE | combat.rs:56 armor_reduce + player.rs damage() (0-toughness simplification) | armor_formula_matches_vanilla_examples | wiki /Armor
- Fall damage | DONE | player.rs: dmg = fall-3, hay/honey x0.2, slime bounce, water/flight zeroing | fall_damage_is_distance_minus_three | wiki /Damage §Fall damage
- Drowning | DONE | player.rs: AIR_MAX 300 (15s), 2 HP/s at 0 air, +7.5/tick regen | air_depletes_drowns_and_regenerates | wiki /Damage §Drowning
- Fire damage | PARTIAL | soul fire 2 HP/0.5s + campfire/bush 1 HP/0.5s + magma 1 HP/s — the plain FIRE block has NO contact damage | soul_fire_contact_doubles_the_campfire_rate | wiki /Soul_Fire, /Fire (the gap)
- XP — orbs | DONE | entities.rs:360-376 ORB_VALUES ladder, despawn 6000t, attraction 7.25, 2-tick gate | phase_e1_split_xp_matches_the_vanilla_ladder | wiki /Experience
- XP — levels | DONE | player.rs add_xp/spend_levels + enchanting.rs:460 xp_to_next (2L+7/5L-38/9L-158) | xp_curve_matches_vanilla | wiki /Experience §Leveling
- Status effects | PARTIAL | effects.rs — 16 kinds: Wither, Poison, Regeneration, Speed, Haste, Resistance, Jump Boost, Strength, Slowness, Hunger, Absorption, Blindness, Water Breathing, Slow Falling, Conduit Power, Dolphin's Grace | wither_ticks_damage_every_second_and_can_kill, poison_cannot_kill | wiki /Effect — 16 of 32 missing
- Game modes | DONE | modes.rs — Survival, Creative, Hardcore, Adventure, Spectator (loads GameType 3) | vanilla_round_trip, spectator_mode_rules | wiki /Adventure

## Domain 6 — Redstone and technical

- Tick backbone/update order | DONE | redstone.rs:26 REDSTONE_TICK_RATE=2 + the notification cascade | redstone_is_deterministic, torch_clock_oscillates | wiki /Redstone_tick
- Wire | DONE | redstone.rs wire_tick — decay 1/block, max 15 | lever_powers_wire_chain_with_decay | wiki /Redstone_Dust
- Redstone torch | DONE | torch_tick — inverts support, clock oscillation | torch_inverts_its_support_signal | wiki /Redstone_Torch
- Redstone block | DONE | direct_feed/power_at | phase_e3_redstone_block_powers_adjacent_wire | wiki /Block_of_Redstone
- Lever | DONE | toggle_lever | lever_powers_wire_chain_with_decay | wiki /Lever
- Repeater (1-4 delay, lock) | DONE | repeater_tick — own 1..4 rt delay, side-lock | repeater_passes_and_delays_signal, repeater_lock_holds_output | wiki /Redstone_Repeater
- Comparator | DONE | comparator_tick — compare/subtract, 2 gt, container 1+14*fill | comparator_modes_match_wiki_formulas | wiki /Redstone_Comparator
- Piston (push limit 12) | DONE | redstone.rs:795 PISTON_PUSH_LIMIT=12, unpushable table | piston_push_limit_is_twelve | wiki /Piston
- Sticky piston/glazed | DONE | retract branch — sticky pulls one, glazed push-never-pull | v112_glazed_pushable_but_not_pullable | wiki /Glazed_Terracotta
- Piston animation | PARTIAL | instant commit on the 2gt tick; no moving-block entity | none | wiki /Piston §Mechanics
- Observer | DONE | observer_pulse — strength 15 for 2 GAME ticks from the BACK | observer_pulses_on_neighbor_change, observer_output_comes_from_the_back | wiki /Observer (Java)
- Dispenser | PARTIAL | redstone.rs:1248 — rising-edge + QC + 4gt; the eject is delegated; no projectile behaviors | phase3_constants_match_the_wiki | wiki /Dispenser
- Dropper | PARTIAL | shared dispenser path — edge + delay only | none | wiki /Dropper
- Hopper | DONE | hopper_enabled + hopper_pass — one item per 8 gt (2.5/s) | phase3_constants_match_the_wiki | wiki /Hopper
- Rails/minecarts | MISSING | none — no RAIL/MINECART ids or entities | none | wiki /Rail, /Minecart
- TNT | DONE | vc-blocks TNT 533/state 863 + faces 794..796 (hardness 0, blast resistance 0 — VERIFIED w/TNT); vc-sim PrimedTntSystem (fuse 80gt, gravity 0.04/drag 0.98, hitbox 0.98, flash alternating every 0.5 s, explosion 4.0 at fuse 0 queued 0.06125 above the rested position); redstone + fire/lava contact ignition via ignition_sweep; chain-priming at the 10-30gt random fuse | tnt_fuse_is_80_ticks_and_explodes_at_zero, tnt_chain_fuse_rolls_between_10_and_30, tnt_flash_phase_alternates_every_10_ticks | wiki /TNT
- Explosions (terrain) | DONE | game.rs explode() — the VERIFIED exposure-based damage/knockback (the bounding-box sample grid + the eye-vector impulse) replaces the old self-declared placeholder; 1/power drop chance; difficulty-scaled player damage; chain-priming ignites the TNT in radius | explosion_exposure + the vc-sim TNT tests | wiki /TNT, /Explosion
- Slime/honey (redstone) | PARTIAL | movement physics only (HONEY_SLOW_FACTOR 0.58); no sticking group/flying machines | slime_bounce_and_sneak_damage | wiki /Slime_Block, /Honey_Block
- Quasi-connectivity | DONE | redstone.rs:956 qc_powered + QC shadow scheduling | quasi_connectivity_powers_the_piston | wiki /Redstone_mechanics
- Buttons | MISSING | none — no button block id | none | wiki /Button
- Pressure plates | PARTIAL | redstone.rs:501 — LIGHT/HEAVY weighted only; wooden/stone absent | phase_e3_weighted_plate_formulas | wiki /Pressure_Plate
- Tripwire | MISSING | TRIPWIRE_HOOK exists but no string/crossing/pulse logic | none | wiki /Tripwire
- Daylight sensor/trapped chest/target/anchor signals | DONE | redstone.rs phase E3 | phase_e3_daylight_sensor_feeds_wires_by_sky_light, target_pulse_feeds_wire_and_decays | wiki /Daylight_Detector, /Target
- Redstone lamp | PARTIAL | lamp_tick — OFF approximates the vanilla 4gt delay | lamp_lights_and_turns_off_with_the_lever | wiki /Redstone_Lamp

## Domain 7 — Fluids

- Water flow rules | DONE | fluids.rs:27 WATER_TICK_RATE=5, levels 1..7, down-first, re-derive | source_spreads_seven_blocks_and_stops | wiki /Water §Fluid behavior
- Lava flow rules | DONE | fluids.rs:202-222 — 30/10 gt, drop-off 2/1, no sources | lava_rates_and_spread_by_dimension | wiki /Lava
- Falling water (level-8 block) | PARTIAL | fluids.rs doc delta — level-1 flow carries it; the mesher renders full height | water_falls_into_a_hole | wiki /Water §Falling
- Source conversion (infinite water) | DONE | fluids.rs:218 — a flowing cell whose neighbours are all full converts to a permanent level-0 source, and sources (level 0) feed their neighbours | fluids_source_* | wiki /Water
- Lava/water interaction | DONE | obsidian / cobblestone / stone products implemented; the fluids.rs:277 note records that no *infinite-source pairing* exists for lava, which is the vanilla behaviour, not a gap | fluids_lava_water_* | wiki /Lava §Water and lava
- Waterlogging | DONE | a waterloggable block holds its own block state and the cell reads as a level-0 source (fluids.rs:38) | fluids_waterlog_* | wiki /Waterlogging
- Bubble columns | DONE | SOUL_SAND seeds the column; the column and its whirlpool behaviour are implemented | fluids_bubble_* | wiki /Bubble_Column
- Water-plant interactions | PARTIAL | water extinguishes a LIT campfire; does NOT break cross plants (documented delta) | none | wiki /Water
- Concrete powder solidification | DONE | fluids.rs gravity_tick + solidify_powder — all 16 colors | v112_all_powder_colors_solidify | wiki /Concrete_Powder

### Domains 4/6/7 gaps (size S/M/L)
- Sneak-walk slowdown missing (S); crawl missing (S/M); plain FIRE damage missing (S)
- Sweep dead-lettered until swords land (S); fly 10.9/21.8 vs 11.0/22.0 — re-verify before touching (S); elytra cap 25 vs 30.0 — re-derivation needed (S/M); spectator speeds + /gamemode missing (S/M)
- 16 of 32 effects missing: Fire Resistance, Invisibility, Night Vision, Weakness, Mining Fatigue, Nausea, Instant Health/Damage, Health Boost, Saturation, Glowing, Levitation, Luck, Unluck, Bad Omen, Hero of the Village (L)
- TNT block + primed entity + chain-priming missing (M); explosion damage curve is a self-declared placeholder (M)
- Rails/minecarts absent (L); buttons absent (S); wooden/stone plates absent (S); tripwire logic absent (M)
- Infinite water source DONE (the fluids round): a flowing block horizontally adjacent to 2+ sources + support, or one horizontal + one above — the water tick converts it (VERIFIED w/Water §Source blocks, live 2026-10-03 raw wikitext; the water_source_conversion gamerule is future work) | the fluids round tests | wiki /Water
- Lava/water products DONE: lava source + water = obsidian; flowing lava + water = cobblestone (no sources removed); lava downward onto water = stone (VERIFIED w/Fluid §Mixing + w/Water §Water and lava, live 2026-10-03 — the two sections AGREE, the cross-check) | the fluids round tests | wiki /Lava §Water and lava
- Waterlogging DONE (the chest pair): the dedicated states 874/875 (a waterlogged cell carries a full source in the same block space, VERIFIED w/Waterlogging); flowing water enters a dry container and waterlogs it; the mesher renders BOTH the container and the water overlay (CPU + GPU parity, the F_WATER flag class); slab/stairs/fence waterlogging is a disclosed trim (the JSON-model dispatch) | the fluids round tests | wiki /Waterlogging
- Bubble columns DONE: the BUBBLE_COLUMN block (538) — soul sand under source water = upward, magma = whirlpool; created 20gt after placing, destroyed 5gt after destroying (all column blocks simultaneously); propagates only through source water; the verified transport (11 b/s up / 4.9 b/s down JE) drives the player drag; air-providing (the drowning meter refills) (VERIFIED w/Bubble_column, live 2026-09-25) | the fluids round tests + the player wiring | wiki /Bubble_Column
- Audit-input correction: observers are 2 GAME ticks in 1.16.5 Java (the engine matches); the jar's tags/blocks/ has NO mineable/ subdirectory (the tool-tier domain needs another source)

## Domain 3 — Mobs (66 MobKinds registered; every 1.16.5 mob id is PRESENT in the roster — registry + model + basic AI. Behavioural depth varies per mob: see the PARTIAL list. Updated 2026-10-06; the old “17 MISSING” row contradicted the gap list in this same file.)

Reference counts: jar loot_tables/entities = 72 per-mob JSONs (+16 sheep variants); the 1.16.5 roster ends at 1.16.2's piglin brutes. Engine: 66 MobKinds (65 MOB_DATA rows; Squid is a classification-only stub) — THE 1.16.5 SET COMPLETE (the mobs batch 2026-10-04: the 16 missing kinds + the verified stats/tiles/drops/AI + the slime-chunk spawn rule), 12 entity-model rigs.

DONE mobs: zombie, skeleton, creeper, spider, cow, pig, chicken, magma_cube, blaze, ocelot, zombie_villager, horse, donkey, mule, rabbit, stray, husk, illusioner, vex, drowned, phantom, dolphin, cod, salmon, pufferfish, tropical_fish, turtle, fox, bee, cave_spider, silverfish, zombified_piglin, villager, wither, ender_dragon.
PARTIAL: enderman (no teleport-on-damage/water), sheep (no wool colors/dye/shear), snow_golem (no trail), iron_golem (drops IRON_BLOCK 1 vs 3-5 ingots), mooshroom (no shear/stew), wither_skeleton (2.5% skull drop MISSING — stale comment game.rs:8004), witch (NO ATTACK ARM — hostile with damage 6 but no ai_tick case), polar_bear (no cub-defense), llama (no carpet), parrot (no perch/mimicry), evoker (fangs = particle adaptation), strider (no riding), piglin (no gold-armor pacification; barter trimmed), hoglin (no zoglin), ghast (no explosion radius), squid (stub, never spawns).
REGISTERED, behaviour varies (updated 2026-10-06): all 16 below exist in the roster in mobs.rs (cat, wolf, slime, panda, guardian, elder_guardian, endermite, shulker, pillager, ravager, wandering_trader, trader_llama, piglin_brute, zoglin, skeleton_horse, zombie_horse). “Registered” means the kind resolves, has stats, a model and spawn rules — NOT that its full vanilla AI is reproduced; check the PARTIAL list and the per-mob tests before claiming parity.

Aspect rows:
- MOB_DATA stats | DONE | 49 rows, Normal-difficulty damage, equine per-instance randomization | mob_table_matches_verified_wiki_rows | wiki infoboxes
- AI goals | PARTIAL | straight-line steering + 1-block step-ups; ~40 per-kind arms; no vanilla Goal system/A* | per-kind suites | wiki Behavior sections
- Spawn rules | DONE | try_spawn_* — light <=7, passives >=9, caps 70/10, despawn 128 + 32/30s 1/800, exact 1.16.5 Nether weights, phantom insomnia 72000 | v113_water_ambient_biome_families | wiki Mob/Spawning
- Drops vs loot tables | PARTIAL | drain_mob_events (HARDCODED per-kind match; not datapack-driven) | creeper_fuses_then_explodes | jar loot_tables/entities
- Sounds | PARTIAL | ~20 procedural families; most mobs reuse generic hurt/death | none | wiki sound events
- Models | PARTIAL | 12 jointed rigs; 48 kinds are billboard sprites | entity_distance_gate_culls_far_mobs | wiki entity models

## Domain 9 — Systems

- Weather (rain/thunder/lightning) | DONE | weather.rs + weather_update + lightning_strike | clear_first_and_ranges, thunder_needs_rain, sky_factors_match_the_wiki | w/Weather: rain 12000-24000/12000-180000, thunder 3600-15600, 30s flash, lightning 5 HP
- Sleep weather reset | DONE | weather.rs sleep_reset() wired by the beds round's sleep flow (rain/snow → Clear on the skip; VERIFIED w/Weather + w/Bed §Passing the night: the Java cycle reset is the rain/snow row) | sleep_resets_flags_not_timers + the E2E_BEDS night-skip leg | w/Weather
- Day/night/moon | DONE | DAY_LEN 1200s = 24000 ticks; moon = day % 8 | day_cycle_is_the_vanilla_20_minutes | w/Daylight_cycle
- Fire spread | DONE | the fire tick (bde9c25 + the fixes): the fire-age window 877..891 (FIRE 805 = age 0, the state-space persistence), the age increment at the deterministic random-tick cadence (1365 ticks = the 4096/3 mean), the extinguish rules (water contact, rain 20%+3%/age, age>3 + nothing flammable/no support, the 1/4 at 15), the burn-away at burn_odds/300 (no drops; TNT ignites via the sweep), the spread (the ignition degree (i+7d+40)/(a+30) against the 100/200/300/400 base, the rain-blocked spread, the jungle/swamp halving) + the flammability table (the wiki's verified ignite/burn odds for the engine's blocks) + the lightning ignition (pre-existing) | the fire round tests | w/Fire
- Plain fire damage | DONE | inside a fire block 1 HP per 10 sim ticks (soul fire 2 — VERIFIED w/Fire + w/Soul_fire), the after-burn floor 160 (the Fire tag, the -20 grace), the outside burn 1 HP per second (1 HP per 20 sim ticks), the water/rain extinguish (the tag resets to -20); the fire-block detection in the player update | the fire round tests | w/Fire
- Farming/random ticks | DONE (behaviors) / PARTIAL (density) | random_plant_tick + grow_crop + RandomTicker | farm_growth_denominators_match_the_wiki_table | w/Farmland — engine samples 3/chunk vs vanilla 3/section (~1/16 density); sugarcane/cactus/cocoa absent
- Raids | MISSING | deferral note only | none | w/Raid
- Trading | DONE (partial scope) | villagers.rs — 15 professions, 5 tiers, restock 2x/day, gossip + reputation | trade_tables_cover_all_professions, gossip_table_matches_wiki | w/Villager, w/Trading
- Piglin bartering | PARTIAL | try_barter_piglin — trimmed table 158/469 weights; gold = IRON_ORE stand-in | v116b_barter_table_items | w/Bartering
- Advancements | MISSING | detected and honestly reported unsupported | datapack_demo_e2e_claims_hold | jar: 80 non-recipe advancements
- Commands | MISSING | no slash-command parser; the command block renders but has no execution bridge | none | w/Commands
- Datapacks/loot/tags | DONE (modeled subset) | datapack.rs — shaped/shapeless, loot pools/rolls/weights, tags merge/replace | tag_merge_and_replace_semantics, recipe_grammar_matches_the_vanilla_jar | jar 849/147/927
- Scoreboard | MISSING | none | none | w/Scoreboard
- Gamerules | DONE | the Gamerules system is live and applied by the tick loop | gamerules_* | w/Game rule
- Difficulty | DONE (partial) | combat.rs difficulty_scale + local_difficulty (0.75-1.5 ramp) | difficulty_scaling_matches_wiki_rows | w/Difficulty
- Fishing loot | DONE (partial rows) | fishing.rs — 85/10/5, Lure -5s/level, 4/7/10 rows | base_roll_matches_the_wiki_percentages | w/Fishing
- Mob-kill XP | DONE | MOB_DATA xp + XpOrbSystem | phase_e1_split_xp_matches_the_vanilla_ladder | w/Experience

### Domains 3/9 gaps (S/M/L)
- 17 missing mobs (L); raids (L); advancements (L); slash commands + command-block bridge (L); A* pathfinding (L); 3D multi-part models for the 48 billboard kinds (L)
- Mob drops not datapack-driven + the Looting enchant unwired (M); random-tick density + missing behaviors (M); gamerules (M); scoreboard (M); piglin pacification (M); per-mob sounds (M); strider riding (M); sheep wool variants (M). (The witch attack + the wither-skeleton skull 2.5% + the fire spread + the beds/sleeping are DONE — the witch/skull, fire, and beds rounds.)
- Wither-skeleton skull stale comment (S); fixed-min-1 drops vs jar uniform-0 rows (S); enderman teleport (S); polar bear cub-defense (S); turtle easter egg (S); bat cap 10 vs 15 (S); evoker fangs (S); enchanted-gear drops (S); stale DEFERRED_ENTITIES comment mobs.rs:1425 (S)

## Domain 1 — Blocks and items (audited directly)

- Block count | PARTIAL | 533 blocks (blocks.rs, STATE_COUNT 863) vs vanilla 1.16.5's ~846 blocks (764 blockstate JSONs in the jar) — ~313 missing (the long tail: the 1.16 sets are present, the pre-1.13 uncommons are trimmed) | STATE_COUNT const | jar blockstates + wiki
- State system (properties/variants) | DONE | property-driven states (blocks.rs:3895+, the vanilla assignment algorithm) + JSON blockstate dispatch (model.rs) | blockstate_variants_and_fallback, wgsl_lut_offsets_match_rust | jar blockstates grammar
- Hardness/blast/tool tiers | PARTIAL | hardness + blast tables in blocks.rs; tool tiers simplified (no mineable/ tag system — the jar ships no mineable/ dir either) | various | blocks.rs tables
- Drops (Fortune/Silk Touch) | PARTIAL | hardcoded per-kind drops in game.rs drain_mob_events; block drops modeled; Fortune exists in enchanting.rs but unwired; Silk Touch absent | various | jar loot tables
- Light emission/opacity | DONE | light ladders in blocks.rs (emissive tables; opacity per block) | golden_glowstone_block_light | blocks.rs
- Block sounds | PARTIAL | 13 material families (sounds.rs) vs vanilla's per-block sound groups | registry_parses_and_resolves | wiki Sounds.json
- Creative tabs | PARTIAL | CreativeTab enum in blocks.rs (BuildingBlocks, Redstone, ...) vs vanilla's 12 tabs | various | wiki Creative

## Domain 5 — Items and crafting (audited directly)

- Recipe set | DONE (in scope) / PARTIAL (breadth) | 1120 recipe entries in craft.rs vs the jar's 859 recipe JSONs — the engine carries MORE entries (variants + the backlog fire); shaped/shapeless grammar matches the jar | recipe_grammar_matches_the_vanilla_jar, stand_recipe_needs_the_exact_layout | the jar recipes dir (859)
- Furnace/blast/smoker | DONE | furnace.rs — COOK_TICKS 200, blast/smoker 100 (COOK_TICKS/2), the class split (furnace rejects food, smoker rejects ore) | phase_e1 tests + v114b_smelter_lantern_recipes | wiki /Blast_Furnace, /Smoker
- Campfire cooking | DONE | campfire.rs — no fuel, 4 slots, 30s | campfire tests | wiki /Campfire
- Stonecutter | MISSING | deferral note blocks.rs:1612 | none | wiki /Stonecutter
- Smithing table | MISSING | none | none | wiki /Smithing_Table
- Recipe book | MISSING | none | none | wiki /Recipe_book
- Tools/armor/enchantments | DONE (partial) | enchanting.rs — the 1.16.5 enchantment set (38-set revision cited); armor exists (armor_reduce) | armor_formula_matches_vanilla_examples | wiki /Enchanting
- Tipped/lingering potions | MISSING | brewing has 9 potion families; no tipped/lingering variants | none | wiki /Tipped_Arrow
- Fireworks | MISSING | a particle kind only (inert); no firework item/entity | none | wiki /Firework_Rocket
- Maps | MISSING | no map item | none | wiki /Map
- Books | MISSING | no book-and-quill/lectern | none | wiki /Book_and_Quill

## Domain 2 — World generation (audited directly)

- Biomes | PARTIAL | 28 variants in gen.rs (with live-cited vanilla registry ids) vs vanilla 1.16.5's 66 — the deep-variant folds are disclosed (deep oceans fold into families); MISSING ~38: the mountains/plateau splits, the 1.16 forest variants beyond crimson/warped, badlands variants, river, snowy variants, stony shore, eroded badlands, bamboo jungle, sunflower/flower-plains exist, modified-edge biomes | biome ids cited in-code | wiki /Biome (66)
- Noise stack | DONE (partial) | vanilla_noise.rs — the Perlin 2002 formulation (accepted clean-room); NOT vanilla's full multi-noise 3D climate sampler (disclosed) | vanilla_noise tests | wiki /Noise_generator
- Caves/ravines | PARTIAL | carve caves exist (the gen's carve pass); ravines? — check at implementation | golden_terrain_patch_hash | wiki /Cave
- Surface rules | PARTIAL | per-biome surfaces (grass/dirt/sand/mycelium/basalt) — NOT vanilla's surface-rule system (the JSON-based rules) | various | wiki /Surface_rule
- Ores | DONE (7 ores) | place_ores — coal 20/0-127, iron 20/0-63, gold 2/0-31, redstone 8/0-15, diamond 1/0-15, lapis 1/8-24, emerald (mountains) | place_ores tests | wiki /Ore — vanilla 1.16.5 has no copper (1.17) ✓ correct set
- Trees | DONE (partial) | oak/birch/spruce/acacia/dark-oak + the jungle? — check | various | wiki /Tree
- Structures | PARTIAL | 8 of ~16: village (grid+houses), dungeon, mineshaft, pyramid (desert), jungle temple, stronghold, fortress, woodland mansion, icebergs (frozen ocean) | structure tests (emit_*) | wiki /Structure; MISSING: bastion, end_city, igloo, pillager_outpost, ruined_portal, shipwreck, underwater_ruin, nether_fossil, fossil, ocean monument, swamp hut, witch hut
- Seed determinism | DONE | the golden-seed tests + Phase-6 world-hash gates | golden_terrain_patch_hash, deterministic_random_sampling | the determinism tests

## Domain 8 — Dimensions (audited directly)

- Overworld/Nether/End rules | DONE | travel_to_dimension + dimension-specific spawns (find_spawn/find_nether_spawn/end_arrival) | various | wiki /Dimension
- Nether 8:1 scaling | DONE | "8:1 horizontal mapping (vanilla nether portals)" (game.rs:20513) | various | wiki /Nether_portal
- Nether portal BLOCK | DONE | vc-blocks NETHER_PORTAL 536/state 872 + tile 801 (the clean-room vortex animation, the mesh census pins 3 animated strips); vc-gameplay portal.rs — the obsidian-frame validation (2x3..21x21 interior, corners not required), the flint-and-steel ignition (FIRE placement path), the walk-in trigger (80gt survival/1gt creative), the 128/16 search radii, the build-spot scan + forced-Y clamp; the flint-and-steel item joins the Tools tab | valid_frame_minimum_2x3_interior, portal_search_finds_closest_within_radius, walk_in_waits_80_game_ticks_survival | wiki /Nether_portal (Round K)
- End platform/exit | DONE | end_arrival (the 5x5 obsidian platform, x=100 z=0) + the exit portal | various | wiki /The_End
- Dragon fight | DONE | dragon.rs (200 HP, 12000/500 XP, crystal heal 1/10t in 32-block cuboid) | dragon_health_and_fight_constants_match_the_wiki | wiki /Ender_Dragon
- Wither | DONE | wither.rs (300 HP, nether star 100%, charge 220 gt) | constants_match_the_live_wiki | wiki /Wither

## Domain 11 — Persistence (audited directly)

- Region/Anvil format | DONE (partial) | anvil.rs — .mca reader+writer, 32x32 chunks, 4096-byte sectors, corrupt inputs never panic | anvil tests | wiki /Region_file_format — byte-compat with REAL 1.16.5 regions needs the 3-H round (the owner runs the reference game)
- level.dat | DONE (partial) | save.rs — SpawnX/Y/Z + the core keys | save tests | wiki /Level_format — the full 1.16.5 key set needs the audit
- Save upgrade | DONE | the save-format versioning/migration tests | migration tests | wiki /Level_format
- Autosave | DONE | 20s cadence + save-on-quit | autosave tests | in-code VERIFIED

---
## Prioritized gap list (player visibility first, then dependencies)

**Tier 1 — player-visible, blocks core gameplay (M):**
1. ~~Nether portal block + walk-in trigger (Round K)~~ DONE — commit 2dd2cd8 + the fix loop (the search's folded-block-id catch); all gates green, E2E green.
2. ~~TNT + primed entity + chain-priming; the explosion damage formula~~ DONE — the exposure-based damage/knockback is VERIFIED (the placeholder is gone); the bottom-anchored probe + the flash-tick fold landed.
3. ~~Beds/sleeping (night skip, spawn point, the weather sleep_reset wiring)~~ DONE — sleep.rs + the E2E_BEDS leg; the phantom reset stays with the mobs batch (no phantom MobKind yet).
4. ~~Waterlogging + bubble columns; infinite water source; lava+water products (obsidian/cobble/stone)~~ DONE — the fluids round (564ab2c + fb1f6f0 + 9af843f + the GPU parity + the player wiring): the waterlogging states (the chest pair 874/875, the overlay renders both the block and the water), the BUBBLE_COLUMN block (538, the 20gt create / 5gt destroy, the verified 11/4.9 b/s transport + the air-provide), the infinite water source (2+ adjacent sources + support, or one horizontal + one above), the mixing products (lava source + water = obsidian, flowing lava + water = cobble, lava downward onto water = stone) — all VERIFIED live (Water + Fluid + Bubble_column, raw wikitext 2026-09-25/2026-10-03) and CROSS-CHECKED (the Water §Water and lava section vs the Fluid §Mixing rules agree; the WATER arm now culls against the column — the column IS water). Slab/stairs/fence waterlogging is a disclosed trim (the JSON-model dispatch).
5. Fire spread + flint & steel ignition; plain FIRE block damage.
6. ~~Sneak-walk slowdown 1.3 b/s; plain fire damage; the 16 missing status effects~~ DONE — the fire/sneak round (bde9c25 + 99ce7ca + 44118d6): the 1.16.5 effect set is COMPLETE (32 of 32) with the verified behaviors wired, the sneak caps verified, the fire round landed. The luck/bad-luck loot modifier + the Bad Omen raid trigger + the Hero trade discount are disclosed future hooks (the systems/mobs rounds).
7. ~~Witch attack (hostile, no attack arm); wither-skeleton skull 2.5% drop (stale comment)~~ DONE — the witch/skull round (ea50aa5 + 26e128a): the VERIFIED splash-potion ladder (Slowness I 1800t at 8-10 blocks, Poison I 900t at health ≥ 8, the 25% Weakness 1800t at < 3 blocks + health ≤ 8 or poisoned, default Harming 6 HP magical), the 3 s interval, the pursue-within-16, the anchored player's health for the gates; the skull drop dropchance 0.025 landed. The witch's drinkable-potion defense + the splash AoE are disclosed trims.

**Tier 2 — content breadth (M-L):**
8. The 17 missing mobs (cat, wolf, slime, panda, guardian/elder, endermite, shulker, pillager, ravager, wandering trader, trader_llama, piglin_brute, zoglin, skeleton/zombie horse) — batched by family.
9. Raids (Bad Omen, waves, bell); advancements (80); slash commands + command-block bridge; scoreboard; gamerules.
10. Structures: bastion, end_city, igloo, pillager_outpost, ruined_portal, shipwreck, underwater_ruin, ocean monument, swamp hut, witch hut.
11. Particles ~70 more types + per-type sprites + the additive split; sounds ~580 more events + .ogg loading + music discs/jukebox.
12. Smoker/blast/stonecutter/loom/smithing/cartography/lectern GUIs; shulker container.
13. Biome breadth (28/66): the missing splits/variants + the 1.17-correct set (no copper needed). RESOLVED during the audit: the save-side biome-id mapping was ground-truthed against the REAL 1.16.4 saves' Biomes IntArrays + the pre-1.13 id table — Sunflower Plains now saves as Java id 129 (was 130 = Desert M, FIXED commit b4a505c); Flower Forest 132 and Ice Spikes 140 verified correct; the nether ids 170-173 match the 1.16.5 registry.
14. Blocks breadth (533/846): the long-tail sets (batches by update bracket).

**Tier 3 — polish/feel (S-M):**
15. F3 combos (F3+A/B/T/C/D/N/P + the pie); subtitle overlay; accessibility wiring (Distortion/FOV).
16. Fly/elytra constants: re-verify 10.9/21.8 vs 11.0/22.0 and the elytra 30.0 cap with live citations.
17. Mob drops driven by the datapack loot tables (the Looting enchant wired); fixed-min-1 vs jar uniform-0 rows.
18. Realms grayed row; Max Framerate/Mipmap on Video; skin-layer meshes; llama carpets; piglin gold-armor pacification; strider riding.

**Dependencies:** Tier 1 items are independent of each other; the TNT formula verification needs the wiki first; the beds item unblocks the phantom/sleep_reset wiring; Tier 2's mobs batch needs the entity-model loading path (L) to be useful for player-visible rendering; the GUI items need the container-kind scaffolding.

**Round-order proposal:** Round K (nether portal) → TNT → beds → fluids (waterlogging/bubble/obsidian) → fire → sneak/effects → witch/skull fix → mobs batch (by family) → systems (raids/advancements/commands/scoreboard/gamerules) → structures batch → particles/sounds batch → GUI batch → biome/blocks breadth → polish tier.
