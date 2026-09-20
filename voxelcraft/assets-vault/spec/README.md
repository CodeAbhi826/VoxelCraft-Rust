# spec/ — the clean-room intermediate

`spec.json` is the **measurement document** of the clean-room chain
(LEGAL-COMPLIANCE.md §1.4 steps 1–2): for each of the 3,855 reference
textures it stores ONLY aggregate facts — dimensions, palette histogram
(top-24 colors with fractional coverage, as numeric facts per §4.2),
banding/edge/symmetry/alpha statistics, and animation metadata — plus
the 176 parsed `.mcmeta` JSON values. **No pixel positions, no masks, no
silhouette coordinates** are stored: a histogram cannot be un-mixed back
into the original arrangement.

The synthesizer (`scripts/vault_synthesize.py`) reads this file — and
nothing else — to draw every vault pixel. Re-running the analyzer
requires the quarantined reference zip (`upload/textures.zip`, git
ignored); re-running the synthesizer requires only this file.

`synth_errors.txt` (if present) is a transient error log from the last
generation run. `verify_report.json` is the output of
`scripts/vault_verify.py`.
