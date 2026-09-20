#!/usr/bin/env python3
"""vault_ascii_dump.py — ASCII luma dump of reference vs vault for study.
Read-only comparison (clean-room verify step). """
import io, sys, zipfile
import numpy as np
from PIL import Image

ZIP = "/home/z/my-project/upload/textures.zip"
V = "/home/z/my-project/voxelcraft/assets-vault/assets/minecraft/textures"
RAMP = " .:-=+*#%@"

def dump(arr, label):
    rgb = arr[..., :3].mean(axis=2)
    a = arr[..., 3]
    print(f"--- {label} ({arr.shape[1]}x{arr.shape[0]}) ---")
    for y in range(arr.shape[0]):
        row = ""
        for x in range(arr.shape[1]):
            if a[y, x] < 40:
                row += " "
            else:
                row += RAMP[min(9, int(rgb[y, x] / 256 * 10))]
        print(row)

def main(name, frame=0):
    ref = np.asarray(Image.open(io.BytesIO(
        zipfile.ZipFile(ZIP).read("textures/" + name))).convert("RGBA"))
    fh = ref.shape[0] // max(1, ref.shape[1]) if ref.shape[0] > ref.shape[1] * 1.25 else 1
    if fh > 1:
        ref = ref[frame * ref.shape[1]:(frame + 1) * ref.shape[1]]
    ours = np.asarray(Image.open(V + "/" + name).convert("RGBA"))
    if fh > 1:
        ours = ours[frame * ours.shape[1]:(frame + 1) * ours.shape[1]]
    dump(ref, "REFERENCE " + name)
    dump(ours, "VAULT " + name)

if __name__ == "__main__":
    main(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 0)
