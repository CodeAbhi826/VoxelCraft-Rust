# BLOCKERS.md — soft blockers (need owner input/checks) and hard stops

HARD STOP: GitHub Actions billing block (2026-10-09) — RESOLVED via
public visibility (owner-ordered): runners accepted work at once,
bench uploads succeeding. Compute gate green; queued slices validating.

(A3: each entry carries the exact ask + the default the work continues under.
HARD STOP OPEN (see top): CI billing block. The STOP file is absent.)

## Soft blocker 5 — provider output-ownership terms (L7, owner confirm, OPEN)
- Plain version: the code in this repo was written with AI help. Your
  AI provider's user terms decide who owns that output. Please check
  your provider's terms for one thing: do they let you (the account
  holder) release AI-assisted output under an open-source license
  (GPL-3.0-or-later)? If yes, reply "confirmed" and this blocker
  closes. If no (or they claim ownership), say so and the release
  re-scopes. Nothing in the repo changes either way right now.
- Default (work continues under it): DCO + GPL-on-entry + AUTHORSHIP
  honesty stands; Part 9 re-verifies before release.

## Soft blocker 4 — docs license DECIDED 2026-10-10 (owner: split)
- Decision: custom art stays CC-BY-SA-4.0; repo docs/prose
  (`docs/**`, root `*.md`) go GPL-3.0-or-later. Queued: L3b
  `reuse lint` gate + REUSE.toml annotations for prose.
- (Split read from owner's "combine" answer; correct this entry if
  misread.)

## Soft blocker 3 — phases gate DECIDED 2026-10-10 (owner: keep)
- Decision: keep the 0.9 CI gate; the lower local number is
  environmental. No E2E threshold touched in code.

## Soft blocker 2 — Part 1 foreground capture sweep (2026-10-09, partial)
- DONE locally: F3 pair, MENU tree, FKEYS views (8 images viewed, V1 in
  WORKLOG 2026-10-09d), beds/fluids/containers green. Quota prune 104→44.
- STILL OPEN: (a) turntable 5 views (needs fresh CI binary post-quota —
  Oct-8 binary predates 1.11); (b) settings/title/inventory screens —
  X capture black + xdotool undelivered (no WM), engine dumps don't
  exist for menu screens (follow-up slice: dumps in E2E_MENU);
  (c) face-eye symmetry + arm-design checks need close-ups.
- Default: licensing → Part 2 through CI; sweep remainder runs when a
  fresh binary is downloadable.

## Soft blocker 1 — SMAA-vs-FXAA 3 ms call (slice 1.7, PLAN-FINAL §4, owner authorized 2026-10-10)
- Owner authorized measuring on this system's real GPU. Queued slice:
  probe GPU presence (numbers only, identity never recorded), run the
  FXAA on/off bench, then decide SMAA vs FXAA on measured cost.
- Default (work continues under it): FXAA ships as the ONE method; no
  MSAA, no TAA; AA forced on pre-upscale below 1.0 render scale. SMAA
  lands only on your explicit ask with your numbers attached.
