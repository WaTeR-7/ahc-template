# scripts/ — 道具の一覧

**役割で 4 つに分かれる。** 迷ったら「いま測ろうとしているのは*時間*か*スコア*か」を先に決める
（時間は直列で、スコアは並列で測る ── 混ぜると両方が信用できなくなる）。

各スクリプトの先頭に**なぜそれが要るか**を書いてあるので、詳細はそちらを読む。
作法（何を同一条件に保つか・何本の統計で採否を決めるか）は **[`../CLAUDE.md`](../CLAUDE.md)** が正本。

## 1. セットアップ

| script | 使い方 | 何をする |
|---|---|---|
| `fetch_tools.sh` | `scripts/fetch_tools.sh` | 保存した問題ページ HTML から `tools.zip` の CDN URL を拾って **DL / 展開 / build / 入力生成**まで全自動。人手は**ページ保存だけ**。⚠ `patches/tools_*.patch`（自分が tools に当てた改造。正本は `patches/`）を**ビルド前に当て直す** |

## 2. 走らせて測る

| script | 使い方 | 何をする |
|---|---|---|
| `test.sh` | `scripts/test.sh <bin> [num\|from:to] [KEY=val ...]` | **直列**で seed 掃引 + 採点。**per-seed の ms が信用できるのはこちらだけ** ⇒ 時間予算の判断に使う。`results.meta` の存在＝完了。★ 採点コマンド（`SCORER`）は回ごとに要編集。**対話形式の回は tester 経由に差し替える**（先頭のコメント参照） |
| `mass.sh` | `scripts/mass.sh <bin> [num\|from:to] [KEY=val ...]` | **並列**で大量 seed を回し **score だけ**集める（採否の判定用。**ms は測らない**）。🔴 **採点器は対話問題なら `TESTER=<tester>`、非対話問題なら `SCORER=<vis>`**（`test.sh` と同じもの）。`INDIR=` で入力ディレクトリ、`OUT=` で出力先、`JOBS=` で並列数。<br>⚠ **並列度そのものが採否を変える**（per-case の実時間が膨らんで壁時計の弁を人工的に発火させる）⇒ **時間を食うレバーは `JOBS=3` で測り直す** |
| `diag.sh` | `scripts/diag.sh <NNd_bin> [num\|from:to] [KEY=val ...]` | 計測版の **stderr（診断行）を大量 seed 分あつめる**（`mass.sh` は score 以外を捨てるので別物）。**壁時計ゲートを外して**走らせる（外さないと「計測のぶん打ち切りが早まった別の解」を測ることになる） |

## 3. 判定する

| script | 使い方 | 何をする |
|---|---|---|
| `rel.py` | `python3 scripts/rel.py <A.csv> <B.csv> [--seeds a:b] [--group <col>] [--ref <csv>]` | **相対評価の判定**。`Δrel`（算術）と `Δlog`（対数平均）を **paired** で出し、`±se` / 必要 n / 群別分解まで。**両方が +2se で初めて採用**。⚠ **先頭が基準**（逆に置くと符号を読み違える） |
| `measure.py` | `python3 scripts/measure.py [results.csv]` | 赤字の構造を測る（特徴量 × 成績の相関 / バケット）。`parse` と `feats` を回ごとに埋める |
| `same.sh` | `A_ENV="AHC_X=0" scripts/same.sh <新> <前> [num]` | **出力の byte 一致**を照合（同じバイナリを 2 回走らせる決定性の自己チェック付き）。新機構は「**OFF で前版と一致**」を確認してから有効化する。<br>⚠ **核を書き換えるときは byte 一致では通らない** ⇒ assert 同値検査に切り替える（[`../knowledge/techniques/equivalence-by-assert.md`](../knowledge/techniques/equivalence-by-assert.md)） |

## 4. コンテスト終了後（真値で順位を直接測る）

推定を一切挟まずに「**この版は何位相当か**」が出せる。延長戦・反省会の判定はこれ一本でよい。

| script | 使い方 | 何をする |
|---|---|---|
| `fetch_truth.sh` | `scripts/fetch_truth.sh [contest_id]` | `ahc_standings` から **全参加者 × 全 seed のスコア行列**（`result.csv`）と**システムテストの seed**（`input.csv`）を取り、**本番と同一の入力**を `tools/in_sys/` に再生成する。<br>🔴 **`--dir in_sys` 固定**: 公式 `gen` は**行番号**でファイル名を決めるので、既定のままだと `tools/in` を上書きして走行中の掃引ごと汚染する |
| `rank.py` | `AHC_ME=<name> python3 scripts/rank.py <cand.csv> [<base.csv>]` | ①その CSV の相対スコアと**順位** ②`base` を渡すと **per-case 比を自分の真値スコアに写した投影順位**（ローカルはジャッジと速度が違うので、**版間の比だけ**を実測値に写す）。paired `Δlog` も併記 |

## 5. 補助

| script | 使い方 | 何をする |
|---|---|---|
| `vis.sh` | `scripts/vis.sh` / `snap <版名> [seed…]` / `fetch` / `stop` | 公式 web ビジュアライザを**ローカルで**動かす（ディレクトリ選択で seed 切替・ターンスライダー・**自前の重ね描き**）。`CONTEST=` で回を指定（既定は repo のディレクトリ名）。⚠ `vis/` は再配布しない |
| `md2pdf-hook.sh` | （フックから自動実行） | **repo 直下の `*.md` と `docs/*.md`** を編集したら PDF を再生成する PostToolUse フック本体（Claude Code / Codex 両対応）。`md2pdf` が無ければ黙って何もしない |

## 出力先

`results.csv` / `results.meta`（`test.sh`）・`mass/*.csv`（`mass.sh`）・`out/`（解答の出力）・`diag/`（`diag.sh`）・
`truth/`（`fetch_truth.sh`）。**`mass/` は生データを `.gitignore` しつつ、判定の分母（`best.csv` / `ub.csv`）だけ追跡する**
── 分母が消えると過去の採否を再現できなくなるため。
