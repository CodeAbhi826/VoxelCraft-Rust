#!/usr/bin/env bash
# 1.16 Nether Update PART 2 research captures (minecraft.wiki, live
# 2026-09-08): the three mobs (strider/piglin/hoglin), the crimson/
# warped wood families, soul torch/lantern, the polished stone
# families, the forest biomes, and the 1.16 changelog.
set -u
cd "$(dirname "$0")"

pages=(
  Strider
  Piglin
  Hoglin
  Crimson_Stem
  Crimson_Hyphae
  Crimson_Planks
  Crimson_Fungus
  Warped_Fungus
  Crimson_Roots
  Warped_Roots
  Weeping_Vines
  Twisting_Vines
  Nether_Sprouts
  Shroomlight
  Warped_Wart_Block
  Soul_Torch
  Soul_Lantern
  Polished_Basalt
  Polished_Blackstone
  Polished_Blackstone_Bricks
  Crimson_Forest
  Warped_Forest
  Java_Edition_1.16
)

for p in "${pages[@]}"; do
  out="v116b_page_${p}.json"
  if [ -s "$out" ]; then
    echo "skip $p (exists)"
    continue
  fi
  z-ai function -n page_reader \
    -a "{\"url\": \"https://minecraft.wiki/w/${p}\"}" \
    -o "$out" >/dev/null 2>&1 || echo "FAIL $p"
  echo "fetched $p -> $out ($(wc -c < "$out" 2>/dev/null || echo 0) bytes)"
done
