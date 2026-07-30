#!/usr/bin/env bash
# 2つのソルバの**出力が byte 単位で一致するか**を md5 で照合する。
# Usage: scripts/same.sh <binA> <binB> [num_seeds]
#   例:  A_ENV="AHC_NEW=0" scripts/same.sh 05_newmech 04_prev 20
#
# 何のためにあるか:
#   新機構は「無効化した設定(AHC_X=0)で前版と出力が完全一致する」ことを確認してから有効化する。
#   これが通れば、その後の差分は**新機構だけ**に帰属できる ⇒ 大きな書き換えを安全に進められる。
#   逆に、一致しないのに気づかず進めると、性能差の原因が特定できなくなる。
#
# 使い方の型:
#   A_ENV / B_ENV に各側の env を空白区切りで渡す(片側だけでもよい)。
#     A_ENV="AHC_NEW=0" scripts/same.sh 05_newmech 04_prev 20   # 新機構 OFF vs 前版
#
# 決定性の自己チェック:
#   **同じバイナリを2回走らせて出力が違えば、この照合自体が無意味**なので先に検査する。
#   壁時計で内部段を打ち切っていると実際にこうなる(ローカルの負荷で打ち切り位置が変わる)。
#   その場合は打ち切りを「作業量カウンタ」に置き換える(壁時計は最外の TL だけに使う)。
set -euo pipefail
cd "$(dirname "$0")/.."

A="${1:?usage: same.sh <binA> <binB> [num_seeds]}"
B="${2:?usage: same.sh <binA> <binB> [num_seeds]}"
NUM="${3:-20}"
A_ENV="${A_ENV:-}"
B_ENV="${B_ENV:-}"

cargo build --release --bin "$A" --bin "$B" >/dev/null
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
cp "./target/release/$A" "$TMP/a"      # 走行中のリビルドで差し替わらないよう固定コピー
cp "./target/release/$B" "$TMP/b"

run() {  # run <a|b> <env-string> <in> <out>
  if [ -n "$2" ]; then env $2 "$TMP/$1" < "$3" > "$4"    # 意図的に分割展開(KEY=val の列)
  else "$TMP/$1" < "$3" > "$4"; fi
}

first_in=""
for ((s=0; s<NUM; s++)); do
  f=$(printf "tools/in/%04d.txt" "$s"); [ -f "$f" ] && { first_in="$f"; break; }
done
[ -n "$first_in" ] || { echo "no tools/in/*.txt found"; exit 1; }

echo "== 決定性の自己チェック(同一バイナリ2回) =="
for side in a b; do
  ev=$([ "$side" = a ] && echo "$A_ENV" || echo "$B_ENV")
  nm=$([ "$side" = a ] && echo "$A" || echo "$B")
  run "$side" "$ev" "$first_in" "$TMP/d1"
  run "$side" "$ev" "$first_in" "$TMP/d2"
  if cmp -s "$TMP/d1" "$TMP/d2"; then
    echo "  $nm: 決定的 OK"
  else
    echo "  $nm: **非決定的** ── 壁時計での打ち切りを作業量カウンタに置き換えるまで、byte 一致検証は使えない"
    exit 2
  fi
done

echo "== byte 一致の照合 ($A [${A_ENV:-none}] vs $B [${B_ENV:-none}], seeds 0..$((NUM-1))) =="
ok=0; diff=0; miss=0; first_diff=-1
for ((s=0; s<NUM; s++)); do
  in=$(printf "tools/in/%04d.txt" "$s")
  if [ ! -f "$in" ]; then miss=$((miss+1)); continue; fi
  run a "$A_ENV" "$in" "$TMP/oa"
  run b "$B_ENV" "$in" "$TMP/ob"
  if cmp -s "$TMP/oa" "$TMP/ob"; then
    ok=$((ok+1))
  else
    diff=$((diff+1)); [ "$first_diff" -lt 0 ] && first_diff=$s
    echo "  seed $s: DIFF  ($(md5sum < "$TMP/oa" | cut -c1-8) vs $(md5sum < "$TMP/ob" | cut -c1-8), 行数 $(wc -l < "$TMP/oa") vs $(wc -l < "$TMP/ob"))"
  fi
done

echo "-------------------------------------"
echo "OK: $ok  DIFF: $diff  missing: $miss"
if [ "$diff" -eq 0 ]; then
  echo "**完全一致** ⇒ 以後の差分は新機構だけに帰属できる"
else
  echo "**不一致** ⇒ 先に seed $first_diff で原因を特定する(新機構が OFF でも経路を変えていないか)"
  exit 1
fi
