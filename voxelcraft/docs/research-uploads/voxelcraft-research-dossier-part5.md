# VoxelCraft-Rust → Minecraft Replication: Research Dossier — PART 5
*Continues from Parts 1-4. Additive only. Built from GLM's (z.ai) delegated research pass, cross-checked against a second independent AI pass (Qwen Max) and live sources where the two disagreed.*

---

## 24. Verification method used this round

GLM was given the Part 4 research-delegation prompt directly. Its output was honest about its own limits (explicitly listed what it couldn't verify) and cited real sources throughout. A second, independently-produced document (Qwen Max, not delegated by this process — user-supplied for comparison) covered similar ground with no hedging at all. Comparing the two surfaced concrete disagreements, three of which were resolved against live sources below. **This cross-referencing technique — running two independent research passes and checking where they diverge — is worth reusing**, since agreement between independent sources raises confidence and disagreement flags exactly what needs a live check.

## 25. Resolved disputes (confirmed against live sources this round)

| Value | GLM said | Qwen said | **Confirmed real value** |
|---|---|---|---|
| Obsidian hardness / blast resistance | 50 / 1,200 | -1 (unbreakable) / 1,800,000 | **50 / 1,200** (Java Edition) — multiple independent sources agree; Qwen's numbers resemble bedrock's, not obsidian's |
| Stone blast resistance | 6 (implied) | 3.5 | **6** — confirmed via End Stone's wiki page stating its own 9 is "3 more than ordinary stone" |
| Fall damage formula | `fall_distance − 3` (half-heart points) | `(fall_distance − 3) / 2` | **`fall_distance − 3`** — confirmed directly from Mojang's own bug tracker (MC-12357) |

**Takeaway**: GLM was correct on all three checked disputes; Qwen was wrong on all three. This doesn't mean everything in GLM's document is correct — it means GLM's citation discipline produced better results than Qwen's confident-no-hedging approach, which is exactly the behavior difference worth expecting going forward.

## 26. Confirmed data from GLM's pass (single-sourced, not yet independently cross-checked beyond GLM's own citations)

### Block hardness/blast resistance (partial — GLM explicitly noted this needs a full extraction pass)
Bedrock: unbreakable / 3,600,000 · Obsidian: 50 / 1,200 (now doubly-confirmed, §25) · Ancient Debris: 30 / 1,200 · Ender Chest: 22.5 / 600 · Anvil: 5 / 1,200 · Enchanting Table: 5 / 1,200 · Stone: 1.5 / 6 (now doubly-confirmed, §25) · Dirt: 0.5 / 0.5 · Planks: 2 / 3 · Water/Lava: 100 / 100.

### Breaking-speed formula
A block breaks when accumulated tool damage exceeds `hardness × 30` ticks; if the tool tier can't harvest the block, hardness is effectively ×5 (takes 5× longer by an insufficient tool).

### Mob data (representative subset — full ~102-entity table still open)
Difficulty damage scaling: Hard = 1.5× Normal; Easy = min(D, 0.5D + 1). Zombie 20 HP / 3 dmg / 0.23 speed · Skeleton 20 HP / 3 melee, 3-5 arrow / 0.25 speed · Creeper 20 HP / 43 explosion / 0.25 speed · Enderman 40 HP / 7 dmg / 0.3 speed · Wither 300 HP (boss) · Ender Dragon 200 HP (boss).

Movement-speed attribute conversion: `blocks/s ≈ 43.178 × attribute_value`. Player walk = 4.317 blocks/s, sprint = 5.612 blocks/s.

### Combat formulas
Armor reduction: `4% per armor point, capped at 80%, reduced by toughness`. Attack cooldown: `T = 20 / attack_speed` ticks; damage scales `0.2 + 0.8p²` where p = cooldown completion fraction; critical hits/sweep/sprint-knockback require p ≥ 84.8%.

Weapon base damage by material (sword/axe/pickaxe/shovel/hoe) documented but GLM flagged the column-header mapping as unconfirmed — needs a direct page visit, not re-derived here.

### Hunger/exhaustion
Swimming 0.01/m · mining 0.005/block · sprinting 0.1/m · jump 0.05 · sprint-jump 0.2 (GLM correction: this was 0.8 pre-1.11, changed to 0.2 in 1.11 — relevant since some sources still cite the old value) · regen costs 1.5 exhaustion/HP (hunger-based) or 6.0/HP (saturation-based, "fast" regen). Starvation floors: Easy = 10 HP min, Normal = 1 HP min, Hard = lethal, Peaceful = no depletion.

### Enchanting
Max 15 bookshelves affect a table (more are ignored); bookshelves must be exactly 1-2 blocks away with the 2-high gap kept clear of even carpets/torches. The four Protection-family enchantments (Protection/Fire/Blast/Projectile) are mutually exclusive.

### Brewing
20-second (400-tick) brew cycle; Blaze Powder fuel = 20 charges; chain is Water Bottle → (Nether Wart) → Awkward Potion → (ingredient) → Potion → (Redstone extends / Glowstone amplifies / Gunpowder → splash / Dragon's Breath → lingering).

### Redstone timings
Repeater: 1-4 redstone-tick delay (2/4/6/8 game ticks), locks when powered laterally by another active repeater/comparator. Torch: 8 state changes in 60 game ticks triggers burnout (5s inert + smoke). Comparator: compare mode (output = A if A ≥ max(B1,B2), else 0) / subtract mode (output = max(0, A − max(B1,B2))).

**Flagged as possibly unit-confused in the Qwen cross-check, not yet resolved**: TNT fuse timing and hopper transfer timing were given by Qwen under a "redstone ticks" column header using what look like game-tick values (80 and 8 respectively) — worth a direct check before use, since redstone ticks are 2× game ticks.

## 27. Explicitly NOT researched (GLM's own admission, still open)

Crafting recipe list (needs real datapack JSON, not attempted), `commands.json` parsing (needs the actual file), structure generation specs, full biome list detail, weather system specifics, advancements/statistics systems, and the expanded farm/glitch catalog. See §28 for the continuation prompt covering exactly these.

## 28. Open items carried forward

- Full block hardness/blast-resistance table (only a partial set confirmed so far).
- Full ~102-mob stat table (only ~10 representative mobs confirmed).
- Per-tier weapon damage table (needs direct wiki page visit, not just search snippet).
- TNT/hopper redstone-timing unit confusion, unresolved.
- Everything in §27.
- All open decisions from Parts 1-4 still stand.
