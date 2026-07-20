# ~/ahc — AHC(ヒューリスティック)コンテスト用

ABC(cargo-compete)とはワークフローが別なので分離。**コンテストごとに独立 git リポジトリ**。

## 初回セットアップ (clone)

新しいマシンでは、このテンプレを `~/ahc/template` に clone しておく(以降 `new.sh` がここを雛形に使う)。

```sh
mkdir -p ~/ahc
git clone https://github.com/WaTeR-7/ahc-template.git ~/ahc/template
```

## 使い方

clone は初回1回だけ。**コンテストごとの開始は `new.sh`** で（テンプレを clone 済み前提）。

```sh
~/ahc/template/new.sh ahc069     # ~/ahc/ahc069 を作り git init
cd ~/ahc/ahc069
# ---- 人手(AIに作業させる前に必ず) ----
#  1) 本コンテストの AI 利用規約を problem/ai_guideline.txt に貼る
#     └ 空だと CLAUDE.md の fail-closed で AI は「規約未読」だけ返して全停止する
#  2) 問題ページ(要ログイン+参加登録)を「完全な形で保存」→ problem/ に置く
scripts/fetch_tools.sh           # 保存HTML内のCDN URLから tools.zip をDL→build→in生成(全自動)
# vis 出力に合わせ scripts/test.sh の SCORER を確認, LOG §1 を記入
cp src/bin/00_base.rs src/bin/01_greedy.rs   # 1アプローチ=1ファイル
cargo run --release --bin 01_greedy < tools/in/0000.txt > out.txt
scripts/test.sh 01_greedy 100                # seed掃引+採点集計
python3 scripts/measure.py results.csv       # 赤字の構造を測る
```

> **AI 利用規約ゲート**: AHC の AI 利用規約は急速に変化する(縮小方向)。恒久テンプレに特定の規約を焼き込まず、
> `problem/ai_guideline.txt` に**その回の規約を人が貼る**設計。`CLAUDE.md` はそれをインポートし、**未記入なら
> AI は一切の作業を拒否**する(fail-closed で規約違反を防ぐ)。
>
> **tools は半自動**: 問題ページはログイン+参加登録が必須で機械取得できないが、保存HTMLの中に
> `img.atcoder.jp/<contest>/<token>.zip`(公開CDN・ログイン不要)が埋まっている。tools のDL/展開/build/
> 入力生成は `fetch_tools.sh` が全部やる。

## 方針(なぜこの構成か)

- **提出は単一 .rs**(外部クレート不可) → 共通コードは「リンクするlib」でなく **00_base.rs に内蔵しコピペ**。
- **1アプローチ=1ファイル**(NN_name.rs) → 動く版を絶対に失わない。
- **LOG.md** が背骨。§0 序盤チェックリスト(測定優先・構造から考える)を毎回踏む。
- **独立リポジトリ** → `git add .` が素直、code-review/ultrareview もリポジトリ単位で綺麗。

## 中身

| ファイル | 役割 |
|---|---|
| `CLAUDE.md` | AIエージェント向け作業ルール。§0 で `problem/ai_guideline.txt` をインポートし未記入なら全停止 |
| `problem/ai_guideline.txt` | その回の AI 利用規約を人が貼る(空placeholderを追跡・中身は毎回貼替) |
| `src/bin/00_base.rs` | 高速I/O・タイマー(AHC_TL)・splitmix RNG 内蔵の骨格 |
| `scripts/fetch_tools.sh` | 保存HTML→tools.zip をDL/展開/build/入力生成(人手はページ保存のみ) |
| `scripts/test.sh` | `<bin> [num]` で seed 掃引+採点(SCORER を編集) |
| `scripts/measure.py` | 特徴量×成績 の相関/バケット(parse/feats を埋める) |
| `LOG.md` | 継続ログ雛形(§0チェックリスト付き) |
| `.gitignore` | /target /tools /problem /out /rep *.html |
| `new.sh` | テンプレ→新コンテスト scaffold(テンプレ側のみ) |
