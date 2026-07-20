#!/usr/bin/env bash
# ソルバを seed 掃引して採点集計。
# Usage: scripts/test.sh <bin_name> [num_seeds]
#   例:  scripts/test.sh 01_greedy 100
#
# 前提:
#   ・入力    : tools/in/0000.txt ... (公式generatorで生成しておく)
#   ・採点器  : 下の SCORER を各コンテストに合わせて編集(vis/tester など)。
#              SCORER <in> <out> が "Score = N" 等を出力する想定。
#   ・既定は seed と score のみ(汎用)。手数/誤差/違反数など問題固有の指標を CSV 列に足せば
#     詳細分析できる(その列を results.csv に出し、measure.py の load_results でも読む。詳細は measure.py)。
#   ・AHC_NFINAL=<最終ケース数> を設定すると avg を最終ケース数へ外挿した est を表示。
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="${1:?usage: test.sh <bin_name> [num_seeds]}"
NUM="${2:-100}"
# ★コンテストごとに編集: 採点コマンド。cargo-compete でない公式 tools の採点器を指す。
SCORER="${SCORER:-./tools/target/release/vis}"

cargo build --release --bin "$BIN" >/dev/null 2>&1
SOL="./target/release/$BIN"
mkdir -p out
CSV="results.csv"; echo "seed,score" > "$CSV"   # measure.py が読む per-seed 記録(汎用は seed,score のみ)

total=0; fails=0; worst=999999999; worst_seed=-1; maxms=0
for ((s=0; s<NUM; s++)); do
  in=$(printf "tools/in/%04d.txt" "$s")
  outf=$(printf "out/%04d.txt" "$s")
  [ -f "$in" ] || { echo "missing $in"; continue; }
  t0=$(date +%s%3N)
  "$SOL" < "$in" > "$outf"
  t1=$(date +%s%3N); dt=$((t1-t0)); [ "$dt" -gt "$maxms" ] && maxms=$dt
  line=$("$SCORER" "$in" "$outf" 2>/dev/null || true)
  score=$(echo "$line" | grep -oE '[0-9]+' | tail -1)
  if [ -z "${score:-}" ] || [ "$score" -eq 0 ]; then
    echo "seed $s: FAIL ($line)"; fails=$((fails+1)); continue
  fi
  total=$((total+score))
  if [ "$score" -lt "$worst" ]; then worst=$score; worst_seed=$s; fi
  echo "$s,$score" >> "$CSV"
done

echo "-------------------------------------"
echo "bin          : $BIN"
echo "seeds        : $NUM   fails: $fails"
echo "total score  : $total"
echo "avg score    : $(( total / (NUM>0?NUM:1) ))"
[ -n "${AHC_NFINAL:-}" ] && \
  echo "est($AHC_NFINAL)     : $(( total * AHC_NFINAL / (NUM>0?NUM:1) ))   # 最終ケース数へ外挿(AHC_NFINAL)"
echo "worst        : seed $worst_seed score $worst"
echo "max time(ms) : $maxms"
echo "results.csv  : per-seed (seed,score) → python3 scripts/measure.py"
