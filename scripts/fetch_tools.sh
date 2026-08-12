#!/usr/bin/env bash
# 公式ツール一式(gen/vis)を problem/ の保存HTMLから取得・ビルドする。
#
# 唯一の人手: 問題ページ(ログイン+参加登録が必須で機械取得できない)を
#   ブラウザで「ページを完全な形で保存」→ problem/ に置く(HTML + _files/)。
#   ※保存HTMLの中に img.atcoder.jp の tools.zip URL が埋まっており、その
#     CDN はログイン不要。だからここから先は本スクリプトで全自動。
#
# Usage: scripts/fetch_tools.sh [--force]
#   --force: tools/ を作り直す(再DL)。既定は展開済みならDLを省く。
set -euo pipefail
cd "$(dirname "$0")/.."

force=0
[ "${1:-}" = "--force" ] && force=1

# 保存HTML(vis URL 提示と、必要なら zip URL 抽出に使う)
html=$(ls problem/*.html 2>/dev/null | head -1 || true)

# --- 1. 展開済みか? 未展開/--force なら DL する ---
if [ -f tools/src/bin/gen.rs ] && [ "$force" -eq 0 ]; then
  echo "tools/ 展開済み(--force で再取得)。build と入力だけ確認します。"
else
  [ -n "$html" ] || { echo "error: problem/*.html が無い。問題ページを保存して problem/ に置いてください。"; exit 1; }
  # 保存HTMLから tools.zip URL を抽出(_windows 版は除外)
  zip_url=$(grep -oiE 'https://img\.atcoder\.jp/[A-Za-z0-9_]+/[A-Za-z0-9_]+\.zip' "$html" \
            | grep -iv '_windows' | head -1 || true)
  [ -n "$zip_url" ] || { echo "error: $html から tools.zip URL を抽出できませんでした。"; exit 1; }
  echo "tools.zip: $zip_url"

  tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
  curl -fsSL "$zip_url" -o "$tmp/tools.zip"
  unzip -q "$tmp/tools.zip" -d "$tmp/x"
  # 展開物の中から src/bin/gen.rs を含むフォルダ(=tools ルート)を探して配置
  genrs=$(find "$tmp/x" -type f -path '*/src/bin/gen.rs' | head -1 || true)
  [ -n "$genrs" ] || { echo "error: 展開物に src/bin/gen.rs が見つかりません。"; exit 1; }
  srcroot=$(dirname "$(dirname "$(dirname "$genrs")")")
  rm -rf tools
  mv "$srcroot" tools
  echo "展開 -> tools/"
fi

# --- 1b. 自分たちの改造を当て直す(**ビルドの前**) ---
#   tools/ は gitignore なので**正本は patches/**。取り直すたびにここで当て直す。
#   当たらなくなったら本家が更新された合図 ── その時は patch を作り直す。
for pf in patches/tools_*.patch; do
  [ -e "$pf" ] || continue
  patch -p0 -s < "$pf" \
    && echo "patch  -> $pf" \
    || echo "⚠ $pf が当たらない(本家が更新された？)。この改造は入っていない。" >&2
done

# --- 2. ビルド(gen/vis)。最新なら no-op で速い ---
cargo build --release --manifest-path tools/Cargo.toml
echo "build -> tools/target/release/{gen,vis}"

# --- 3. 入力。seed 0-99 は zip 同梱。無ければ gen で生成 ---
if ! ls tools/in/*.txt >/dev/null 2>&1; then
  ( cd tools && cargo run -r --bin gen seeds.txt )
fi
echo "inputs : $(ls tools/in/*.txt 2>/dev/null | wc -l) 件 (tools/in/)"

# --- 4. Web ビジュアライザ URL を提示(人が目視に使う) ---
vis_url=""
[ -n "$html" ] && vis_url=$(grep -oiE 'https://img\.atcoder\.jp/[A-Za-z0-9_]+/[A-Za-z0-9_]+\.html[^"]*' "$html" \
            | sed 's/&amp;/\&/g' | grep -i 'lang=ja' | grep -iv 'seed=' | head -1 || true)

echo
echo "== 準備完了 =="
echo "  Web ビジュアライザ: ${vis_url:-<HTMLから抽出できず。問題ページで確認>}"
echo "  次にやること:"
echo "    1) 採点確認: tools/target/release/vis <in> <out> の出力行を見て,"
echo "       scripts/test.sh の score 抽出が合っているか確認(vis の 'Score = N' 等)。"
echo "    2) docs/SETUP.md に問題(盤面/制約/スコア式)を要約。"
echo "    3) cp src/bin/00_base.rs src/bin/01_<approach>.rs で着手。"
