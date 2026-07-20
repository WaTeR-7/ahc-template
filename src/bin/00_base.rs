#![allow(dead_code)] // 骨格の道具(Scanner/Rng等)は埋めるまで未使用。開発中は付けたままで良い。
// AHC テンプレ ── 単一ファイル・外部クレート非依存(AtCoder提出はこの1ファイルを貼る)。
// 使い方: これを `cp 00_base.rs 01_<approach>.rs` して各アプローチを書く。
//   ・1アプローチ=1ファイル(過去の版を絶対に上書きしない)。
//   ・時間は AHC_TL(ms) で自己制限。ローカルは遅い前提で予算は上限近くに。
//   ・チューナブルは env で(再コンパイル無しに掃引: AHC_TL, AHC_W, ...)。

use std::io::{self, Read, Write};
use std::time::Instant;

// ---------------- fast input ----------------
struct Scanner {
    it: std::str::SplitAsciiWhitespace<'static>,
}
impl Scanner {
    fn new() -> Self {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s).unwrap();
        // プロセスは即終了するので 'static リーク(競プロ定石)で自己参照を回避。
        let s: &'static str = Box::leak(s.into_boxed_str());
        Scanner { it: s.split_ascii_whitespace() }
    }
    fn next<T: std::str::FromStr>(&mut self) -> T {
        self.it.next().unwrap().parse().ok().unwrap()
    }
    fn nexts(&mut self) -> &'static str {
        self.it.next().unwrap()
    }
    fn vec<T: std::str::FromStr>(&mut self, n: usize) -> Vec<T> {
        (0..n).map(|_| self.next()).collect()
    }
}

// ---------------- rng (splitmix64: 高速・種から決定的) ----------------
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    #[inline]
    fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    #[inline]
    fn below(&mut self, n: usize) -> usize {
        (self.u64() % n as u64) as usize
    }
    #[inline]
    fn f64(&mut self) -> f64 {
        (self.u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

// タイマー: 経過ms。deadline との比較で探索を打ち切る。
#[inline]
fn ms(start: Instant) -> u128 {
    start.elapsed().as_millis()
}

fn main() {
    let start = Instant::now();
    // 時間予算(ms)。ジャッジ制限より少し手前に。env で上書き可。
    let tl: u128 = std::env::var("AHC_TL").ok().and_then(|s| s.parse().ok()).unwrap_or(1900);
    let mut sc = Scanner::new();
    let mut rng = Rng::new(0x1234_5678_9ABC_DEF0);

    // ---- 入力 ----
    // let n: usize = sc.next();
    // let a: Vec<i64> = sc.vec(n);

    // ---- 初期解 ----
    // (貪欲/構築でまず制約を満たす実行可能解を作る。ベースラインは常に保持して best-of。)

    // ---- 改善(焼きなまし/ビーム/局所探索。while ms(start) < tl) ----
    // let mut iters = 0u64;
    // while ms(start) < tl {
    //     iters += 1;
    //     // 近傍生成 rng.below(..), 受理判定 rng.f64() < exp(delta/temp) 等
    // }
    // eprintln!("iters={} ms={}", iters, ms(start));

    // ---- 出力 ----
    let out = io::stdout();
    let mut o = io::BufWriter::new(out.lock());
    // writeln!(o, "{}", ans).unwrap();
    o.flush().unwrap();

    // 未使用警告抑制(実装時に削除)
    let _ = (&mut sc, &mut rng, tl, start);
}
