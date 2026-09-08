#!/usr/bin/env python3
"""1.14 part-3 flowers round: blocks.rs test/count sync.

Bumps the registry-count drift asserts for the two new flower blocks
(ids 430/431, states 696/697, picker 391) and extends the V11 window
test with flower coverage (F3 plain-name lines, picker entries, atlas
guard).
"""
import sys

P = "crates/vc-blocks/src/blocks.rs"
src = open(P).read()
orig = src

def sub1(old, new, count=1):
    global src
    if old not in src:
        print(f"MISS: {old!r}")
        sys.exit(1)
    src = src.replace(old, new, count)

def LH(s):  # literal '[h' safe-constructor (display-pipeline paranoia)
    return s.replace("@@", "[h")

# --- 1) registry-count drift asserts: 430 -> 432, 696 -> 698 ---
n = src.count("assert_eq!(BLOCK_COUNT, 430")
src = src.replace("assert_eq!(BLOCK_COUNT, 430", "assert_eq!(BLOCK_COUNT, 432")
print(f"BLOCK_COUNT asserts bumped: {n}")
n = src.count("assert_eq!(STATE_COUNT, 696")
src = src.replace("assert_eq!(STATE_COUNT, 696", "assert_eq!(STATE_COUNT, 698")
print(f"STATE_COUNT asserts bumped: {n}")
sub1('"merged registry + V6 + V7 + V8 + V9 + 1.14 V10/V11 (nature half)"',
     '"merged registry + V6 + V7 + V8 + V9 + 1.14 V10/V11 (nature half, flowers)"')
sub1('"merged state space, V11 states end at 695"', '"merged state space, V11 states end at 697"')

# --- 2) flower F3 description asserts (after the lantern [hanging lines) ---
anchor = LH('assert_eq!(state_description(V11_STATE_BASE + 5), "Lantern@@anging=true]");')
add = '''
        // the flowers carry no properties — plain names in the F3 line
        assert_eq!(state_description(V11_STATE_BASE + 7), "Cornflower");
        assert_eq!(state_description(V11_STATE_BASE + 8), "Lily of the Valley");'''
sub1(anchor, anchor + add)

# --- 3) the v11 test tail: picker contains + atlas guard + window shape ---
old_tail = """        // the window is in the picker; the nugget is an item-block
        for want in [BLAST_FURNACE, SMOKER, LANTERN] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        assert!(!PICKER_BLOCKS.contains(&IRON_NUGGET), "the nugget is an item, not a picker block");
        assert!(is_item_block(IRON_NUGGET), "nugget is an item-block");
        // tiles within the atlas guard
        assert!(TILE_MAX >= TILE_IRON_NUGGET);
        // bounds + window shape
        assert_eq!(V11_COUNT, 7);
        assert_eq!(BLOCK_COUNT, 432);
        assert_eq!(STATE_COUNT, 698);
    }
}"""
new_tail = """        // the window is in the picker; the nugget is an item-block;
        // the flowers are placeable picker blocks (not items)
        for want in [BLAST_FURNACE, SMOKER, LANTERN, CORNFLOWER, LILY_OF_THE_VALLEY] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        assert!(!PICKER_BLOCKS.contains(&IRON_NUGGET), "the nugget is an item, not a picker block");
        assert!(is_item_block(IRON_NUGGET), "nugget is an item-block");
        assert!(!is_item_block(CORNFLOWER), "cornflower is placeable");
        assert!(!is_item_block(LILY_OF_THE_VALLEY), "lily of the valley is placeable");
        // tiles within the atlas guard
        assert!(TILE_MAX >= TILE_IRON_NUGGET);
        assert!(TILE_MAX >= TILE_LILY_OF_THE_VALLEY, "flower tiles within the atlas guard");
        // bounds + window shape
        assert_eq!(V11_COUNT, 9);
        assert_eq!(BLOCK_COUNT, 432);
        assert_eq!(STATE_COUNT, 698);
    }
}"""
sub1(old_tail, new_tail)

assert src != orig
open(P, "w").write(src)
print("blocks.rs patched OK")
