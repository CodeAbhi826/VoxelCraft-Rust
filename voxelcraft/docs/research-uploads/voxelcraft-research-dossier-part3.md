# VoxelCraft-Rust → Minecraft Replication: Research Dossier — PART 3
*Continues from Parts 1 & 2. Additive only. Hand all three files over when ready to draft the master prompt.*

---

## 15. Real data extracted from user's own `level.dat` (Fabric 26.2, Arch Linux)

File confirmed as genuine gzip-compressed NBT, 558 bytes decompressed. Parsed directly (not memory-derived):

- `GameType: 1` (Creative) — live confirmation matching what was already found hardcoded in VoxelCraft's `vc-anvil/save.rs`.
- `DataPacks.Enabled: ["vanilla"]`, `DataPacks.Disabled: ["minecart_improvements", "redstone_experiments", "trade_rebalance"]` — confirms 26.2 ships these three as **opt-in experimental datapacks**, off by default. Real, current, exact names.
- `ServerBrands: ["fabric"]`, `WasModded: true` — world was created via Fabric.
- `singleplayer_uuid` stored as a **4-element Int Array** (not a string) — exact NBT schema detail for player-UUID compatibility if ever relevant to `vc-anvil`.
- `Version: { Series: "main", Name: "26.2" }`.
- `spawn.dimension: "minecraft:overworld"` — confirms the exact registry-key string format (`minecraft:overworld`) used for dimension references in real save data.
- **No `RandomSeed`/`WorldGenSettings` field present** — this file was captured essentially at world creation (`LevelName: "New World"`, ~14,000 ticks / ~12 in-game minutes played). A `level.dat` from a world played longer would be needed to capture the exact seed-storage schema.

## 16. Data-generator command — corrected

User's `--reports` attempt failed two ways, both now resolved:
1. Ran it against the **client** jar (`26.3-pre-1.jar`) — data generator only reliably runs from the **server** jar.
2. Ran it without the bundler override — since 1.18 the server jar is a bundler whose default entry point doesn't recognize `--reports` at all (hence the `joptsimple.UnrecognizedOptionException`). Correct invocation:
```bash
java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports
```
If `--reports` still misbehaves on 26.x, try `--all` instead.

Quick way to locate all relevant files inside a Prism/MultiMC-style instance (user's actual setup, not default `~/.minecraft`):
```bash
find "/home/abhin/Games/Minecraft/versions/Fabric 26.2/Fabric 26.2/" -maxdepth 2 -iname "options*.txt" -o -iname "level.dat" -o -iname "*.jar"
```

---

## 17. Farm/glitch mechanics — verified this round (source-quality tiered)

**Note on source reliability going forward**: `bugs.mojang.com` / `bugs-legacy.mojang.com` (Mojang's own public bug tracker) is the highest-confidence source for this category — official, primary, exact version numbers. Wiki-style sites (redstuff.fandom, mcdf.wiki.gg) are good secondary sources. Generic SEO/content-farm sites returned mixed/garbled results this round and should be treated as low-confidence or skipped in future pulls — flagged explicitly rather than repeated here.

### Zero-tick farming — root cause confirmed, connects to instant repeaters below
While a piston is extending/retracting (1.5 ticks), the block it's moving temporarily becomes a **"moving block" placeholder (block ID 36)** that does not conduct redstone signal and behaves like glass. Exploit: pushing dirt under a self-stacking plant (sugarcane/cactus/bamboo/kelp/vines/chorus flower) during this transient window caused the plant to receive conflicting support-check signals, forcing a full growth stage instantly instead of waiting for its normal random tick. **Patched in Java Edition 1.16.**

### Instant repeaters — same root mechanism as zero-ticking
Because the "moving block" (ID 36) doesn't conduct redstone during a piston's 1.5-tick travel, redstone circuits built around a piston push/pull can produce a **0-tick-delay signal change** in one direction while still exhibiting normal 1.5-tick delay in the other — used deliberately as a legitimate redstone-engineering technique, not just a leftover bug artifact. This is a well-documented, still-relevant technical-redstone building block, not a patched-out exploit.

### Headless pistons — long multi-version history, still partially alive
A piston can end up in an "extended" state missing its visible head block, used decoratively/functionally by technical builders. Documented via multiple distinct historical methods (mining the head during specific 1.8 snapshots, pre-powering bugs, bed-placement-on-replaceable-plant interactions, 0-ticking a piston push). **Confirmed still present as an active, unresolved bug on Bedrock Edition** as of recent 1.21.4x preview builds per Mojang's own tracker (MCPE-186432) — Java and Bedrock diverge on exact rendering behavior.

### Bedrock-breaking exploits — category confirmed, specifics too version-volatile to log exactly
General category is real and long-running (piston-based manipulation, TNT-duplication-based explosions, ender-pearl-phasing at Nether-roof boundaries), but methods are **highly version-specific and patched frequently** — multiple sources this round explicitly warned that a working method in one version is often already dead in the next. Treat as a category to reference generally ("bedrock has historically had piston/explosion/pearl-phasing exploits"), not a specific recipe to replicate exactly, since no single current verified method was confirmed with high confidence this round.

### Still open
The full "100+" catalog is not complete — this round covered 4 well-verified mechanics plus the previously-logged quasi-connectivity/TNT-duping/raid-farms from Part 2 (7 total). A further dedicated pass specifically through Mojang's own bug tracker and `minecraft.wiki`'s `Tutorials/` namespace (confirmed to exist, per Part 2 §11) is still the right way to extend this list — flagged as a continuing task, not attempted further by broad web search this round due to declining source quality.

---

## 18. Open items carried forward

- Full farm/glitch catalog still incomplete (§17) — needs a `Category:Tutorials` / Mojang-bug-tracker-specific pass, not generic web search.
- A `level.dat` from an actually-played world still needed for exact `WorldGenSettings`/seed schema (§15).
- All open decisions from Part 1 §8 and Part 2 §14 still stand.
