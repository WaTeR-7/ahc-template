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
# 相対評価の回はここから: 大量 seed を並列で回して paired で判定する
scripts/mass.sh 01_greedy 2000               # mass/01_greedy.csv (score だけ)
python3 scripts/rel.py mass/01_greedy.csv mass/02_newmech.csv   # Δrel と Δlog(両方 +2se で採用)
# 機構を足したら: 無効化した設定で前版と byte 一致することを先に確認する
A_ENV="AHC_NEW=0" scripts/same.sh 02_newmech 01_greedy 20
```

> **フックは初回に trust が要る**: Claude Code は `.claude/settings.json` を、Codex は `.codex/hooks.json` を
> 読み込むが、**内容を確認して trust するまで動かない**(Codex は `/hooks`)。定義を変えたときも同じ。
> PDF 生成は `md2pdf` が PATH に無ければ**黙って何もしない**(フック自体は常に成功する)。

> **tools は半自動**: 問題ページはログイン+参加登録が必須で機械取得できないが、保存HTMLの中に
> `img.atcoder.jp/<contest>/<token>.zip`(公開CDN・ログイン不要)が埋まっている。tools のDL/展開/build/
> 入力生成は `fetch_tools.sh` が全部やる。

## AIエージェントの前提

このテンプレは **Claude Code(AI エージェント)で回す**前提で組んである。

- **方針**: **方法論・作業ルール・過去の失敗例は `CLAUDE.md`、そのコンテストの記録は `LOG.md`** と
  棲み分ける(両方に書くと必ず片方が古くなる)。AI はセッション開始時に CLAUDE.md を読んで従う。
  人間は README、AI は CLAUDE.md が入口。
- **規約ガイド**: AHC の AI 利用規約は急速に変化する(縮小方向)ので、恒久テンプレに焼き込まない。
  その回の規約を **`problem/ai_guideline.txt` に人が貼る** → `CLAUDE.md §0` がインポートする。
  **未記入なら AI は「規約未読」とだけ返して全停止**(fail-closed で規約違反を防ぐ)。
- **AIに任せる範囲**: 「AI にどこまで任せるか(自動反復の可否など)」は**その回の規約が決める**。
  規約に従い、規約が触れない範囲はユーザーが主導する。規約が AI 利用を禁じるなら、AI は作業しない。

## 中身

| ファイル | 役割 |
|---|---|
| `CLAUDE.md` | AIエージェント向け**方法論・作業ルール**(前提/コード/進め方/検証と計測/記録の規律/git)。§0 で `problem/ai_guideline.txt` をインポートし未記入なら全停止 |
| `AGENTS.md` | **Codex / 他エージェントの入口**。「`CLAUDE.md` を読め」と読む順番とハーネス差分だけを書いた薄いシム。**ルール本文は写さない**(唯一の例外が §0 の規約ゲート ── fail-closed を成立させるため**機構だけ**再掲する。条文は写さない) |
| `problem/ai_guideline.txt` | その回の AI 利用規約を人が貼る(空placeholderを追跡・中身は毎回貼替) |
| `src/bin/00_base.rs` | 基本部品(io/rng/timer)を貼った解答の最小スタート(追加部品は `lib/` からコピペ) |
| `lib/` | 再利用部品を**種類ごと1ファイル**で分割保持(`mod` ブロック)。cargo の lib ターゲット `ahc_lib`(`lib/lib.rs` が include!)で `cargo check`/`test` 可。解答へはコピペ |
| `knowledge/` | **過去コンテストの蓄積**: `contests/`(参加記録) / `methods/`(**大枠の手法＝手持ちの一覧**) / `techniques/`(部品) + 構造カルテ8軸(軸3 は 2 段)と索引。テンプレに同梱されるので**新しいコンテストの開始時に手元にある**。**正本はテンプレ側** ── 回が終わったら書いてここへ戻す |
| `scripts/fetch_tools.sh` | 保存HTML→tools.zip をDL/展開/build/入力生成(人手はページ保存のみ) |
| `scripts/test.sh` | `<bin> [num] [KEY=val ...]` で seed 掃引+採点(SCORER を編集)。`results.meta` の存在＝完了 |
| `scripts/same.sh` | `<binA> <binB> [num]` で**出力の byte 一致**を照合(決定性の自己チェック付き)。新機構は「OFF で前版と一致」を確認してから有効化する |
| `scripts/measure.py` | 特徴量×成績 の相関/バケット(parse/feats を埋める) |
| `scripts/mass.sh` | `<bin> [num\|from:to] [KEY=val]` で**大量 seed を並列**に走らせ score だけ集める(採否の判定用。ms は測らない)。`INDIR=` で入力ディレクトリを差し替え。⚠ **並列度そのものが採否を変える**ので、時間を食うレバーは `JOBS=3` で測り直す |
| `scripts/rel.py` | **相対評価の判定**。`Δrel`(算術)と `Δlog`(対数平均)を paired で出し、`±se`・必要 n・群別分解まで。**両方が +2se で初めて採用** |
| `scripts/fetch_truth.sh` | 🔴 **終了後**に `ahc_standings` の真値(全参加者×全 seed のスコア行列)を取り、**本番と同一の入力を再生成**する(`--dir in_sys` 固定 ── 既定だと `tools/in` を上書きする事故がある) |
| `scripts/rank.py` | 🔴 **終了後に「この版は何位相当か」を直接出す**(推定を挟まない)。`base` を渡すと**版間の比を自分の真値スコアに写した投影順位**も出る |
| `scripts/vis.sh` | 公式 web ビジュアライザをローカルで動かす(seed 切替・ターンスライダー・自前の重ね描き)。`CONTEST=` で回を指定 |
| `scripts/diag.sh` | 計測版(`NNd_*.rs`)の **stderr(診断行)を大量 seed 分あつめる**(mass.sh は score 以外を捨てるので別物)。壁時計ゲートを外して走らせる |
| `scripts/md2pdf-hook.sh` | `docs/*.md` を編集したら PDF を再生成する PostToolUse フック本体(Claude/Codex 両対応) |
| `LOG.md` | **このコンテストの記録**の雛形(用語集 / チェックリスト / 問題 / **設計上の選択 register** / 試したアプローチ / 発見+否定結果の棚 / 現行ベスト+実ジャッジ校正 / 変更履歴)。冒頭に「どの節に何を書くか」の対応表あり。**方法論は `CLAUDE.md` 側**(二重に書かない) |
| `.gitignore` | /target /tools /out /rep /vis /truth と生成 *.html/*.pdf を無視。**`mass/` は生データを無視しつつ分母(`best.csv`/`ub.csv`)だけ追跡**。**problem/ は追跡**(private前提) |
| `.claude/settings.json` | Claude Code の**プロジェクト設定**。`Write\|Edit` の PostToolUse フックで `scripts/md2pdf-hook.sh` を呼ぶ(記録の PDF を自動再生成。非同期) |
| `.codex/hooks.json` | Codex 側の同じフック(`apply_patch` にマッチ)＋ **Stop フックで完了音**。音は `~/.claude/sounds/done.wav` → 無ければ Windows のビープ → 無ければ無音(`\|\| true` で必ず成功する) |
| `.codex/config.toml` | Codex の既定(`approval_policy` / `sandbox_mode=workspace-write` / **`network_access=true`**)。tools と真値の取得に外部通信が要るため |
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
