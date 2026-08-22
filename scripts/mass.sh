#!/usr/bin/env bash
# **大量 seed の score だけを並列で集める**(相対評価の判定用)。
# Usage: scripts/mass.sh <bin_name> [num_seeds | from:to | seedfile] [KEY=val ...]
#   例:  scripts/mass.sh 01_ff 2000                 # seed 0..1999
#        scripts/mass.sh 01_ff 5000:10000            # **holdout**(最終確認専用の未使用 seed)
#        scripts/mass.sh 02_x 2000 AHC_THR=30        # env はタグとしてファイル名にも入る
#        scripts/mass.sh 02_x run/seeds/c4_dev.txt   # 🔴 **seed リスト**(1行1seed)。下の「帯」を参照
#
# 🔴 **クラス別に設定を変える設計にしたら、掃引の seed も帯で割る**(knowledge/techniques/band-audit.md):
#   作用域の外の seed は**平均 0・分散だけ**を持ち込むので、`t` 値が「1 / 作用域の割合」だけ目減りする
#   ── 全体の 28% にしか効かない帯なら **3.6 倍の損**。seed リストで回せば
#   **同じ壁時計のまま検出力が数倍**になる。
#   ⚠ 帯を入れる前に取った null は偽陰性の可能性がある ⇒ 帯の導入時に過去の null を洗い直す。
#
# なぜ test.sh と別なのか(**相対評価のコンテスト**):
#   ・最終順位は 2000 ケースの**相対スコア**で決まる。**絶対スコア和は seed ごとの ΣV の違い(入力ノイズ)で
#     8.8倍も重みが違う**ので、絶対和で判定すると「ΣV の大きい seed だけ改善する版」を選んでしまう。
#     ⇒ 判定は `scripts/rel.py`(seed ごとに自分たちのベストで割った平均)で行い、そのために**大量 seed**を回す。
#   ・test.sh は**直列**で per-seed の ms を信用できる形に保つ(時間予算の判断に使う)。こちらは**並列**なので
#     **ms は測らない/信用しない**。役割分担: 「rel で採否を決め、test.sh で時間を測る」。
#
# 出力: mass/<bin>[__<envtag>].csv  (seed,score / seed 昇順)。**無効解は score=0 として必ず記録する**
#       (行を落とすと rel.py の seed 集合がずれて比較が壊れる)。
#
# 注意:
#   ・**解が壁時計に依存すると並列実行で結果が変わる**(負荷でターン内の探索量が変わる)。
#     ⚠ **並列度そのものが採否を変える**: 並列数を上げると per-case の実時間が膨らみ、壁時計の弁を人工的に
#     発火させるので**時間を食うレバーだけが不当に負ける**(ahc069: nproc-1 並列で Δlog が −0.74% 目減りした)。
#     ⇒ **時間中立なレバーは既定の並列で、時間を食うレバーは JOBS=3 程度で測り直す。**
#     内部の打ち切りは作業量カウンタにして、壁時計は最外の安全弁だけに使う(CLAUDE.md)。
#   ・JOBS=<n> で並列数を変更(既定 nproc-1)。
#
# 採点器の 2 形式(**どちらのコンテストでも動く**):
#   ・**対話問題**: TESTER=<tester>  … `tester <solver> < in` を実行し、stderr の "Score = N" を読む(既定)。
#   ・**非対話問題**: SCORER=<vis>   … `solver < in > out` の後に `vis <in> <out>` を実行し、
#     その stdout/stderr の "Score = N"(または "Score: N")を読む。**test.sh の SCORER と同じもの**を渡せる。
#   ⚠ どちらか一方を用意すること。両方あるときは SCORER を優先する。
set -euo pipefail
cd "$(dirname "$0")/.."

# 🔴 **掃引の二重起動を止める。** 2 本同時に走ると CPU を食い合い、TL 律速の解が半分しか
#    仕事をしないので **両方の測定が壊れる**。しかも壊れたことは結果の数字に出ない。
#    ⚠ 実際に踏んだ: 掃引の driver を 2 つ持ってしまい、同じ設定を 8 並列で流して 1 本を捨てた。
#    pgrep での自己検出は「自分の親シェル」まで数えてしまうので **flock を使う**。
LOCK="${LOCK:-/tmp/ahc-mass.lock}"
exec 9>"$LOCK"
if ! flock -n 9; then
  echo "⚠ 既に掃引が走っている（$LOCK）。同居させると両方の測定が壊れる。" >&2
  echo "  終わるまで待つ。意図的に同居させるなら LOCK=/tmp/other.lock を指定する。" >&2
  exit 1
fi

