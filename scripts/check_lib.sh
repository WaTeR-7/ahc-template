#!/usr/bin/env bash
# lib/ の各部品(mod ブロック)がコンパイル可能かを型検査する。
# 部品は提出ファイルに「貼るだけ」で cargo build には載らないため、これで腐りを防ぐ。
# 全 lib/*.rs を連結し、1クレートとして rustc で型検査(バイナリは生成しない)。
set -euo pipefail
cd "$(dirname "$0")/.."
shopt -s nullglob
files=(lib/*.rs)
[ ${#files[@]} -gt 0 ] || { echo "lib/ に .rs が無い"; exit 0; }

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
{ echo '#![allow(dead_code, unused)]'; cat "${files[@]}"; } > "$tmp/all.rs"
# 各 mod は crate::<mod>::… で相互参照できる(部品が別部品に依存しても連結でOK)。
if rustc --edition 2024 --crate-type lib --emit=metadata -o "$tmp/out.rmeta" "$tmp/all.rs"; then
  echo "lib OK: ${#files[@]} 部品 — ${files[*]##*/}"
else
  echo "lib COMPILE ERROR (上記)"; exit 1
fi
