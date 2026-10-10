#!/usr/bin/env python3
"""lib/sprof.rs が PROF_OUT に書いた命令の番地を、addr2line で関数・行に直して集計する。

Usage:
  python3 scripts/sprof.py <debug 情報つきの bin> <src .rs> <番地のファイル>... [--names f1,f2,...] [--top N]

  bin は `CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release --bin <bin> --target-dir <別の場所>` で作る。
  番地のファイルは何件ぶん渡してもよい（標本を合わせて集計する）。

出力は 3 つ:
  1) 自分の関数ごと … 標本の呼び出しの連なり（インライン展開を含む）のうち、<src> の中の関数で数える。
     既定は一番内側。`--names` を渡すと、その名前のうち一番内側のものに寄せる
     （クロージャや小さい補助関数を、段の名前でまとめて読みたいとき）。
  2) 一番内側の関数 … ライブラリ（memcpy・Vec の操作など）を含む。`??` は共有ライブラリ（libm など）の中。
  3) 行ごと … <src> の中の一番内側の行。

標本の間隔はカーネルの刻みで決まる（WSL2 では 1 秒に約 100 個）ので、何十件か走らせてまとめて渡す。

読み方: 細かい区間ごとにタイマーを挟む計測（lib/prof.rs）は軽い段を水増しする。
段の「回数 × 1 回あたり」は prof.rs、内側ループの中の割合はこちらで見る。
"""
import collections
import re
import subprocess
import sys


def main():
    args = sys.argv[1:]
    names, top = [], 40
    if "--names" in args:
        i = args.index("--names")
        names = args[i + 1].split(",")
        del args[i:i + 2]
    if "--top" in args:
        i = args.index("--top")
        top = int(args[i + 1])
        del args[i:i + 2]
    if len(args) < 3:
        sys.exit(__doc__)
    binp, src, files = args[0], args[1], args[2:]

    addrs = []
    for f in files:
        addrs += [l.strip() for l in open(f) if l.strip()]
    if not addrs:
        sys.exit("標本が 0 個（PROF_OUT を付けて走らせたか、区間が短すぎないかを確かめる）")

    # -a: 番地の行（0x…）に続いて (関数, file:line) の組が内側から順に出る。番地が多いので分けて渡す
    out = []
    for k in range(0, len(addrs), 20000):
        r = subprocess.run(["addr2line", "-e", binp, "-f", "-C", "-i", "-a"] + addrs[k:k + 20000],
                           capture_output=True, text=True)
        out += r.stdout.split("\n")
    samples, cur, i = [], None, 0
    while i < len(out):
        l = out[i]
        if l.startswith("0x"):
            cur = []
            samples.append(cur)
            i += 1
        elif cur is not None and i + 1 < len(out):
            cur.append((l, out[i + 1]))
            i += 2
        else:
            i += 1

    n = len(samples)
    srcname = src.split("/")[-1]
    lines = open(src).read().split("\n")

    def short(f):
        f = re.sub(r"<.*?>", "", f)
        return f.split("::")[-1] if "::" in f else f

    byfn, byline, inner = collections.Counter(), collections.Counter(), collections.Counter()
    for fr in samples:
        mine = [(short(f), loc) for f, loc in fr if srcname in loc]
        if mine:
            name = next((f for f, _ in mine if f in names), mine[0][0])
            m = re.search(r":(\d+)", mine[0][1])
            byline[int(m.group(1)) if m else 0] += 1
        else:
            name = "(ソースの外) " + (short(fr[0][0]) if fr else "?")
        byfn[name] += 1
        inner[short(fr[0][0]) if fr else "?"] += 1

    print(f"標本 {n} 個")
    print("== 自分の関数ごと" + ("（--names のうち一番内側）" if names else "（ソースの中で一番内側）"))
    for f, c in byfn.most_common(20):
        print(f"  {100 * c / n:5.1f}%  {f}")
    print("== 一番内側の関数（ライブラリを含む）")
    for f, c in inner.most_common(12):
        print(f"  {100 * c / n:5.1f}%  {f}")
    print(f"== 行ごと（上位 {top}）")
    for ln, c in byline.most_common(top):
        text = lines[ln - 1].strip()[:150] if 0 < ln <= len(lines) else ""
        print(f"  {100 * c / n:5.1f}%  {ln:5d}: {text}")


if __name__ == "__main__":
    main()
