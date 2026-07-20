#!/usr/bin/env python3
"""赤字の構造を測るハーネス(骨格)。AHC で最も効くのは「どのケース群で・なぜ負けているか」を早く測ること。

使い方の型:
  1) test.sh 等で per-seed の score を results.csv に出す(seed,score)。
  2) ここで tools/in/*.txt から各ケースの特徴量(feats)を計算。
  3) join して 特徴量×成績 の相関/バケット別平均を見る → 効果配分を決める。

★ parse() と feats() を各コンテストの入出力形式に合わせて埋める(TODO)。相関/バケットの道具は再利用可。

★★ 詳細分析(拡張): 既定は seed+score のみだが、手数・誤差・違反数など問題固有の指標を足すと分析が深まる。
    ・出力側の指標(手数/誤差など): test.sh の results.csv に列を足し(ヘッダも)、下の load_results で
      その列も読む。数値列は自動で score との相関に載る(例: corr(score, 手数))。
    ・入力側の特徴(サイズ/密度など): feats() に足す。同様に相関/バケットに載る。
    これで「どの入力特徴・どの指標が成績を左右するか」まで踏み込める。
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
    """このケースの特徴量(dict)。★何が難易度/成績を左右しそうかを列挙。
    例: 入力サイズ N, 要素密度, 分散, 制約の厳しさ ...(問題に合わせて)
    """
    return {
        # "N": ...,
        # "density": ...,
    }  # TODO

# ---------- 実行 ----------
def load_results(csv_path):
    """seed,score 形式(ヘッダ有)を読む。無ければ {} 。"""
    res = {}
    if not os.path.exists(csv_path):
        return res
    for i, line in enumerate(open(csv_path)):
        if i == 0 or not line.strip(): continue
        p = line.strip().split(",")
        res[int(p[0])] = {"score": int(p[1])}
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
    print(f"cases={len(rows)}  features={[k for k in keys if k != 'score']}")
    if any("score" in r for r in rows):
        print(f"\n== 特徴量 と score の相関(どの入力特徴が成績を左右するか) ==")
        for k in keys:
            if k == "score": continue
            xs = [r[k] for r in rows if "score" in r and k in r]
            ys = [r["score"] for r in rows if "score" in r and k in r]
            if xs: print(f"  corr(score, {k:12s}) = {corr(xs, ys):+.2f}")
        print(f"\nスコアが低い側のケース(最大化なら弱点。最小化問題なら逆に高い側を見る):")
        for r in sorted([r for r in rows if 'score' in r], key=lambda r: r["score"])[:10]:
            fs = " ".join(f"{k}={r[k]}" for k in keys if k != "score" and k in r)
            print(f"  seed{r['seed']:4d} score={r['score']:>12} | {fs}")
    else:
        print("(results.csv が無い/空: まず test.sh で per-seed の seed,score を出す)")

if __name__ == "__main__":
    main()
