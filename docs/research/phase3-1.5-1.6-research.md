# Phase E3 research record — MC 1.5–1.6 bracket (live round, 2026-09-06)

The strict protocol's per-bracket research record: what was checked,
against which live page, and every disagreement/adaptation. Raw search
transcripts: `voxelcraft/scripts/verify_e3_*.json`; raw page fetches:
`voxelcraft/scripts/e3_page_*.json`.

## Verified live (minecraft.wiki unless noted)

| Value | Page | Note |
|---|---|---|
| Block of coal 800 s / 16000 ticks / 80 items (10× coal) | w/Block_of_Coal, w/Smelting | 3-source agreement |
| Block of quartz = 4 nether quartz | w/Block_of_Quartz | |
| Quartz pillar: 2 blocks of quartz → 2 pillars | w/Quartz_Pillar + a 2nd search source | count column unreadable in extract — **flagged lightly-sourced** |
| Chiseled quartz from quartz slabs | w/Chiseled_Quartz_Block | no slab system → deferred |
| Quartz ore drops 1 quartz (Fortune to 4), 2–5 XP | w/Nether_Quartz_Ore | |
| Carpets: 2 wool → 3 (13w17a); hitbox 1/16 (14w29a) | w/Carpet | |
| Terracotta 16 dye colors, "abundant in badlands" | w/Terracotta, w/Badlands | band table unpublished → clean-room sequence |
| Hay bale: fall damage −80% (take 20%); feeds horses; foal growth +3 min | w/Hay_Bale | |
| Daylight sensor recipe (glass + quartz + any wooden slab) | w/Daylight_Detector | |
| Daylight signal factors (time/weather/sky exposure/internal sky light) | w/Daylight_Detector | engine maps sky-light × day-phase (no weather — disclosed) |
| Trapped chest: tripwire hook + chest; signal = viewers (max 15) | w/Trapped_Chest | single-player = 1 while open |
| Light plate: signal = entity count, 1..15, type-independent | w/Light_Weighted_Pressure_Plate | |
| Heavy plate: ceil(entities/10), max 15 | w/Heavy_Weighted_Pressure_Plate | |
| Block of redstone: 9 dust; permanently powered, weak 15 to direct neighbors | w/Block_of_Redstone | wire = the engine's dust (disclosed) |
| Superflat classic: grass + 2 dirt + bedrock; plains | w/Superflat + tutorial source | JE structures out of engine scope |
| Horse: health 15–30 (avg 22.5); speed 0.1125–0.3375 (×43.17 ≈ 4.86–14.57 b/s, avg 9.71); jump 0.4–1.0 (1.153 / 3.124 / 5.9197 blocks) | w/Horse §Health/§Movement_speed/§Jump_strength | |
| Horse taming: temper 0/100, threshold 0–99 at first mount, +5 per failed mount | w/Horse §Taming (+ Mule page agreeing) | |
| Horse breeding: golden apple/carrot, 2 tamed adults; 5-step bred formula; 1–7 XP | w/Horse §Breeding/§Bred_values | |
| Horse spawning: plains 5/46 (10.87%), savanna 1/52, herds 2–6, 20% babies | w/Horse §Spawning | engine roll-share adaptation |
| Horse drops: 0–2 leather + equipped saddle/armor; 1–3 XP | w/Horse §Drops (search) | |
| Donkey: 15–30 HP (avg 22–23), speed 0.175 spawned | w/Donkey | |
| Mule: horse×donkey; 15–30 HP tending 22–23 | w/Mule | |
| Saddle = control requirement | w/Horse §Riding, w/Riding | |
| Lead: 1.16.5 stretch max **10** blocks | w/Lead + Fandom + history search | **version-scoping catch**: current wiki says 12 — that is the 2025 "Chase the Skies" buff ("Leash snapping distance has been increased to 12 blocks"); both cited in code |
| Lead on fence = knot; stays within 5 of post; breaks/drops | w/Lead | |
| Name tag needs anvil rename | w/Name_Tag | deferred (no anvil GUI rename) |

## Disagreements / version traps caught

1. **Lead length** — the current wiki's 12 blocks is post-1.21.5
   content; 1.16.5 is 10. Exactly the dripstone-style contamination
   the protocol guards against.
2. **Quartz pillar output count** — recipe table count column not
   text-extractable; second source says 2. Implemented as 2, flagged.
3. **Daylight sensor formula** — Java's exact internal-sky-light
   curve is not published in the text layer; engine maps its real
   sky light through the day-phase brightness curve (disclosed).

## Adaptations (disclosed in code)

- Wire-as-dust for the block-of-redstone recipe (the engine's redstone
  "item" is the wire block).
- Ore-for-ingot plate recipes (gold/iron ore — the E2 convention).
- Badlands banding: deterministic color sequence by (y + seed offset).
- Carpets: 5 colors (the engine wool palette), non-solid full-tile
  render at the 1/16 look.
- Rider fall immunity while mounted (vanilla gives the rider +7 safe
  blocks; the mount's landing carries the damage here).
- Hay temper gain +10 (the wiki temper table covers sugar/wheat/apple
  which the engine lacks).
