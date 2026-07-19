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
echo "next:"
echo "  1) 公式ツールを $dest/tools/ に展開(gitignore済)"
echo "  2) 問題HTML等を $dest/problem/ に(gitignore済)"
echo "  3) generator で tools/in/0000.. を生成、scripts/test.sh の SCORER を編集"
echo "  4) LOG.md の §1問題 を埋める → §0チェックリストから着手"
