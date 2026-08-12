#!/usr/bin/env bash
# **web ビジュアライザをローカルで動かす**（AtCoder 配布の wasm をそのまま使う）。
#
# Usage:
#   scripts/vis.sh              # 足りないファイルを取ってきて http://localhost:8000 で配信
#   scripts/vis.sh serve [port]
#   scripts/vis.sh fetch        # vis/app/ を取り直すだけ
#   scripts/vis.sh snap <版名> [seed ...]   # out/ の出力ログを vis/logs/<版名>/ に退避
#   scripts/vis.sh stop
#
# なぜローカルで動かすのか:
#   ・**ディレクトリを選ぶと seed を素早く切り替えられる**（本家と同じ UI）。ターンスライダーも使える。
#   ・`vis(io, t, option)` は wasm の export なので、**JS 側で `ret.svg` に重ね描きできる**
#     （空き 5×5 窓 / dt / repack の blocker など、自分の評価関数が見ているものを可視化する足場）。
#   ・**生成器・採点器はローカルの tools/ と同一**であることを確認済み（seed 2 で入力1行目とスコアが一致）。
#
# ファイルの規約（本家と同じ。外すと seed を手で入れ直すことになる）:
#   ・**入力ファイルは要らない**（wasm の `gen(seed)` が生成する）⇒ 出力ログだけ置けばよい。
#   ・出力ファイル名を `1234.txt` / `abcd_1234.txt` にすると選択時に seed 番号が自動設定される。
#     `out/` と `vis/logs/<版名>/` はこの規約を満たしている（prefix に `_` を入れないこと）。
#
# ⚠ vis/ は .gitignore 済み。**AtCoder の配布物なので再配布しない**（この repo は private 前提）。
set -euo pipefail
cd "$(dirname "$0")/.."

APP=vis/app
# ★コンテスト id。既定はこの repo のディレクトリ名(ahcNNN)。`CONTEST=ahc070 scripts/vis.sh` で上書きできる。
CONTEST="${CONTEST:-$(basename "$PWD")}"
BASE="https://img.atcoder.jp/$CONTEST"
JQ=https://img.atcoder.jp/public/4432a1b/js/lib/jquery-1.9.1.min.js
PORT_DEFAULT=8000

fetch() {
  mkdir -p "$APP"
  # 本家 HTML を**直接**取る。ブラウザの「ページを保存」は拡張機能の注入で 10 倍に膨らむので使わない。
  curl -fsS -o "$APP/Visualizer.html" "$BASE/AdcJXWH4.html?lang=ja"
  for f in AdcJXWH4.js AdcJXWH4_bg.wasm gif.js jszip.min.js; do
    curl -fsS -o "$APP/$f" "$BASE/$f"
  done
  curl -fsS -o "$APP/jquery-1.9.1.min.js" "$JQ"
  # 自分たちの改造を当て直す（**取り直すと消えるので、正本は patches/ にある**）:
  #   ・色分けに v / fee / loss / dens を追加（tools 側と同じ計算。patches/tools_color_modes.patch と対）
  #   ・jquery 参照をローカルへ（オフラインで動かすため）
  if patch -p0 -s < patches/web_vis_color_modes.patch; then
    echo "fetched + patched -> $APP"
  else
    # 本家が更新されて当たらなくなった場合。素の状態でも動くよう jquery だけは直す。
    echo "⚠ patches/web_vis_color_modes.patch が当たらない（本家が更新された？）。色モードの追加は入っていない。" >&2
    sed -i 's|src="//img.atcoder.jp/public/[0-9a-f]*/js/lib/jquery-1.9.1.min.js"|src="./jquery-1.9.1.min.js"|' "$APP/Visualizer.html"
  fi
}

serve() {
  local port="${1:-$PORT_DEFAULT}"
  [ -f "$APP/AdcJXWH4_bg.wasm" ] || fetch
  if curl -s -o /dev/null "http://127.0.0.1:$port/Visualizer.html"; then
    echo "既に配信中: http://localhost:$port/Visualizer.html?lang=ja"; return
  fi
  # file:// では module import が CORS で弾かれるので HTTP 経由が必須。127.0.0.1 のみに bind する。
  nohup python3 -m http.server "$port" --bind 127.0.0.1 --directory "$APP" >/dev/null 2>&1 &
  sleep 1
  echo "http://localhost:$port/Visualizer.html?lang=ja  (WSL2 なら Windows のブラウザからそのまま開ける)"
  echo "出力ログは「ファイルを選択」で **ディレクトリ** を選ぶ: $(pwd)/out もしくは $(pwd)/vis/logs/<版名>"
}

snap() {
  local name="${1:?usage: vis.sh snap <版名> [seed ...]}"; shift
  local dst="vis/logs/$name"; mkdir -p "$dst"
  if [ $# -eq 0 ]; then
    cp out/*.txt "$dst"/
  else
    for s in "$@"; do cp "$(printf 'out/%04d.txt' "$s")" "$dst"/; done
  fi
  echo "$(ls "$dst" | wc -l) files -> $dst"
}

case "${1:-serve}" in
  fetch) fetch ;;
  serve) shift || true; serve "${1:-}" ;;
  snap)  shift; snap "$@" ;;
  stop)  pkill -f "http.server .* --directory $APP" && echo stopped || echo "動いていない" ;;
  *)     echo "usage: vis.sh [serve [port] | fetch | snap <版名> [seed ...] | stop]"; exit 1 ;;
esac
