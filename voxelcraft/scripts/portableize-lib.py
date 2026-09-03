#!/usr/bin/env python3
"""portableize-lib.py — make a workspace member crate STANDalone.

The released library archives must build WITHOUT the workspace:
  * `X.workspace = true` package keys -> concrete values
  * external deps `{ workspace = true }` -> concrete version specs
  * internal vc-* deps -> sibling path deps (`../vc-blocks`), so a user
    who extracts several library archives into one folder gets a
    working dependency tree with no workspace, no registry lookup.

Usage: python3 portableize-lib.py crates/vc-nbt   (rewrites in place,
or with --staging DIR copies first and rewrites the copy)
"""
import re
import shutil
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent  # voxelcraft/
WS = tomllib.loads((ROOT / "Cargo.toml").read_text())
WSVER = WS["workspace"]["package"]["version"]
WSDEPS = WS["workspace"].get("dependencies", {})


def concrete_external(dep: str, extra: str) -> str:
    spec = WSDEPS[dep]
    parts = []
    if isinstance(spec, str):
        parts.append(f'version = "{spec}"')
    else:
        if "version" in spec:
            parts.append(f'version = "{spec["version"]}"')
        if "git" in spec:
            parts.append(f'git = "{spec["git"]}"')
        if "branch" in spec:
            parts.append(f'branch = "{spec["branch"]}"')
        if "tag" in spec:
            parts.append(f'tag = "{spec["tag"]}"')
    # merge member extras (features/optional) with workspace features
    feats = list(spec.get("features", [])) if isinstance(spec, dict) else []
    m = re.search(r'features\s*=\s*\[([^\]]*)\]', extra)
    if m:
        for f in m.group(1).split(","):
            f = f.strip().strip('"').strip("'")
            if f and f not in feats:
                feats.append(f)
    if feats:
        parts.append("features = [" + ", ".join(f'"{f}"' for f in feats) + "]")
    if "optional = true" in extra or (isinstance(spec, dict) and spec.get("optional")):
        parts.append("optional = true")
    if not parts:
        parts.append('version = "*"')
    return "{" + ", ".join(parts) + "}"


def portableize(crate_dir: Path) -> None:
    toml_path = crate_dir / "Cargo.toml"
    s = toml_path.read_text()

    # package keys
    s = re.sub(r"^version\.workspace = true$",
               f'version = "{WSVER}"', s, flags=re.M)
    s = re.sub(r"^edition\.workspace = true$",
               'edition = "2021"', s, flags=re.M)
    repo = WS["workspace"]["package"].get("repository", "")
    s = re.sub(r"^repository\.workspace = true$",
               f'repository = "{repo}"', s, flags=re.M)
    s = re.sub(r"^license\.workspace = true$", 'license = "MIT"', s, flags=re.M)

    # dependency specs: NAME = { workspace = true, ...extras }
    def repl(m):
        indent, dep, extra = m.group(1), m.group(2), m.group(3)
        if dep.startswith("vc-"):
            inner = ['path = "../%s"' % dep]
            feats = re.search(r'features\s*=\s*\[([^\]]*)\]', extra)
            if feats:
                inner.append("features = [" + feats.group(1) + "]")
            if "optional = true" in extra:
                inner.append("optional = true")
            return f"{indent}{dep} = {{ {', '.join(inner)} }}"
        try:
            return f"{indent}{dep} = {concrete_external(dep, extra)}"
        except KeyError:
            return m.group(0)  # unknown -> leave alone (compile will tell)

    s = re.sub(r"^(\s*)([A-Za-z0-9_-]+) = \{ workspace = true([^}]*)\}",
               repl, s, flags=re.M)
    toml_path.write_text(s)
    print(f"portableized {crate_dir.name}")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    staging = None
    if "--staging" in sys.argv:
        staging = Path(sys.argv[sys.argv.index("--staging") + 1])
    for a in args:
        crate = ROOT / a if not a.startswith("/") else Path(a)
        if not (crate / "Cargo.toml").exists():
            print(f"skip (no Cargo.toml): {crate}")
            continue
        if staging:
            dest = staging / crate.name
            shutil.rmtree(dest, ignore_errors=True)
            shutil.copytree(crate, dest,
                            ignore=shutil.ignore_patterns("target"))
            portableize(dest)
        else:
            portableize(crate)


if __name__ == "__main__":
    main()
