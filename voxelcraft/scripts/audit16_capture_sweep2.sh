#!/usr/bin/env bash
# The 1.0-1.16.5 completeness audit — follow-up sweep captures
# (minecraft.wiki, live 2026-09-09): the five food items the independent
# recheck found still unfinalized (rotten flesh value+effect, spider eye
# edibility, chorus fruit, golden apple, melon slice).
set -u
cd "$(dirname "$0")"

pages=(
  Rotten_Flesh
  Spider_Eye
  Chorus_Fruit
  Golden_Apple
  Melon_Slice
  Melon
)

for p in "${pages[@]}"; do
  out="audit16_page_${p}.json"
  if [ -s "$out" ]; then
    echo "skip $p (exists)"
    continue
  fi
  z-ai function -n page_reader \
    -a "{\"url\": \"https://minecraft.wiki/w/${p}\"}" \
    -o "$out" >/dev/null 2>&1 || echo "FAIL $p"
  echo "fetched $p -> $out ($(wc -c < "$out" 2>/dev/null || echo 0) bytes)"
done
