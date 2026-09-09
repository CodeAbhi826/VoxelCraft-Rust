#!/usr/bin/env python3
"""Bracket 16/16 follow-up: the 8 wasm32-only warnings (the mirror set).

- Threading variant: constructed only on native -> allow + doc
- ws_selected: read only by the native world-select UI -> allow + doc
- world_name / shader_packs `mut`: used by native-only scan blocks ->
  the file's existing #[cfg_attr(wasm32, allow(unused_mut))] convention
- report_datapacks: native-only callers (wasm boots without a datapack
  filesystem) -> cfg_attr allow
- drop(&mut entry) / drop(&mut e): no-op reference drops -> removed
- Some("emerald") arm at 8438: dead duplicate (8417 already maps it) ->
  removed; behavior unchanged (the earlier arm always won)
"""

import re
import sys
from pathlib import Path

ROOT = Path("/home/z/my-project/voxelcraft")
F = "crates/voxelcraft/src/game.rs"

DELETES = [  # (1-based line, exact content) — applied bottom-up
    (8504, "                        drop(e);"),
    (8438, "                            Some(\"emerald\") => Some(EMERALD_ORE),"),
    (8344, "                                drop(entry);"),
]

INSERTS = [  # (regex anchor, lines to insert above, expected hits)
    (r"^    Threading \{$",
     ["    /// the native-only backend (wasm constructs Inline below)",
      "    #[allow(dead_code)]"], 1),
    (r"^    ws_selected: Option<usize>,$",
     ["    #[allow(dead_code)] // the world-select state (native-only UI)"], 1),
    (r"^        let mut world_name = String::from\(\"VoxelCraft\"\);$",
     ["        #[cfg_attr(target_arch = \"wasm32\", allow(unused_mut))] // native scan below"], 1),
    (r"^        let mut shader_packs = vc_render::shaders::builtin_packs\(\);$",
     ["        #[cfg_attr(target_arch = \"wasm32\", allow(unused_mut))] // native append below"], 1),
    (r"^fn report_datapacks\(loaded: &vc_pack::datapack::LoadedData\) \{$",
     ["/// native-only callers (the wasm boot has no datapack filesystem)",
      "#[cfg_attr(target_arch = \"wasm32\", allow(dead_code))]"], 1),
]


def main() -> int:
    p = ROOT / F
    lines = p.read_text(encoding="utf-8").split("\n")

    # validate deletes
    for line, want in DELETES:
        if lines[line - 1] != want:
            print(f"VALIDATION FAILED {F}:{line}\n want: {want!r}\n got:  {lines[line - 1]!r}")
            return 1
    # validate insert anchors
    for pat, ins, expect in INSERTS:
        hits = [i for i, l in enumerate(lines) if re.match(pat, l)]
        if len(hits) != expect:
            print(f"VALIDATION FAILED {F}: {pat!r} matched {len(hits)} (want {expect})")
            return 1
    print("validation ok")

    for line, _ in sorted(DELETES, key=lambda d: -d[0]):
        del lines[line - 1]
    print(f"deleted {len(DELETES)} lines")

    for pat, ins, expect in INSERTS:
        hits = [i for i, l in enumerate(lines) if re.match(pat, l)]
        assert len(hits) == expect
        for i in reversed(hits):
            lines[i:i] = ins
    print(f"inserted {len(INSERTS)} blocks")

    p.write_text("\n".join(lines), encoding="utf-8")
    print("APPLIED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
