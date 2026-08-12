#!/usr/bin/env bash
# **コンテスト終了後に「真値」を取ってきて、本番と同一のケースを再生成する**（延長戦・反省会用）。
#
# Usage: scripts/fetch_truth.sh [contest_id]      # 既定はこの repo のディレクトリ名(ahcNNN)
#
# 取るもの（**ログイン不要**。開催中の回は 403 なので終了後のみ）:
#   truth/result.csv … 全参加者 × 全 seed のスコア行列 ＝ **相対スコアの分母の真値**
#   truth/input.csv  … システムテストの seed 値（`file,seed,...`）
# 作るもの:
#   tools/in_sys/0000.txt … **本番と同一の入力**（`input.csv` の seed を*行順のまま*生成器に渡す）
#
# 🔴 事故に注意: **公式 `gen` は「seeds ファイルの*行番号*」で出力ファイル名を決める**（seed 値ではない）。
#    既定の `--dir in` に出すと **`tools/in/0000.txt` から順に上書き**され、走行中の掃引ごと汚染する。
#    ⇒ ここでは必ず `--dir in_sys` に出す。
#
# 使い方（判定は推定を挟まずに順位で出せる）:
#   INDIR=tools/in_sys OUT=mass/SYS_A.csv scripts/mass.sh <bin> <ケース数>
#   AHC_ME=<ユーザ名> python3 scripts/rank.py mass/SYS_A.csv mass/SYS_base.csv
set -euo pipefail
cd "$(dirname "$0")/.."
C="${1:-$(basename "$PWD")}"
mkdir -p truth
for f in result.csv input.csv; do
  if [ ! -s "truth/$f" ]; then
    echo "== fetch truth/$f ($C)"
    curl -fsS -o "truth/$f" "https://img.atcoder.jp/ahc_standings/$C/$f" \
      || { echo "!! 取得失敗: 終了後の回か、id が正しいか確認する（開催中は 403）"; exit 1; }
  fi
done
N=$(( $(wc -l < truth/input.csv) - 1 ))
echo "== truth: $(wc -c < truth/result.csv) B / システムテスト $N ケース"
[ -x tools/target/release/gen ] || { echo "!! tools/ が未ビルド。scripts/fetch_tools.sh を先に。"; exit 1; }
awk -F, 'NR>1{print $2}' truth/input.csv > tools/sys_seeds.txt
( cd tools && cargo run -r --bin gen sys_seeds.txt --dir in_sys >/dev/null 2>&1 )
echo "== tools/in_sys に $(ls tools/in_sys | wc -l) 件生成した（tools/in は無傷）"
echo "   次: INDIR=tools/in_sys scripts/mass.sh <bin> $N  →  AHC_ME=<name> python3 scripts/rank.py ..."
