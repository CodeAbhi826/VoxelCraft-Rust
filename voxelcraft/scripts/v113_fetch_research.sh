#!/usr/bin/env bash
# 1.13 "Update Aquatic" bracket research captures — pre-implementation.
# Primary source: minecraft.wiki. Cross-checks: minecraft.fandom.com (multi-site
# verification per the standing round protocol).
set -u
cd "$(dirname "$0")"

wiki="https://minecraft.wiki/w"
fandom="https://minecraft.fandom.com/wiki"

# name|url  (primary minecraft.wiki captures)
pages=(
  "changelog|${wiki}/Java_Edition_1.13"
  "drowned|${wiki}/Drowned"
  "phantom|${wiki}/Phantom"
  "dolphin|${wiki}/Dolphin"
  "cod|${wiki}/Cod"
  "salmon|${wiki}/Salmon"
  "pufferfish|${wiki}/Pufferfish"
  "tropical_fish|${wiki}/Tropical_Fish"
  "turtle|${wiki}/Turtle"
  "sea_pickle|${wiki}/Sea_Pickle"
  "coral_block|${wiki}/Coral_Block"
  "dead_coral_block|${wiki}/Dead_Coral_Block"
  "blue_ice|${wiki}/Blue_Ice"
  "kelp|${wiki}/Kelp"
  "dried_kelp_block|${wiki}/Dried_Kelp_Block"
  "conduit|${wiki}/Conduit"
  "heart_of_the_sea|${wiki}/Heart_of_the_Sea"
  "nautilus_shell|${wiki}/Nautilus_Shell"
  "trident|${wiki}/Trident"
  "slow_falling|${wiki}/Slow_Falling"
  "phantom_membrane|${wiki}/Phantom_Membrane"
  "turtle_shell|${wiki}/Turtle_Shell"
  "scute|${wiki}/Scute"
)

# multi-site cross-checks (fandom)
cross=(
  "x_fandom_drowned|${fandom}/Drowned"
  "x_fandom_phantom|${fandom}/Phantom"
  "x_fandom_trident|${fandom}/Trident"
  "x_fandom_113|${fandom}/Update_Aquatic"
)

fetch() { # $1 out-name $2 url
  local out="v113_page_${1%.json}.json"
  if [ -s "${out%.json}_text.txt" ]; then echo "skip ${out} (have text)"; return; fi
  z-ai function -n page_reader -a "{\"url\": \"$2\"}" -o "${out}" 2>/dev/null
  if [ -s "${out%.json}.json" ]; then :; else return; fi
}

for entry in "${pages[@]}"; do
  name="${entry%%|*}"; url="${entry#*|}"
  fetch "$name" "$url"
done
for entry in "${cross[@]}"; do
  name="${entry%%|*}"; url="${entry#*|}"
  fetch "$name" "$url"
done
echo "captures done:"
ls -la v113_page_*.json 2>/dev/null | wc -l
