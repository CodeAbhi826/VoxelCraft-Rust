# Research Verdicts — Extended Mechanics & UI/Visuals Documents

Status: **GATE — read before implementing anything from either raw research
document.** Both documents were AI-generated research passes, spot-checked (not
verified line-by-line). This file records the verified verdicts so no claim
flows into code without a decision. Rule: **confirmed-correct → use as-is;
confirmed-wrong → skip entirely and apply the correction; unverified → treat as
a placeholder, verify live (minecraft.wiki, the project's own generated
registry data) or mark it explicitly before it reaches code.**

Verdict pass date: 2026-09-05 (this session, user-supplied categorization).

---

## Document 1 — mechanics/physics extension

### Confirmed correct (verified against real sources)

| Claim | Verdict | Notes |
|---|---|---|
| Gravity/falling formula `v1 = (v0 − 0.08) × 0.98` | ✅ use exactly | Verified against a quoted decompiled MC code snippet (`motionY *= 0.9800000190734863D` applied after the 0.08 subtraction). Terminal velocity 3.92 blocks/tick is consistent. |
| Minecart max speed 8 blocks/second | ✅ use | Confirmed from minecraft.wiki's own Minecart page (0.4 blocks/tick cap). |
| Mob pathfinding = "modified A* algorithm" | ✅ at this generality | Real, well-established. The doc is honest that exact weight/cost values are not published — do NOT invent specific numbers beyond this. |

### Confirmed WRONG — never implement

| Claim | Verdict | Correction |
|---|---|---|
| Section 11 "Pointed Dripstone Fall Damage" — **the entire section** | ❌ delete outright | Pointed dripstone / dripstone caves / stalactites / stalagmites are Caves & Cliffs Part I content — **added in Minecraft 1.17**. None of it exists in 1.16.5. Not a "verify the formula" case: the mechanic itself is out of scope for this project's target version. No dripstone-related fall damage, ever. |

### Unverified — placeholders until a live check

- Falling-block "fall delay" values (2/5 game ticks) — source does not clearly
  establish this as a per-block-type mechanic; verify against the
  `FallingBlockEntity` behavior description on minecraft.wiki first.
- Minecart friction 0.01 and "empty minecarts decelerate faster than occupied"
  — the document itself flags this as not fully verified; do not hardcode 0.01
  without a direct citation.
- Swimming speeds (1.81 / 0.39 / 3.918 blocks/s) — plausible, not checked.
- Nether biome mob spawn weights — community-compiled, explicitly not
  1.16.5-exact; cross-check against the project's own registries or a direct
  minecraft.wiki biome page before using specific numbers.
- End enderman group-of-4 packing, firework boost 33.5 b/s + duration formula,
  passive-mob "only respawns in new chunks", villager gossip decay/share
  numbers, boat acceleration, furnace-minecart fuel values, hopper-minecart
  collection rate — none checked this round. The document's own
  "STILL NOT FULLY VERIFIED" table is honest: **keep it, don't discard it**,
  when this content is used.

---

## Document 2 — UI/visuals

### Confirmed correct

| Claim | Verdict | Notes |
|---|---|---|
| Monocraft font: OFL 1.1 (font) + GPL 3 (generator code) | ✅ use | Matches earlier independent verification. The font solution. |
| Legal framework: functional layout dims/slot counts = safe; specific pixel art, exact palettes, bitmap fonts = not safe | ✅ keep | Consistent with the established clean-room boundary (Tetris Holding v. Xio / Spry Fox v. Lolapps). |

### Confirmed WRONG — re-derive per screen

| Claim | Verdict | Correction |
|---|---|---|
| "All 18 container screens are uniformly 176×166" (Part 5's blanket table) | ❌ do not use | A real vanilla-convention hopper GUI is **176×133** — hoppers have 5 content slots, fewer rows than a crafting table or chest, so the screen is genuinely shorter. The table cannot be trusted for ANY screen without individual verification. Also likely wrong for: brewing stand, beacon, anvil, enchanting table (distinct content ⇒ distinct heights). **Every container screen's height must be individually verified before implementation.** |

### Unverified — placeholders until a live check

- F3 "28 elements" list + exact hotkey table — not version-pinned to 1.16.5;
  some listed lines (e.g. "Looking at Fluid" as a separate line) may be from a
  different version. Re-verify the exact 1.16.5 F3 set before implementing all
  28 lines as stated.
- `options.txt` defaults, specifically `version:3105` — a real captured
  options.txt from a live 26.2 install shows `version:3465`; **neither number
  is confirmed as the genuine 1.16.5 default**. If the exact `version:` field
  matters for save compatibility, verify against a real 1.16.5 capture — not
  either document.
- HUD pixel dimensions (hotbar 182×22, health/hunger 81×8, …) — commonly and
  consistently cited across multiple independent sources in the broader
  research; lower-risk, but not independently re-verified this round. Treat as
  "probably fine", not "confirmed".
- "~60–80% of a major engine update" work estimate — the document's own
  self-assessment/opinion, NOT a verified fact; not authoritative scope sizing.

---

## Cross-cutting trap — the wrong fall-damage formula

Multiple low-quality SEO/content-farm sources repeat `(height − 3) × 0.2`.
**That formula is wrong** — this project already debunked it via Mojang's own
bug tracker. The real formula (1.16.5):

```
damage = fall_distance − 3        (half-heart units, no multiplier)
```

The engine already implements this correctly (game.rs, "MC-12357: fall − 3
HP"). If either document — or any future research pass — restates the `× 0.2`
version: **flag and correct it.** Repetition across scraped/SEO sources is NOT
independent confirmation.

---

## Enforcement

1. Anything implemented from either raw document cites this file's row for the
   claim (commit message or `§`-note).
2. Unverified values that survive a reasonable verification effort unverified
   become **explicitly-marked placeholders** in code, disclosed in the commit —
   never silently treated as fact.
3. Confirmed-wrong rows are not "future work" — they are deleted scope.
