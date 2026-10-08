# Contributing to VoxelCraft-Rust

## License of contributions

By contributing you agree your work is licensed under the
**GNU General Public License v3.0 or later** (same as the project, see
LICENSE), with no additional terms. If that is unacceptable, do not
contribute.

Sign off every commit (Developer Certificate of Origin):

```
Signed-off-by: Your Name <you@example.com>
```

(`git commit -s` adds this.) The sign-off certifies the DCO 1.1:
you wrote the change or have the right to submit it, and you agree to
license it as above.

## Clean-room rules (binding, mirror legal rules L1–L8)

1. **Black-box only.** Never open, decompile, or read the reference
   game's jars, class files, mappings, or leaked source. Parity work
   comes from public documentation plus behaviour probes only.
2. **Zero copied assets.** No rights-holder texture, sound, model,
   font, text, or splash enters the repo. All art is synthesized
   procedurally in-repo (`scripts/legal_audit.py` enforces this in CI).
3. **No decompiled transcription.** Never copy proprietary decompiled
   source, variable names, or pseudocode into code, comments, or docs.
   Express mechanics as clean-room behavioural descriptions.
4. **No third-party trademarks** in code, UI, names, or docs
   (see TRADEMARKS.md). Functional vocabulary uses generic terms-of-art.
5. **Facts need sources.** Every numerical constant, formula, timing,
   or mechanic cites an authoritative public source; unsourced values
   are flagged `[ESTIMATED / APPROXIMATION]` (honest-claims rule).
6. **One slice per commit** (~300 changed lines max); keep diffs
   reviewable. Never rewrite pushed history.
7. **Behaviour must not change** for licensing/metadata-only commits;
   gameplay and rendering changes need tests and, where visual, V1
   screenshot review.

Violations are reverted on sight. When in doubt, stop and ask in the
pull request rather than guessing.
