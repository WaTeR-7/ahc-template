#!/usr/bin/env bash
# このテンプレから新しいAHCコンテストを scaffold する。
# Usage: ~/ahc/template/new.sh ahc069
#   -> ~/ahc/ahc069/ を作り、独立 git リポジトリとして初期化する。
set -euo pipefail

id="${1:?usage: new.sh <contest-id, e.g. ahc069>}"
tdir="$(cd "$(dirname "$0")" && pwd)"
dest="$HOME/ahc/$id"

[ -e "$dest" ] && { echo "error: $dest already exists"; exit 1; }

cp -r "$tdir" "$dest"
rm -f "$dest/new.sh"                      # new.sh はテンプレ側だけに置く
rm -rf "$dest/.git" "$dest/target" \
       "$dest/scripts/__pycache__"        # ビルド/キャッシュ成果物は持ち込まない
# パッケージ名をコンテストIDに
sed -i "s/^name = \"sol\"/name = \"$id\"/" "$dest/Cargo.toml" 2>/dev/null || true
# LOG のタイトル雛形
sed -i "s/AHC<XXX>/${id^^}/" "$dest/LOG.md" 2>/dev/null || true

cd "$dest"
git init -q
git add .
git commit -q -m "$id: scaffold from AHC template"

echo "created $dest  (独立 git リポジトリ)"
echo "※ この repo は private 前提(problem/ に AtCoder 問題文=著作物を追跡)。public 化しないこと。"
echo "next:"
echo "  1) 本コンテストの AI 利用規約を $dest/problem/ai_guideline.txt に貼る"
echo "     └ 空だと CLAUDE.md の fail-closed で AI は『規約未読』とだけ返し全停止する"
echo "  2) 問題ページ(要ログイン+参加登録)を『完全な形で保存』→ $dest/problem/ に置く"
echo "  3) cd $dest && scripts/fetch_tools.sh   # tools.zip をDL→展開→build→in生成"
echo "  4) vis 出力に合わせ scripts/test.sh の SCORER を確認, LOG.md §1 を記入"
echo "  5) cp src/bin/00_base.rs src/bin/01_<approach>.rs で着手(§0チェックリスト)"
