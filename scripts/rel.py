#!/usr/bin/env python3
"""**相対評価での判定**。版どうしを paired で比べる（算術 rel と 対数平均の2本立て）。

Usage:
  python3 scripts/rel.py <csv> [<csv> ...] [--update-best] [--best mass/best.csv]
                         [--ref mass/ref.csv] [--group <feat.csv の列>] [--top N] [--seeds a:b] [--min|--max]

## 🔴 最大化か最小化か（★回ごとに下の `OBJECTIVE` を編集する。`--min` / `--max` で一時的に上書き）

- **max**: 各ケース `1e9 × 自分 / 全参加者最大`。分母 `D_i` = 最大、`rel_i = score_i / D_i`。
- **min**: 各ケース `1e9 × 全参加者最小 / 自分`（例: AHC072）。分母 `D_i` = 最小、`rel_i = D_i / score_i`。
- どちらでも **Δ は「良くなる向きが正」**に揃える（min では score が下がると Δrel・Δlog が正）。
- `score <= 0` は**無効解**（vis が不正出力に Score = 0 を返す）として扱う: `rel_i = 0`。
  対数は max では 1 に丸め（従来どおり）、min では `INVALID_MIN_SCORE` とみなす（＝ 大敗として数える）。
  分母・best の計算からは除く（0 が「最小」になって分母を壊すのを防ぐ）。

## 何を見るか（以下は max の書き方。min では「自分/最大」を「最小/自分」に読み替える）

真の目的関数は `Σ_i score_i / (全参加者の最大)_i`。分母は観測できないので、**分母の選び方＝seed の重み付けの
選び方**になる。そこで2つの統計を並べて出す:

1. **`rel`（算術）** = `mean_i(score_i / D_i) × 1e9`。`D_i` は `--ref` があればそれ、無ければ
   「与えた版と `best.csv` の max」。**真の目的関数と同じ形**だが、分母の質に依存する。
   - `D_i` = 自分たちのベスト は**動く分母**なので、**`rel` の絶対値を版間・セッション間で比べてはいけない**
     （比べて良いのは同一実行内の `Δrel`）。
2. **`log`（対数平均）** = `exp(mean_i ln score_i)`（＝ 幾何平均）。**paired 差 `Δlog = mean_i ln(a_i/b_i)`
   は分母が完全に消える**（どんな `D_i` を選んでも同じ）ので、**分母を選ばずに済む唯一の統計**。
   各 seed の「何 % 変わったか」を等しく扱う。

**採用の門**: `Δrel > 2se` かつ `Δlog > 2se` を満たしたら採用。**符号が食い違ったら採用しない**
（食い違い＝改善が「弱い seed」か「強い seed」の片方に偏っている、という情報。どちらに偏ったかを
`--group` で分解してから判断する）。検出力の目安: 効果の seed 間 sd を σ とすると `se ≈ σ/√n`
⇒ n=2000, σ=5% なら 0.2% の改善まで見える。
"""
import csv
import math
import os
import sys

# ★回ごとに編集: "max"（自分/全参加者最大）か "min"（全参加者最小/自分）。
OBJECTIVE = "max"
# min で無効解（score <= 0）を対数に入れるときに使う値（どの有効解よりも十分悪い値）
INVALID_MIN_SCORE = 10**9


def load(path):
    d = {}
    with open(path) as f:
        for row in csv.DictReader(f):
            d[int(row["seed"])] = int(row["score"])
    return d


def stats(xs):
    """(mean, se)"""
    n = len(xs)
    mean = sum(xs) / n
    var = sum((x - mean) ** 2 for x in xs) / max(1, n - 1)
    return mean, math.sqrt(var / n)