BIN="${1:?usage: mass.sh <bin_name> [num_seeds | from:to] [KEY=val ...]}"; shift
# 第2引数は「件数」(0..NUM-1) か **範囲 from:to**(to は含まない)。
# 範囲は dev / holdout の分割に使う: dev=0:5000(調整用) / holdout=5000:10000(**最終確認専用**)。
FROM=0; TO=1000
SEEDFILE=""
if [ $# -gt 0 ] && [[ "$1" =~ ^[0-9]+$ ]]; then TO="$1"; shift
elif [ $# -gt 0 ] && [[ "$1" =~ ^[0-9]+:[0-9]+$ ]]; then FROM="${1%%:*}"; TO="${1##*:}"; shift
elif [ $# -gt 0 ] && [ -f "$1" ]; then SEEDFILE="$1"; shift; fi
ENVS=("$@")

TESTER="${TESTER:-./tools/target/release/tester}"
SCORER="${SCORER:-}"          # 非対話問題はこちら(test.sh と同じ vis 系の採点器)
# 入力ディレクトリ。**本番と同一の 2000 ケース**は `tools/in_sys`(公式 input.csv の seed で生成)。
INDIR="${INDIR:-tools/in}"
JOBS="${JOBS:-$(( $(nproc) > 2 ? $(nproc) - 1 : 1 ))}"
if [ -n "$SCORER" ]; then
  [ -x "$SCORER" ] || { echo "no scorer: $SCORER"; exit 1; }
  MODE=scorer
else
  [ -x "$TESTER" ] || {
    echo "no tester: $TESTER"
    echo "  対話問題なら TESTER=<tester> を、非対話問題なら SCORER=<vis> を指定する(scripts/fetch_tools.sh)。"
    exit 1; }
  MODE=tester
fi

cargo build --release --bin "$BIN" >/dev/null
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
cp "./target/release/$BIN" "$TMP/bin"      # 走行中のリビルドで差し替わらないよう固定コピー

TAG=""
if [ ${#ENVS[@]} -gt 0 ]; then TAG="__$(IFS=_; echo "${ENVS[*]}" | tr -d ' ')"; fi
# 範囲を指定したときは**ファイル名に範囲を入れる**(dev と holdout の csv を混同しないため)
if [ -n "$SEEDFILE" ]; then TAG="${TAG}__$(basename "$SEEDFILE" .txt)"
elif [ "$FROM" -ne 0 ]; then TAG="${TAG}__s${FROM}-${TO}"; fi
mkdir -p mass
# OUT=<path> で出力先を上書きできる(同一設定を2回走らせて再現性を見る時などに使う)
OUT="${OUT:-mass/${BIN}${TAG}.csv}"

# 1 seed 分のワーカ(xargs から並列に呼ぶ)
cat > "$TMP/worker.sh" <<'WORKER'
#!/usr/bin/env bash
s="$1"
in=$(printf "$INDIR/%04d.txt" "$s")
[ -f "$in" ] || { echo "$s,MISSING"; exit 0; }
err="$TMPD/err.$s"
if [ "$MODE" = scorer ]; then
  out="$TMPD/out.$s"
  if [ -n "$ENVSTR" ]; then env $ENVSTR "$TMPD/bin" < "$in" > "$out" 2>/dev/null || true
  else "$TMPD/bin" < "$in" > "$out" 2>/dev/null || true; fi
  "$SCORER" "$in" "$out" > "$err" 2>&1 || true
  rm -f "$out"
else
  if [ -n "$ENVSTR" ]; then env $ENVSTR "$TESTER" "$TMPD/bin" < "$in" > /dev/null 2> "$err" || true
  else "$TESTER" "$TMPD/bin" < "$in" > /dev/null 2> "$err" || true; fi
fi
sc=$(sed -n 's/^Score *[=:] *\([0-9][0-9]*\).*/\1/p' "$err" | tail -1)
echo "$s,${sc:-0}"
rm -f "$err"
WORKER
chmod +x "$TMP/worker.sh"

# seed の供給元。リスト指定なら 1 行 1 seed(空行・`#` コメントは飛ばす)、そうでなければ連番。
if [ -n "$SEEDFILE" ]; then
  grep -vE '^\s*(#|$)' "$SEEDFILE" | tr -d ' \t' > "$TMP/seeds.txt"
  NSEED=$(wc -l < "$TMP/seeds.txt")
  echo "== mass: bin=$BIN env=${ENVS[*]:-none} seeds=$SEEDFILE (n=$NSEED) jobs=$JOBS =="
else
  seq "$FROM" $((TO-1)) > "$TMP/seeds.txt"
  NSEED=$((TO-FROM))
  echo "== mass: bin=$BIN env=${ENVS[*]:-none} seeds=$FROM..$((TO-1)) (n=$NSEED) jobs=$JOBS =="
fi
t0=$(date +%s)
TMPD="$TMP" TESTER="$TESTER" SCORER="$SCORER" MODE="$MODE" INDIR="$INDIR" ENVSTR="${ENVS[*]:-}" \
  xargs -a "$TMP/seeds.txt" -P "$JOBS" -n 1 "$TMP/worker.sh" > "$TMP/raw.csv"
t1=$(date +%s)

miss=$(grep -c ',MISSING$' "$TMP/raw.csv" || true)
zero=$(awk -F, '$2=="0"' "$TMP/raw.csv" | wc -l)
{ echo "seed,score"; grep -v ',MISSING$' "$TMP/raw.csv" | sort -t, -k1,1n; } > "$OUT"
n=$(( $(wc -l < "$OUT") - 1 ))
tot=$(awk -F, 'NR>1{s+=$2} END{print s+0}' "$OUT")
echo "-------------------------------------"
echo "csv          : $OUT   (n=$n, missing=$miss, score=0 の件数=$zero)"
echo "abs total    : $tot   # **判定には使わない**(相対評価なので rel.py を見る)"
echo "abs avg      : $(( n > 0 ? tot / n : 0 ))"
echo "wall         : $((t1-t0))s (並列 $JOBS ⇒ per-seed ms は測っていない)"
echo "次           : python3 scripts/rel.py $OUT   (--update-best でベスト表を更新)"
