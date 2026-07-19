# ~/ahc — AHC(ヒューリスティック)コンテスト用

ABC(cargo-compete)とはワークフローが別なので分離。**コンテストごとに独立 git リポジトリ**。

## 使い方

```sh
~/ahc/template/new.sh ahc069     # ~/ahc/ahc069 を作り git init
cd ~/ahc/ahc069
# 公式tools を tools/ に展開 → tools/in を生成 → scripts/test.sh の SCORER 編集
cp src/bin/00_base.rs src/bin/01_greedy.rs   # 1アプローチ=1ファイル
cargo run --release --bin 01_greedy < tools/in/0000.txt > out.txt
scripts/test.sh 01_greedy 100                # seed掃引+採点集計
python3 scripts/measure.py results.csv       # 赤字の構造を測る
```

## 方針(なぜこの構成か)

- **提出は単一 .rs**(外部クレート不可) → 共通コードは「リンクするlib」でなく **00_base.rs に内蔵しコピペ**。
- **1アプローチ=1ファイル**(NN_name.rs) → 動く版を絶対に失わない。
- **LOG.md** が背骨。§0 序盤チェックリスト(測定優先・構造から考える)を毎回踏む。
- **独立リポジトリ** → `git add .` が素直、code-review/ultrareview もリポジトリ単位で綺麗。

## 中身

| ファイル | 役割 |
|---|---|
| `src/bin/00_base.rs` | 高速I/O・タイマー(AHC_TL)・splitmix RNG 内蔵の骨格 |
| `scripts/test.sh` | `<bin> [num]` で seed 掃引+採点(SCORER を編集) |
| `scripts/measure.py` | 特徴量×成績 の相関/バケット(parse/feats を埋める) |
| `LOG.md` | 継続ログ雛形(§0チェックリスト付き) |
| `.gitignore` | /target /tools /problem /out /rep *.html |
| `new.sh` | テンプレ→新コンテスト scaffold(テンプレ側のみ) |
