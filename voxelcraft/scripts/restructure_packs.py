#!/usr/bin/env python3
"""restructure_packs.py — Phase 3 disk restructure.

Flattens the legacy ecosystem-style pack layout (assets/<namespace>/** +
pack.mcmeta) into the project's own layout (blockstates/ models/ textures/
+ pack.json at the pack root). Also renames the "programmer-art" pack to
"classic-art" and .png.mcmeta sidecars to .png.json.

Run from voxelcraft/ root: python3 scripts/restructure_packs.py
"""
import json
import os
import shutil
import sys

def move_tree(src, dst):
    if not os.path.isdir(src):
        sys.exit(f"missing: {src}")
    if os.path.exists(dst):
        sys.exit(f"dest exists: {dst}")
    shutil.move(src, dst)
    print(f"moved {src} -> {dst}")

def write_manifest(path, name, description):
    with open(path, "w", encoding="utf-8") as f:
        json.dump({"pack": {"format": 1, "name": name,
                            "description": description}}, f, indent=2)
        f.write("\n")
    print(f"wrote {path}")

# ---------------- builtin-pack ----------------
ns = "builtin-pack/assets/voxelcraft"
for sub in ["blockstates", "models", "textures"]:
    move_tree(f"{ns}/{sub}", f"builtin-pack/{sub}")
shutil.rmtree("builtin-pack/assets")
os.remove("builtin-pack/pack.mcmeta")
write_manifest("builtin-pack/pack.json", "voxelcraft-builtin",
               "VoxelCraft builtin pack (clean-room assets)")

# ---------------- classic-art (was programmer-art) ----------------
pa = "builtin-packs/programmer-art"
ns_pa = f"{pa}/assets/voxelcraft"
for sub in sorted(os.listdir(ns_pa)):
    move_tree(f"{ns_pa}/{sub}", f"builtin-packs/classic-art/{sub}")
shutil.rmtree(f"{pa}/assets")
os.remove(f"{pa}/pack.mcmeta")
os.rmdir(pa)
write_manifest("builtin-packs/classic-art/pack.json", "voxelcraft-classic-art",
               "The classic look of VoxelCraft (retro-style clean-room art)")

# ---------------- assets-vault ----------------
move_tree("assets-vault/assets/voxelcraft/textures", "assets-vault/textures")
shutil.rmtree("assets-vault/assets")
os.remove("assets-vault/pack.mcmeta")
write_manifest("assets-vault/pack.json", "voxelcraft-asset-vault",
               "VoxelCraft clean-room asset vault (original "
               "procedurally-synthesized art). Library copy for future "
               "versions — not wired into the engine. All pixels generated "
               "in-project; no third-party assets.")

# ---------------- .mcmeta sidecars -> .png.json ----------------
renamed = 0
for root, dirs, files in os.walk("."):
    dirs[:] = [d for d in dirs if d not in
               {"target", "node_modules", ".git", "crates"}]
    for f in files:
        if f.endswith(".png.mcmeta"):
            p = os.path.join(root, f)
            shutil.move(p, p[: -len(".png.mcmeta")] + ".png.json")
            renamed += 1
print(f"renamed {renamed} .png.mcmeta sidecars -> .png.json")
print("DONE")
