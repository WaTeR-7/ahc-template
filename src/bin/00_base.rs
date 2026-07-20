#![allow(dead_code)] // 骨格の道具は埋めるまで未使用。開発中は付けたままで良い。
// AHC 解答の骨格。再利用部品は lib/ に「種類ごと1ファイル」で分割保持し、必要分をコピペして単一 .rs に組む。
//   ・提出はこの1ファイルをそのまま貼る(外部クレート不可)。
//   ・cp src/bin/00_base.rs src/bin/NN_<approach>.rs して各手法を書く(過去版は絶対に上書きしない)。
//   ・部品を足すときは lib/<name>.rs の `mod` ブロックを丸ごと貼る。**lib/ が各部品の正**。
//     下の io/rng/timer は毎回要るので貼り込んだ最小スタート(コピー元は lib/)。
//     ※ 対話(リアクティブ)問題は io(バッチ)ではなく lib/interactive.rs を貼る。
//   ・lib/ は cargo の lib ターゲット(ahc_lib)なので `cargo check` / `cargo test` で通常どおり検証できる
//     (解答は use せずコピペ ── 自己完結でそのまま提出でき、その場で改造もできる)。

// ===== lib/io.rs =====
mod io {
    use std::io::Read;
    pub struct Scanner {
        it: std::str::SplitAsciiWhitespace<'static>,
    }
    impl Scanner {
        pub fn new() -> Self {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            let s: &'static str = Box::leak(s.into_boxed_str());
            Scanner { it: s.split_ascii_whitespace() }
        }
        pub fn next<T: std::str::FromStr>(&mut self) -> T {
            self.it.next().unwrap().parse().ok().unwrap()
        }
        pub fn nexts(&mut self) -> &'static str {
            self.it.next().unwrap()
        }
        pub fn vec<T: std::str::FromStr>(&mut self, n: usize) -> Vec<T> {
            (0..n).map(|_| self.next()).collect()
        }
    }
}

// ===== lib/rng.rs =====
mod rng {
    pub struct Rng(pub u64);
    impl Rng {
        pub fn new(seed: u64) -> Self {
            Rng(seed)
        }
        #[inline]
        pub fn u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        #[inline]
        pub fn below(&mut self, n: usize) -> usize {
            (self.u64() % n as u64) as usize
        }
        #[inline]
        pub fn f64(&mut self) -> f64 {
            (self.u64() >> 11) as f64 / (1u64 << 53) as f64
        }
    }
}

// ===== lib/timer.rs =====
mod timer {
    use std::time::Instant;
    pub struct Timer {
        pub start: Instant,
        pub tl: u128,
    }
    impl Timer {
        pub fn new() -> Self {
            let tl = std::env::var("AHC_TL").ok().and_then(|s| s.parse().ok()).unwrap_or(1900);
            Timer { start: Instant::now(), tl }
        }
        #[inline]
        pub fn ms(&self) -> u128 {
            self.start.elapsed().as_millis()
        }
    }
}

fn main() {
    let timer = timer::Timer::new();
    let mut sc = io::Scanner::new();
    let mut rng = rng::Rng::new(0x1234_5678_9ABC_DEF0);

    // ---- 入力 ----
    // let n: usize = sc.next();
    // let a: Vec<i64> = sc.vec(n);

    // ---- 初期解(貪欲/構築でまず制約を満たす1つ。ベースラインは best-of で退行させない) ----

    // ---- 改善(焼きなまし/ビーム/局所探索。while timer.ms() < timer.tl) ----
    // let mut iters = 0u64;
    // while timer.ms() < timer.tl {
    //     iters += 1;
    //     // 近傍生成 rng.below(..), 受理判定 rng.f64() < exp(delta/temp) 等
    // }
    // eprintln!("iters={} ms={}", iters, timer.ms());

    // ---- 出力 ----
    use std::io::Write;
    let out = std::io::stdout();
    let mut o = std::io::BufWriter::new(out.lock());
    // writeln!(o, "{}", ans).unwrap();
    o.flush().unwrap();

    // 未使用警告抑制(実装時に削除)
    let _ = (&mut sc, &mut rng, &timer);
}
