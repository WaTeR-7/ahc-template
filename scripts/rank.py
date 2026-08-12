#!/usr/bin/env python3
"""**コンテスト終了後に「この版は何位相当か」を直接出す**（推定を一切挟まない）。

終了後は `img.atcoder.jp/ahc_standings/<contest>/` に
  - `result.csv` … **全参加者 × 全 seed のスコア行列**（1行目は `score_type,base,vis_url`）
  - `input.csv`  … **システムテストの seed 値**（1列目がファイル名、2列目が seed）
が**ログイン不要**で置かれる。⇒ 本番と同一のケースを再生成してローカルで走らせれば、
**相対スコアの分母（per-case max）が真値で分かる**ので、順位がそのまま計算できる。
（取得と入力の再生成は `scripts/fetch_truth.sh`）

Usage:
  python3 scripts/rank.py <cand.csv> [<base.csv>] [--me NAME] [--truth truth/result.csv]

  <cand.csv> / <base.csv> は `mass.sh` の出力（`seed,score`。seed は 0..N-1 ＝ 本番ケースの番号）。
  `--me` は `result.csv` に載っている自分のユーザ名（env `AHC_ME` でも可）。

出るもの:
  ① 与えた CSV そのものの相対スコアと順位
  ② **`base` を与えたら、per-case 比 `cand/base` を*自分の真値スコア*に掛けた投影順位**
     ── ローカルはジャッジより遅い（または速い）ので①は素の値がズレる。
        **版間の比だけをジャッジの実測値に写す**のが②で、投資判断はこちらを使う。

⚠ 判定そのものは paired の `Δlog`（下に出る）で行う。順位は「効果の大きさを順位の単位で見る」ためのもの。
"""
import csv, sys, math, os

args = [a for a in sys.argv[1:] if not a.startswith("--")]
opts = {a.split("=")[0]: a.split("=", 1)[1] for a in sys.argv[1:] if a.startswith("--") and "=" in a}
for i, a in enumerate(sys.argv[1:]):
    if a in ("--me", "--truth") and i + 2 <= len(sys.argv) - 1:
        opts[a] = sys.argv[i + 2]
ME = opts.get("--me") or os.environ.get("AHC_ME")
TRUTH = opts.get("--truth") or os.environ.get("AHC_TRUTH", "truth/result.csv")
if not args or not ME:
    print(__doc__)
    print("!! --me <あなたのユーザ名>（または env AHC_ME）が要る。`result.csv` の1列目に載っている名前。")
    sys.exit(1)

def load_mass(p):
    d = {}
    for r in csv.reader(open(p)):
        if not r or r[0] == "seed":
            continue
        d[int(r[0])] = int(r[1])
    return d

rows = []
with open(TRUTH) as f:
    rd = csv.reader(f)
    next(rd)                                   # score_type,base,vis_url
    for r in rd:
        rows.append((r[0], [int(x) for x in r[1:]]))
m = len(rows[0][1])
env = [0] * m                                  # per-case max = 相対スコアの分母（真値）
for _, sc in rows:
    for j, v in enumerate(sc):
        if v > env[j]:
            env[j] = v
if ME not in dict(rows):
    print(f"!! '{ME}' が {TRUTH} に無い。名前は大文字小文字も一致させる。")
    sys.exit(1)
mine_true = dict(rows)[ME]

def rel_of(scores):                            # 負値・0 は分母に入れない（未提出/不正解の扱い）
    return sum(1e9 * v / env[j] for j, v in enumerate(scores) if v > 0)

others = sorted(((rel_of(sc), nm) for nm, sc in rows if nm != ME), reverse=True)

def rank_of(rel):
    lo, hi = 0, len(others)
    while lo < hi:
        mid = (lo + hi) // 2
        if others[mid][0] > rel:
            lo = mid + 1
        else:
            hi = mid
    return lo + 1

def report(tag, scores):
    r = rel_of(scores)
    rk = rank_of(r)
    print(f"{tag:24s} rel {r:,.0f} ({r/(1e9*m)*100:.3f}%)  rank {rk:4d} / {len(others)+1}  "
          f"abs {sum(scores):,} (mean {sum(scores)/m/1e6:.3f}M)")
    return r, rk

print(f"truth: {ME} は {rank_of(rel_of(mine_true))}位 / rel {rel_of(mine_true):,.0f}  （{TRUTH}, {m} ケース）")
cand = load_mass(args[0])
report("local " + os.path.basename(args[0]), [cand.get(i, 0) for i in range(m)])
if len(args) > 1:
    base = load_mass(args[1])
    report("local " + os.path.basename(args[1]), [base.get(i, 0) for i in range(m)])
    proj, nz, dl = [], 0, []
    for i in range(m):
        b, c = base.get(i, 0), cand.get(i, 0)
        if b > 0 and c > 0:
            proj.append(mine_true[i] * c / b)
            dl.append(math.log(c / b))
        else:
            proj.append(float(mine_true[i]))
            nz += 1
    mu = sum(dl) / len(dl)
    sd = (sum((x - mu) ** 2 for x in dl) / (len(dl) - 1)) ** .5
    se = sd / len(dl) ** .5
    print(f"paired Δlog {mu*100:+.4f}% ± {se*100:.4f}%  ({mu/se:+.2f}se, n={len(dl)}, 比が取れない対 {nz})")
    r, rk = report("PROJECTED(judge)", proj)
    tr = rel_of(mine_true)
    print(f"  → 真値 {rank_of(tr)}位 から {rk}位（{rank_of(tr)-rk:+d}）／ rel {(r/tr-1)*100:+.3f}%")
