#!/usr/bin/env python3
"""赤字の構造を測るハーネス(骨格)。AHC で最も効くのは「どのケース群で・なぜ負けているか」を早く測ること。

使い方の型:
  1) test.sh 等で per-seed の (T, score) を results.csv に出す(seed,T,score)。
  2) ここで tools/in/*.txt から各ケースの特徴量(feats)を計算。
  3) join して 特徴量×成績 の相関/バケット別平均を見る → 効果配分を決める。

★ parse() と feats() を各コンテストの入出力形式に合わせて埋める(TODO)。相関/バケットの道具は再利用可。
"""
import sys, glob, os, math

# ---------- 汎用: 相関・バケット(再利用可) ----------
def corr(xs, ys):
    n = len(xs)
    if n == 0: return 0.0
    mx, my = sum(xs)/n, sum(ys)/n
    cov = sum((x-mx)*(y-my) for x, y in zip(xs, ys))
    dx = sum((x-mx)**2 for x in xs) ** 0.5
    dy = sum((y-my)**2 for y in ys) ** 0.5
    return cov/(dx*dy) if dx*dy else 0.0

def buckets(rows, key, edges, keyfn):
    """edges=[(lo,hi,name),...] で rows を key の値でバケット分けし、各バケットの平均を表示用に返す。"""
    out = []
    for lo, hi, name in edges:
        sel = [r for r in rows if lo <= keyfn(r) <= hi]
        if sel:
            out.append((name, len(sel), sel))
    return out

# ---------- TODO: コンテスト依存 ----------
def parse(path):
    """入力ファイルをパース。★形式に合わせて実装。"""
    toks = open(path).read().split()
    it = iter(toks)
    # 例: n = int(next(it)); a = [int(next(it)) for _ in range(n)]
    return {"raw": toks}  # TODO

def feats(parsed):
    """このケースの特徴量(dict)。★何が難易度/赤字を左右しそうかを列挙。
    例(ahc068): 壁密度 W, クリーン列/行数, 最大部屋サイズ, 被覆率 ...
    """
    return {
        # "W": ...,
        # "size": ...,
    }  # TODO

# ---------- 実行 ----------
def load_results(csv_path):
    """seed,T,score 形式(ヘッダ有)を読む。無ければ {} 。"""
    res = {}
    if not os.path.exists(csv_path):
        return res
    for i, line in enumerate(open(csv_path)):
        if i == 0 or not line.strip(): continue
        p = line.strip().split(",")
        res[int(p[0])] = {"T": int(p[1]), "score": int(p[2])}
    return res

def main():
    results = load_results(sys.argv[1] if len(sys.argv) > 1 else "results.csv")
    rows = []
    for f in sorted(glob.glob("tools/in/*.txt")):
        seed = int(os.path.basename(f)[:4])
        m = feats(parse(f))
        m["seed"] = seed
        if seed in results:
            m.update(results[seed])
        rows.append(m)
    if not rows:
        print("no tools/in/*.txt found"); return
    keys = [k for k in rows[0] if k not in ("seed",) and isinstance(rows[0].get(k), (int, float))]
    print(f"cases={len(rows)}  features={[k for k in keys if k not in ('T','score')]}")
    if any("T" in r for r in rows):
        Ts = [r["T"] for r in rows if "T" in r]
        print(f"\n== 特徴量 と T の相関(赤字の在処を掴む) ==")
        for k in keys:
            if k in ("T", "score"): continue
            xs = [r[k] for r in rows if "T" in r and k in r]
            ys = [r["T"] for r in rows if "T" in r and k in r]
            if xs: print(f"  corr(T, {k:12s}) = {corr(xs, ys):+.2f}")
        print(f"\n最難(T上位)ケースの特徴を見て、効果配分を決める:")
        for r in sorted([r for r in rows if 'T' in r], key=lambda r: -r["T"])[:10]:
            fs = " ".join(f"{k}={r[k]}" for k in keys if k not in ("T", "score") and k in r)
            print(f"  seed{r['seed']:4d} T={r['T']:6d} score={r.get('score','?'):>9} | {fs}")
    else:
        print("(results.csv が無い/空: まず test.sh で per-seed の T,score を出す)")

if __name__ == "__main__":
    main()
