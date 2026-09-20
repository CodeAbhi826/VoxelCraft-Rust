#!/usr/bin/env python3
"""voxel_synth_shim.py — re-export the core synthesis helpers so the
special/gui/entity modules can use star-imports uniformly."""
from vault_synth_core import (rng_for, value_noise, fbm, Pal, canvas, fill,
                              px, to_image, save, disc, blob, line,
                              bevel_panel, sprite_from_fn)
