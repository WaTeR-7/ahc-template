#!/usr/bin/env bash
# **計測版(`NNd_*.rs`)の stderr を大量 seed 分あつめる**。
# Usage: scripts/diag.sh <bin_name> [num_seeds | from:to] [KEY=val ...]
#   例:  scripts/diag.sh 12d_diag 1000 AHC_TL=100000
#
# なぜ mass.sh ではだめか:
#   mass.sh は stderr から `Score = ` の行しか拾わずに捨てる(採否の判定が目的なので)。
#   計測版は `[12d-shape] ...` のような**診断行**を stderr に出すので、それを丸ごと残す必要がある。
#
# 注意:
#   ・**計測版は本体より遅い**ので、壁時計ゲートを外して走らせる(`AHC_TL=100000 AHC_RPTL=100000`)。
#     外さないと「計測のぶんだけ打ち切りが早まった別の解」を測ることになる(CLAUDE.md §4a)。
#   ・**並列なので ms は測らない**(mass.sh と同じ役割分担)。時間は test.sh で。
#   ・tester を通さず**バイナリに入力を直接与える**(この問題の入出力は追記型で、tester なしでも解答は完走する)。
#     ⇒ Score は取れないが、診断行に必要な量は解答自身が出している。
# 出力: diag/<bin>[__<envtag>].txt  各行 = `<seed>\t<stderr の 1 行>`
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="${1:?usage: diag.sh <bin_name> [num_seeds | from:to] [KEY=val ...]}"; shift
FROM=0; TO=1000
if [ $# -gt 0 ] && [[ "$1" =~ ^[0-9]+$ ]]; then TO="$1"; shift
elif [ $# -gt 0 ] && [[ "$1" =~ ^[0-9]+:[0-9]+$ ]]; then FROM="${1%%:*}"; TO="${1##*:}"; shift; fi
ENVS=("$@")
JOBS="${JOBS:-$(( $(nproc) > 2 ? $(nproc) - 1 : 1 ))}"

cargo build --release --bin "$BIN" >/dev/null
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
cp "./target/release/$BIN" "$TMP/bin"      # 走行中のリビルドで差し替わらないよう固定コピー

TAG=""
if [ ${#ENVS[@]} -gt 0 ]; then TAG="__$(IFS=_; echo "${ENVS[*]}" | tr -d ' ')"; fi
if [ "$FROM" -ne 0 ]; then TAG="${TAG}__s${FROM}-${TO}"; fi
mkdir -p diag
OUT="${OUT:-diag/${BIN}${TAG}.txt}"

cat > "$TMP/worker.sh" <<'WORKER'
#!/usr/bin/env bash
s="$1"
in=$(printf "tools/in/%04d.txt" "$s")
[ -f "$in" ] || exit 0
err="$TMPD/err.$s"
if [ -n "$ENVSTR" ]; then env $ENVSTR "$TMPD/bin" < "$in" > /dev/null 2> "$err" || true
else "$TMPD/bin" < "$in" > /dev/null 2> "$err" || true; fi
sed "s/^/$s\t/" "$err"
rm -f "$err"
WORKER
chmod +x "$TMP/worker.sh"

echo "== diag: bin=$BIN env=${ENVS[*]:-none} seeds=$FROM..$((TO-1)) (n=$((TO-FROM))) jobs=$JOBS =="
t0=$(date +%s)
seq "$FROM" $((TO-1)) | TMPD="$TMP" ENVSTR="${ENVS[*]:-}" \
  xargs -P "$JOBS" -n 1 "$TMP/worker.sh" > "$TMP/raw.txt"
t1=$(date +%s)
sort -n -k1,1 "$TMP/raw.txt" > "$OUT"
echo "wall         : $((t1-t0))s (並列 $JOBS ⇒ per-seed ms は測っていない)"
echo "out          : $OUT  ($(wc -l < "$OUT") 行)"
