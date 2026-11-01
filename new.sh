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
# テンプレ固有物は contest repo に持ち込まない
#   new.sh: テンプレ側だけに置く / LICENSE(CC0): 問題文=著作物を含む repo に不適 / README: テンプレ自身の説明
rm -f "$dest/new.sh" "$dest/LICENSE" "$dest/README.md" \
      "$dest/results.csv" "$dest/results.meta" "$dest/results.meta.tmp" "$dest/sweep.log"
# git/ビルド/DL/キャッシュ成果物も持ち込まない
rm -rf "$dest/.git" "$dest/target" "$dest/tools" "$dest/out" "$dest/rep" \
       "$dest/scripts/__pycache__"
# パッケージ名をコンテストIDに
sed -i "s/^name = \"sol\"/name = \"$id\"/" "$dest/Cargo.toml" 2>/dev/null || true
# NOW のタイトル雛形
for f in NOW SETUP EVIDENCE VERSIONS HISTORY; do
  sed -i "s/AHC<XXX>/${id^^}/" "$dest/docs/$f.md" 2>/dev/null || true
done

cd "$dest"
git init -q
git add .
git commit -q -m "$id: scaffold from AHC template"

echo "created $dest  (独立 git リポジトリ)"
echo "※ この repo は private 前提(problem/ に AtCoder 問題文=著作物を追跡)。public 化しないこと。"
echo "next:"
echo "  0) 部門制の回は、コンテストページの『部門』タブで部門を選ぶ(With AI は AI を使う前に)"
echo "  1) 本コンテストの AI 利用規約を $dest/problem/ai_guideline.txt に貼り、部門制の回は先頭に『部門: With AI』と書く"
echo "     └ 長期 AHC の規約: https://info.atcoder.jp/entry/long-ahc-llm-rules-ja"
echo "     └ 空・部門の行なしだと AGENTS.md の fail-closed で AI は 1 文だけ返して全停止する"
echo "  2) 問題ページ(要ログイン+参加登録)を『完全な形で保存』→ $dest/problem/ に置く"
echo "  3) cd $dest && scripts/fetch_tools.sh   # tools.zip をDL→展開→build→in生成"
echo "  4) vis 出力に合わせ scripts/test.sh の SCORER を確認, docs/SETUP.md の問題/カルテを記入"
echo "  5) cp src/bin/000_base.rs src/bin/001_<approach>.rs で着手(§1チェックリスト)"
echo "  6) 実行可能解ができたら docs/NOW.md の『設計上の選択』register を埋める"
echo "     └ 行き詰まった時に戻る場所。負の連続は収束ではなく共通の上流前提を疑う合図"
