#!/usr/bin/env bash
# Claude Code / Codex の PostToolUse フック本体。
#   stdin に来る JSON から編集されたファイルのパスを取り出し、
#   それが `docs/*.md` なら `md2pdf` で同名の PDF を再生成する。
#   Claude の Write/Edit と Codex の apply_patch（複数ファイル可）の
#   両方の入力形式を受け付ける。
#
# 設定は `.claude/settings.json` と `.codex/hooks.json`。単体テストは:
#   echo '{"tool_input":{"file_path":"'$PWD'/docs/SETUP.md"}}' | scripts/md2pdf-hook.sh
#   printf '%s' '{"cwd":"'$PWD'","tool_input":{"command":"*** Begin Patch\n*** Update File: docs/SETUP.md\n*** End Patch"}}' | scripts/md2pdf-hook.sh
#
# ⚠ 注意:
#   - `md2pdf`（~/.local/bin/md2pdf ＝ nvim の <leader>mp と同じ実体）が PATH に無ければ**黙って何もしない**。
#   - **Claude/Codex が編集したときだけ発火する**。自分で nvim から編集した分は従来どおり `<leader>mp`。
#   - フックが失敗してもターンを止めないよう、最後に必ず exit 0 する。
set -uo pipefail

mapfile -d '' -t files < <(python3 -c '
import json
import os
import re
import sys
from pathlib import Path

try:
    d = json.load(sys.stdin)
except Exception:
    sys.exit(0)

tr = d.get("tool_response") or {}
ti = d.get("tool_input") or {}
raw_paths = []

if isinstance(tr, dict):
    raw_paths.extend(tr.get(k) for k in ("filePath", "file_path"))
if isinstance(ti, dict):
    raw_paths.extend(ti.get(k) for k in ("filePath", "file_path"))
    command = ti.get("command")
    if isinstance(command, str):
        for line in command.splitlines():
            match = re.match(r"^\*\*\* (?:Add|Update) File: (.+)$", line)
            if match:
                raw_paths.append(match.group(1))
                continue
            match = re.match(r"^\*\*\* Move to: (.+)$", line)
            if match:
                raw_paths.append(match.group(1))

base = Path(d.get("cwd") or os.getcwd())
seen = set()
for raw_path in raw_paths:
    if not isinstance(raw_path, str) or not raw_path:
        continue
    path = Path(raw_path)
    if not path.is_absolute():
        path = base / path
    path = path.resolve(strict=False)
    value = os.fspath(path)
    if value not in seen:
        seen.add(value)
        sys.stdout.buffer.write(os.fsencode(value) + b"\0")
' 2>/dev/null)

((${#files[@]})) || exit 0
command -v md2pdf >/dev/null 2>&1 || exit 0
repo_root="$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0

for f in "${files[@]}"; do
  case "$f" in
    "$repo_root"/docs/*.md) ;;
    *) continue ;;
  esac
  [ -f "$f" ] || continue
  md2pdf "$f" >/dev/null 2>&1 || true
done
exit 0
