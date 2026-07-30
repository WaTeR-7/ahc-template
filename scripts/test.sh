#!/usr/bin/env bash
# ソルバを seed 掃引して採点集計。
# Usage: scripts/test.sh <bin_name> [num_seeds] [KEY=val ...]
#   例:  scripts/test.sh 01_greedy 100
#        scripts/test.sh 01_greedy 100 AHC_TL=3000 AHC_MODE=2   # env は解に渡され、サマリに記録される
#
# 前提:
#   ・入力    : tools/in/0000.txt ... (公式generatorで生成しておく)
#   ・採点器  : 下の SCORER を各コンテストに合わせて編集(vis/tester など)。
#              既定は "Score = N" / "Score: N" の行だけを読む。形式が違う時は SCORE_RE を上書き。
#   ・INVALID_BELOW : この値未満のスコアを「無効解」として数え、採点器の出力を警告に出す。
#              **無効解が 0 ではなく小さな正の値になる問題がある**ので、0 判定では検出できない
#              (例: 誤差ペナルティで 400-E 点になる問題 → INVALID_BELOW=401 を指定する)。
#   ・既定の CSV 列は seed,score,ms。手数/誤差/違反数など問題固有の指標を足せば詳細分析できる
#     (その列を results.csv に出し、measure.py の load_results でも読む。詳細は measure.py)。
#   ・AHC_NFINAL=<最終ケース数> を設定すると avg を最終ケース数へ外挿した est を表示。
#
# 出力:
#   ・results.csv  : seed,score,ms を1行/seed(measure.py が読む)
#   ・results.meta : サマリ。**最後に atomic に置かれるので「存在＝掃引完了」の目印**になる
#                    (開始時に消す)。バックグラウンド実行の完了待ちはこのファイルを見る。
#   ・out/NNNN.txt : 各 seed の出力。**残すのは「旧設定と byte 一致」を md5 で検証するため**
#                    (リファクタ/高速化は旧設定で出力が完全一致することを確認してから有効化する)。
#
# 100 seed × 2秒 = 3.5分。フォアグラウンドで待てない場合は
#   nohup scripts/test.sh 01_greedy 100 > sweep.log 2>&1 &
# で投げ、results.meta の出現を待つ。
#
# 注意:
#   ・掃引は**直列**。並列化すると per-seed の ms が信用できなくなる(時間予算の判断が壊れる)。
#   ・比較は**同一 env・同一 seed 集合・同一時間予算**でのみ行う(サマリに両方を記録してあるので必ず照合する)。
#   ・est は最終ケース数への外挿にすぎない。実ジャッジは系統的にずれるので、提出のたびに
#     (est, 実スコア, ローカル maxms, 実 ms) を LOG に積んで乖離を測る(差分は転写されるが絶対値はずれる)。
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="${1:?usage: test.sh <bin_name> [num_seeds] [KEY=val ...]}"; shift
NUM=100
if [ $# -gt 0 ] && [[ "$1" =~ ^[0-9]+$ ]]; then NUM="$1"; shift; fi
ENVS=("$@")   # 残りは KEY=val。再コンパイル無しで掃引するための env チューナブル

# ★コンテストごとに編集: 採点コマンド。cargo-compete でない公式 tools の採点器を指す。
SCORER="${SCORER:-./tools/target/release/vis}"
# スコア行のパターン(BRE、キャプチャ1つ)。**数字を全部拾って最後を取る方式にはしない**:
# 採点器は無効解のとき診断行も出すため、全桁 scrape は複数行の数字を1つに連結して
# 巨大なゴミスコアを作る(実際に掃引1本を丸ごと汚染した)。行を限定して1つだけ読む。
SCORE_RE="${SCORE_RE:-^[Ss]core *[=:] *\([0-9][0-9]*\)}"
INVALID_BELOW="${INVALID_BELOW:-1}"

[ -x "$SCORER" ] || { echo "no scorer: $SCORER  (SCORER=<path> で指定, または scripts/fetch_tools.sh)"; exit 1; }

cargo build --release --bin "$BIN" >/dev/null
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
# **走行中のバイナリ差し替えを防ぐための固定コピー**。掃引の途中で cargo build が走ると
# 途中から別のバイナリを測ることになり、出力が同一でも時間が変わって計測が無効になる。
cp "./target/release/$BIN" "$TMP/bin"
SOL="$TMP/bin"

mkdir -p out
CSV="results.csv"; META="results.meta"
rm -f "$META" "$META.tmp"                  # 古い meta を完了の目印と誤読しないように消す
echo "seed,score,ms" > "$CSV"              # measure.py が読む per-seed 記録

total=0; scored=0; missing=0; failed=0; invalid=0
worst=""; worst_seed=-1; maxms=0
for ((s=0; s<NUM; s++)); do
  in=$(printf "tools/in/%04d.txt" "$s")
  outf=$(printf "out/%04d.txt" "$s")
  if [ ! -f "$in" ]; then echo "seed $s: MISSING $in"; missing=$((missing+1)); continue; fi

  t0=$(date +%s%3N)
  rc=0
  if [ ${#ENVS[@]} -gt 0 ]; then env "${ENVS[@]}" "$SOL" < "$in" > "$outf" || rc=$?
  else "$SOL" < "$in" > "$outf" || rc=$?; fi
  t1=$(date +%s%3N); dt=$((t1-t0))
  if [ "$dt" -gt "$maxms" ]; then maxms=$dt; fi
  if [ "$rc" -ne 0 ]; then echo "seed $s: SOLVER EXIT $rc (${dt}ms)"; failed=$((failed+1)); continue; fi

  # 採点器の stderr も受ける(捨てると「なぜ落ちたか」が消える)。診断は先頭160文字だけ出す。
  VOUT=$("$SCORER" "$in" "$outf" 2>&1 || true)
  diag=$(printf '%s' "$VOUT" | tr '\n' ' ' | cut -c1-160)
  score=$(printf '%s\n' "$VOUT" | sed -n "s/$SCORE_RE.*/\1/p" | tail -1)
  if [ -z "$score" ]; then
    echo "seed $s: NO SCORE | $diag"; failed=$((failed+1)); continue
  fi
  score=$((10#$score))                     # 先頭ゼロを8進として誤解釈しない
  # 無効解も total には入れる(ジャッジも数えるので隠さない)。ただし必ず件数と診断を出す。
  if [ "$score" -lt "$INVALID_BELOW" ]; then
    echo "seed $s: INVALID score=$score | $diag"; invalid=$((invalid+1))
  fi

  echo "$s,$score,$dt" >> "$CSV"
  total=$((total+score)); scored=$((scored+1))
  if [ -z "$worst" ] || [ "$score" -lt "$worst" ]; then worst=$score; worst_seed=$s; fi
done

{
  echo "-------------------------------------"
  echo "bin          : $BIN"
  echo "env          : ${ENVS[*]:-none}"
  echo "seeds        : 0..$((NUM-1)) (n=$NUM)   scored: $scored  missing: $missing  failed: $failed"
  echo "INVALID      : $invalid   (score < INVALID_BELOW=$INVALID_BELOW)"
  echo "total score  : $total"
  echo "avg score    : $(( scored > 0 ? total / scored : 0 ))   # 除数は実際に採点できた件数"
  if [ -n "${AHC_NFINAL:-}" ] && [ "$scored" -gt 0 ]; then
    echo "est($AHC_NFINAL)     : $(( total * AHC_NFINAL / scored ))   # 最終ケース数へ外挿(実ジャッジとの乖離は別途校正)"
  fi
  echo "min score    : seed $worst_seed score ${worst:-n/a}   # 最小化問題では最大側が弱点"
  echo "max time(ms) : $maxms"
  echo "results.csv  : per-seed (seed,score,ms) → python3 scripts/measure.py"
} > "$META.tmp"
mv "$META.tmp" "$META"     # 完了の目印は atomic に置く(途中の中断で meta は生まれない)
cat "$META"