def main():
    args = list(sys.argv[1:])

    def take(flag, default=None):
        if flag in args:
            i = args.index(flag)
            v = args[i + 1]
            del args[i : i + 2]
            return v
        return default

    objective = OBJECTIVE
    for flag in ("--min", "--max"):
        if flag in args:
            args.remove(flag)
            objective = flag[2:]
    minimize = objective == "min"
    # 良くなる向きを正にする符号（対数の差に掛ける）
    sgn = -1 if minimize else 1

    def valid(v):
        return v > 0

    def ln_score(v):
        if minimize:
            return math.log(v if valid(v) else INVALID_MIN_SCORE)
        return math.log(max(v, 1))

    def rel_of(v, dn):
        if minimize:
            return dn / v if valid(v) else 0.0
        return v / dn

    def better(a, b):
        """a が b より良い（無効解はどの有効解より悪い）"""
        if not valid(a):
            return False
        if not valid(b):
            return True
        return a < b if minimize else a > b

    update_best = "--update-best" in args
    if update_best:
        args.remove("--update-best")
    best_path = take("--best", "mass/best.csv")
    ref_path = take("--ref")
    group = take("--group")
    top = int(take("--top", 10))
    srange = take("--seeds")  # "a:b" (b は含まない) ── dev / holdout を切り分ける
    if not args:
        print(__doc__)
        return

    runs = [(os.path.basename(p).replace(".csv", ""), load(p)) for p in args]
    best = load(best_path) if os.path.exists(best_path) else {}
    ref = load(ref_path) if ref_path and os.path.exists(ref_path) else None

    common = set(runs[0][1])
    for _, d in runs[1:]:
        common &= set(d)
    if ref:
        common &= set(ref)
    if srange:
        a, b = (int(x) for x in srange.split(":"))
        common = {s for s in common if a <= s < b}
    seeds = sorted(common)
    if not seeds:
        print("共通 seed が無い")
        return

    # 分母 D_i（算術 rel 用）。--ref があれば固定分母、無ければ「与えた版 + best.csv」の最良（max / min）
    agg = "min" if minimize else "max"
    if ref:
        den = {s: max(ref[s], 1) for s in seeds}
        den_kind = f"--ref {ref_path}（固定分母）"
    else:
        den = {}
        for s in seeds:
            cand = [d[s] for _, d in runs] + ([best[s]] if s in best else [])
            if minimize:
                ok = [v for v in cand if valid(v)]
                den[s] = min(ok) if ok else 1
            else:
                m = max(cand)
                den[s] = m if m > 0 else 1
        den_kind = f"与えた版 + {best_path} の {agg}（**動く分母** ⇒ 絶対値は比較に使わない）"

    nzero_total = sum(1 for s in seeds for _, d in runs if d[s] <= 0)
    if nzero_total:
        how = f"rel=0、対数では {INVALID_MIN_SCORE:,} とみなす" if minimize else "対数では 1 に丸める"
        print(f"⚠ score<=0 の行が {nzero_total} 件ある（無効解: {how}。**無効解を隠さないこと**）")

    print(f"objective: {objective}（{'全参加者最小/自分' if minimize else '自分/全参加者最大'}。Δ は良くなる向きが正）")
    print(f"seeds: n={len(seeds)}   分母: {den_kind}")
    print(f"{'bin':<26}{'rel(算術, /1e9)':>18}{'log(幾何平均)':>16}{'abs avg':>14}{'best獲得':>9}{'score=0':>8}")
    rel, lg = {}, {}
    for name, d in runs:
        rel[name] = [rel_of(d[s], den[s]) for s in seeds]
        lg[name] = [ln_score(d[s]) for s in seeds]
        nbest = sum(1 for s in seeds if not better(den[s], d[s]) and valid(d[s]))
        nzero = sum(1 for s in seeds if d[s] == 0)
        print(
            f"{name:<26}{sum(rel[name]) / len(seeds) * 1e9:>18,.0f}"
            f"{math.exp(sum(lg[name]) / len(seeds)):>16,.0f}"
            f"{sum(d[s] for s in seeds) / len(seeds):>14,.0f}{nbest:>9}{nzero:>8}"
        )

    if len(runs) >= 2:
        base = runs[0][0]
        print(f"\n=== paired 比較（基準 = {base}）  **両方が +2se で初めて採用** ===")
        for name, _ in runs[1:]:
            dr, sr = stats([(rel[name][i] - rel[base][i]) * 1e9 for i in range(len(seeds))])
            dl, sl = stats([sgn * (lg[name][i] - lg[base][i]) for i in range(len(seeds))])
            win = sum(1 for i in range(len(seeds)) if sgn * (lg[name][i] - lg[base][i]) > 0)
            lose = sum(1 for i in range(len(seeds)) if sgn * (lg[name][i] - lg[base][i]) < 0)
            ok_r, ok_l = dr > 2 * sr, dl > 2 * sl
            bad_r, bad_l = dr < -2 * sr, dl < -2 * sl
            if ok_r and ok_l:
                verdict = "採用可"
            elif bad_r and bad_l:
                verdict = "退行"
            elif (dr > 0) != (dl > 0):
                verdict = "**符号が食い違う**(改善が偏っている ⇒ --group で分解)"
            else:
                verdict = "有意差なし"
            print(f"  {name:<24} Δrel={dr:>+11,.0f}±{sr:<9,.0f}  Δlog={dl * 100:>+7.3f}%±{sl * 100:<6.3f}%  勝{win:>5}/負{lose:>5}  {verdict}")
            # **必要な seed 数**: この効果のばらつき σ から逆算する(件数は「本番と同じ2000」ではなくここで決まる)
            sigma = sl * math.sqrt(len(seeds))
            need = lambda d: (2 * sigma / d) ** 2
            print(
                f"    └ 効果の seed 間 sd σ={sigma * 100:.2f}%  ⇒ 2se≦0.5% には n≈{need(0.005):,.0f} / "
                f"≦0.2% には n≈{need(0.002):,.0f}（現在 n={len(seeds)} で 2se={2 * sl * 100:.3f}%）"
            )

    if group:
        feat = {}
        try:
            with open("mass/feat.csv") as f:
                for row in csv.DictReader(f):
                    feat[int(row["seed"])] = float(row[group])
        except (FileNotFoundError, KeyError):
            print(f"\n[--group {group}] mass/feat.csv が無い/列が無い ⇒ python3 scripts/feat.py で作る")
        gs = [s for s in seeds if s in feat]
        if gs:
            vals = sorted(feat[s] for s in gs)
            qs = [vals[int(len(vals) * k / 4)] for k in (1, 2, 3)]
            labels = [f"{group}<{qs[0]:.3g}", f"{qs[0]:.3g}-{qs[1]:.3g}", f"{qs[1]:.3g}-{qs[2]:.3g}", f">={qs[2]:.3g}"]

            def bucket(s):
                v = feat[s]
                return 0 if v < qs[0] else 1 if v < qs[1] else 2 if v < qs[2] else 3

            idx = {s: i for i, s in enumerate(seeds)}
            print(f"\n=== ケース群別（{group} の4分位）: log(幾何平均) ===")
            print(f"{'群':<22}{'n':>6}" + "".join(f"{nm[:16]:>18}" for nm, _ in runs))
            for b in range(4):
                sub = [s for s in gs if bucket(s) == b]
                if not sub:
                    continue
                line = f"{labels[b]:<22}{len(sub):>6}"
                for nm, _ in runs:
                    line += f"{math.exp(sum(lg[nm][idx[s]] for s in sub) / len(sub)):>18,.0f}"
                print(line)
            if len(runs) >= 2:
                base = runs[0][0]
                print(f"  -- paired Δlog% ± se（基準 {base}）: **レバーが普遍的か case 依存かを見る** --")
                for nm, _ in runs[1:]:
                    line = f"  {nm[:20]:<20}"
                    for b in range(4):
                        sub = [s for s in gs if bucket(s) == b]
                        if not sub:
                            continue
                        m, se = stats([sgn * (lg[nm][idx[s]] - lg[base][idx[s]]) for s in sub])
                        line += f"  {m * 100:>+7.3f}±{se * 100:<6.3f}"
                    print(line)

    name0, d0 = runs[0]
    r0 = dict(zip(seeds, rel[name0]))
    gaps = sorted(seeds, key=lambda s: r0[s])
    print(f"\n=== {name0} が分母に負けている seed (worst {top}) ===")
    for s in gaps[:top]:
        print(f"  seed {s:>5}: {d0[s]:>12,}  vs 分母 {den[s]:>12,}  rel = {r0[s]:.3f}")

    if update_best:
        newbest = dict(best)
        upd = 0
        for s in seeds:
            m = None
            for _, d in runs:
                if m is None or better(d[s], m):
                    m = d[s]
            if not valid(m):
                continue
            if s not in newbest or better(m, newbest[s]):
                newbest[s] = m
                upd += 1
        os.makedirs(os.path.dirname(best_path) or ".", exist_ok=True)
        with open(best_path, "w") as f:
            f.write("seed,score\n")
            for s in sorted(newbest):
                f.write(f"{s},{newbest[s]}\n")
        print(f"\nbest 更新: {best_path} ({upd} seed を更新, 合計 {len(newbest)} seed)")


if __name__ == "__main__":
    main()
