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
# vis 出力に合わせ scripts/test.sh の SCORER を確認, LOG §2 を記入
cp src/bin/00_base.rs src/bin/01_greedy.rs   # 1アプローチ=1ファイル(必要な部品は lib/ からコピペ)
cargo run --release --bin 01_greedy < tools/in/0000.txt > out.txt
scripts/test.sh 01_greedy 100                # seed掃引+採点集計(results.meta の存在＝完了)
python3 scripts/measure.py results.csv       # 赤字の構造を測る
# 機構を足したら: 無効化した設定で前版と byte 一致することを先に確認する
A_ENV="AHC_NEW=0" scripts/same.sh 02_newmech 01_greedy 20
```

> **tools は半自動**: 問題ページはログイン+参加登録が必須で機械取得できないが、保存HTMLの中に
> `img.atcoder.jp/<contest>/<token>.zip`(公開CDN・ログイン不要)が埋まっている。tools のDL/展開/build/
> 入力生成は `fetch_tools.sh` が全部やる。

## AIエージェントの前提

このテンプレは **Claude Code(AI エージェント)で回す**前提で組んである。

- **方針**: AI の作業ルールは **`CLAUDE.md`** に集約(1アプローチ=1ファイル/単一 .rs 提出/測定優先/
  `LOG.md` が背骨…)。AI はセッション開始時にこれを読んで従う。人間は README、AI は CLAUDE.md が入口。
- **規約ガイド**: AHC の AI 利用規約は急速に変化する(縮小方向)ので、恒久テンプレに焼き込まない。
  その回の規約を **`problem/ai_guideline.txt` に人が貼る** → `CLAUDE.md §0` がインポートする。
  **未記入なら AI は「規約未読」とだけ返して全停止**(fail-closed で規約違反を防ぐ)。
- **AIに任せる範囲**: 「AI にどこまで任せるか(自動反復の可否など)」は**その回の規約が決める**。
  規約に従い、規約が触れない範囲はユーザーが主導する。規約が AI 利用を禁じるなら、AI は作業しない。

## 中身

| ファイル | 役割 |
|---|---|
| `CLAUDE.md` | AIエージェント向け作業ルール。§0 で `problem/ai_guideline.txt` をインポートし未記入なら全停止 |
| `problem/ai_guideline.txt` | その回の AI 利用規約を人が貼る(空placeholderを追跡・中身は毎回貼替) |
| `src/bin/00_base.rs` | 基本部品(io/rng/timer)を貼った解答の最小スタート(追加部品は `lib/` からコピペ) |
| `lib/` | 再利用部品を**種類ごと1ファイル**で分割保持(`mod` ブロック)。cargo の lib ターゲット `ahc_lib`(`lib/lib.rs` が include!)で `cargo check`/`test` 可。解答へはコピペ |
| `scripts/fetch_tools.sh` | 保存HTML→tools.zip をDL/展開/build/入力生成(人手はページ保存のみ) |
| `scripts/test.sh` | `<bin> [num] [KEY=val ...]` で seed 掃引+採点(SCORER を編集)。`results.meta` の存在＝完了 |
| `scripts/same.sh` | `<binA> <binB> [num]` で**出力の byte 一致**を照合(決定性の自己チェック付き)。新機構は「OFF で前版と一致」を確認してから有効化する |
| `scripts/measure.py` | 特徴量×成績 の相関/バケット(parse/feats を埋める) |
| `LOG.md` | 継続ログ雛形(§0 用語集 + §1 序盤チェックリスト + **§3 設計上の選択 register** + §7a 否定結果の棚 + §8a 実ジャッジ校正) |
| `.gitignore` | /target /tools /out /rep と生成 *.html/*.pdf を無視。**problem/ は追跡**(private前提) |
| `new.sh` | テンプレ→新コンテスト scaffold(テンプレ側のみ) |
| `LICENSE` | CC0-1.0(パブリックドメイン提供。帰属表示不要) |

## ライセンス

- **本テンプレのオリジナル内容**(コード・スクリプト・雛形・著者自身の解説)は **[CC0-1.0](LICENSE)**
  でパブリックドメインに提供する(**帰属表示不要**・自由利用)。
- ただし CC0 が及ぶのは**著者オリジナルの寄与のみ**。**第三者由来の素材**(他者のブログ/editorial の本文・
  図・コード片)は各元著者の権利のもとにあり、**CC0 の対象外**(勝手に再ライセンスはできない)。
- **contest repo はそのまま public にできない**: `problem/` に AtCoder の問題文(著作物)を追跡するため、
  public 化は**第三者著作物の再配布**になる。よって **contest repo は private**、public はこのテンプレ
  (`ahc-template`)のみ。
- **知見の載せ方**: 他人発のコツ・解法は**自分の言葉で言い換え＋出典を明記**して書く
  (アイデア・技法は著作権対象外だが、**逐語コピーは避ける**)。やむを得ず引用する時は短く・出典明記・
  「CC0対象外」と注記。
