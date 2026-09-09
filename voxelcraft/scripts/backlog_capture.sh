#!/usr/bin/env bash
# The backlog-closure round (2026-09-09): live wiki captures for the
# Part-5 priority backlog — weather, the two missing nether biomes,
# farming, door/bed/TNT, the 1.14 village half, vehicles, jukebox/
# note-block, and the missing-mob roster. Primary source: minecraft.wiki.
set -u
cd "$(dirname "$0")"

pages=(
  Weather
  Soul_Sand_Valley
  Basalt_Deltas
  Farmland
  Wheat_Crops
  Beetroot
  Beetroot_Seeds
  Carrot
  Potato
  Hoe
  Bread
  Door
  Bed
  TNT
  Music_Disc
  Jukebox
  Note_Block
  Boat
  Minecart
  Rail
  Pillager
  Raid
  Wandering_Trader
  Bell
  Ravager
  Slime
  Guardian
  Elder_Guardian
  Shulker
  Endermite
  Wolf
  Cat
  Panda
  Zombified_Piglin
  Zoglin
  Piglin_Brute
  Skeleton_Horse
  Zombie_Horse
  Bad_Omen
  Hero_of_the_Village
  Splash_Potion
  Lingering_Potion
)

for p in "${pages[@]}"; do
  out="backlog_page_${p}.json"
  if [ -s "$out" ]; then
    echo "skip $p (exists)"
    continue
  fi
  z-ai function -n page_reader \
    -a "{\"url\": \"https://minecraft.wiki/w/${p}\"}" \
    -o "$out" >/dev/null 2>&1 || echo "FAIL $p"
  echo "fetched $p -> $out ($(wc -c < "$out" 2>/dev/null || echo 0) bytes)"
done
echo DONE
