#!/usr/bin/env python3
"""vault_verify.py — clean-room chain step 4: VERIFY.

1. Byte identity: md5 of every reference byte-stream vs every vault PNG
   and mcmeta — must be ZERO matches (nothing copied).
2. Coverage: every reference png/mcmeta has a vault counterpart.
3. Per-texture similarity metrics (loaded read-only for comparison —
   verification, not synthesis): exact-pixel match, palette Jaccard,
   luma-profile distance, alpha-class agreement.
4. Report: per-family aggregates written to a JSON + console summary.
"""
import hashlib, io, json, zipfile
import numpy as np
from PIL import Image

ZIP = "/home/z/my-project/upload/textures.zip"
VAULT = "/home/z/my-project/voxelcraft/assets-vault/assets/minecraft/textures"
OUT = "/home/z/my-project/voxelcraft/assets-vault/spec/verify_report.json"

def md5(b):
    return hashlib.md5(b).hexdigest()

def main():
    z = zipfile.ZipFile(ZIP)
    ref_md5 = {}
    for info in z.infolist():
        if info.is_dir():
            continue
        ref_md5[md5(z.read(info.filename))] = info.filename
    # byte identity over vault tree
    import os
    collisions = []
    vault_files = []
    for root, _, files in os.walk(VAULT):
        for f in files:
            p = os.path.join(root, f)
            vault_files.append(p)
            h = md5(open(p, "rb").read())
            if h in ref_md5:
                collisions.append((p, ref_md5[h]))
    # coverage + metrics
    metrics = []
    per_family = {}
    for info in z.infolist():
        name = info.filename
        if info.is_dir() or not name.endswith(".png"):
            continue
        rel = name.replace("textures/", "", 1)
        vp = os.path.join(VAULT, rel)
        if not os.path.exists(vp):
            metrics.append(("MISSING", rel))
            continue
        try:
            ref = np.asarray(Image.open(io.BytesIO(z.read(name))).convert("RGBA"))
            ours = np.asarray(Image.open(vp).convert("RGBA"))
        except Exception as e:
            metrics.append(("DECODE-ERR", rel))
            continue
        same_dims = ref.shape == ours.shape
        if not same_dims:
            metrics.append(("DIMS", rel))
            continue
        h, w = ref.shape[:2]
        # exact pixel equality
        eq = (ref == ours).all(axis=2)
        pix_eq = float(eq.mean())
        # palette Jaccard (color sets)
        refc = {tuple(c) for c in ref.reshape(-1, 4)[:: max(1, (w * h) // 4096)]}
        ourc = {tuple(c) for c in ours.reshape(-1, 4)[:: max(1, (w * h) // 4096)]}
        jac = len(refc & ourc) / max(1, len(refc | ourc))
        # luma profile (8-bucket histogram, OPAQUE pixels only — the RGB
        # of transparent pixels is undefined junk)
        mask = (ref[..., 3] >= 128) & (ours[..., 3] >= 128)
        if mask.sum() > 16:
            rl = ref[..., :3].mean(axis=2)[mask]
            ol = ours[..., :3].mean(axis=2)[mask]
            rh, _ = np.histogram(rl, bins=8, range=(0, 255))
            oh, _ = np.histogram(ol, bins=8, range=(0, 255))
            rh = rh / max(1, rh.sum()); oh = oh / max(1, oh.sum())
            luma_d = float(np.abs(rh - oh).sum() / 2)  # total variation distance
        else:
            luma_d = 0.0
        # alpha class agreement
        ra = (ref[..., 3] < 128).mean() > 0.2
        oa = (ours[..., 3] < 128).mean() > 0.2
        metrics.append(dict(n=rel, cat=rel.split("/")[0], pix=round(pix_eq, 4),
                            pal=round(jac, 3), luma=round(luma_d, 3),
                            alpha_ok=bool(ra == oa)))
        fam = rel.split("/")[0]
        d = per_family.setdefault(fam, dict(n=0, pix=0.0, pal=0.0, luma=0.0, alpha=0))
        d["n"] += 1
        d["pix"] += pix_eq; d["pal"] += jac; d["luma"] += luma_d
        d["alpha"] += int(ra == oa)
    for fam, d in per_family.items():
        n = max(1, d["n"])
        d["pix"] = round(d["pix"] / n, 4)
        d["pal"] = round(d["pal"] / n, 3)
        d["luma"] = round(d["luma"] / n, 3)
        d["alpha"] = round(d["alpha"] / n, 3)
    # mcmeta coverage
    mc_missing = 0
    for info in z.infolist():
        nm = info.filename
        if nm.endswith(".mcmeta"):
            rel = nm.replace("textures/", "", 1)
            if not os.path.exists(os.path.join(VAULT, rel)):
                mc_missing += 1
    report = dict(
        byte_collisions=[c[0] for c in collisions],
        vault_files=len(vault_files),
        per_family=per_family,
        mcmeta_missing=mc_missing,
        issues=[m for m in metrics if isinstance(m, tuple)],
    )
    json.dump(report, open(OUT, "w"), indent=1)
    print(f"vault files: {len(vault_files)} | byte collisions: {len(collisions)} "
          f"(must be 0) | mcmeta missing: {mc_missing} | issues: {len(report['issues'])}")
    print(f"{'category':14s} {'n':>5s} {'pix-eq':>8s} {'pal-jac':>8s} {'luma-TVD':>9s} {'alpha':>7s}")
    for fam in sorted(per_family):
        d = per_family[fam]
        print(f"{fam:14s} {d['n']:5d} {d['pix']:8.3f} {d['pal']:8.3f} {d['luma']:9.3f} {d['alpha']:7.2f}")
    if collisions:
        print("COLLISIONS:", collisions[:10])
        raise SystemExit(1)

if __name__ == "__main__":
    main()
