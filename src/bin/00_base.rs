#![allow(dead_code)] // 骨格の道具は埋めるまで未使用。開発中は付けたままで良い。
// AHC 解答の骨格。再利用部品は lib/ に「種類ごと1ファイル」で分割保持し、必要分をコピペして単一 .rs に組む。
//   ・提出はこの1ファイルをそのまま貼る(外部クレート不可)。
//   ・cp src/bin/00_base.rs src/bin/NN_<approach>.rs して各手法を書く(過去版は絶対に上書きしない)。
//   ・部品を足すときは lib/<name>.rs の `mod` ブロックを丸ごと貼る。**lib/ が各部品の正**。
//     下の io/rng/timer は毎回要るので貼り込んだ最小スタート(コピー元は lib/)。
//     ※ io には バッチ入力 Scanner / 対話入力 Interactor / 共通出力 Writer がある。対話問題は Interactor を使う。
//   ・lib/ は cargo の lib ターゲット(ahc_lib)なので `cargo check` / `cargo test` で通常どおり検証できる
//     (解答は use せずコピペ ── 自己完結でそのまま提出でき、その場で改造もできる)。

// ===== lib/io.rs =====
mod io {
    use std::collections::VecDeque;
    use std::fmt::Display;
    use std::io::{BufRead, Read, Write};

    /// バッチ入力: stdin を全部読んでトークン列にする(高速)。対話問題では使わない。
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

    /// 対話入力: 必要になった時だけ次の行を読む(EOF を待って固まらない)。出力は Writer を使う。
    pub struct Interactor {
        reader: std::io::BufReader<std::io::Stdin>,
        toks: VecDeque<String>,
    }
    impl Interactor {
        pub fn new() -> Self {
            Interactor { reader: std::io::BufReader::new(std::io::stdin()), toks: VecDeque::new() }
        }
        pub fn next<T: std::str::FromStr>(&mut self) -> T {
            loop {
                if let Some(t) = self.toks.pop_front() {
                    return t.parse().ok().expect("parse error");
                }
                let mut line = String::new();
                if self.reader.read_line(&mut line).expect("read error") == 0 {
                    panic!("EOF: 対話相手からの入力が尽きた");
                }
                self.toks.extend(line.split_whitespace().map(str::to_owned));
            }
        }
        pub fn vec<T: std::str::FromStr>(&mut self, n: usize) -> Vec<T> {
            (0..n).map(|_| self.next()).collect()
        }
    }

    /// 共通出力: バッチも対話も使える。対話では書くたびに flush() すること。
    pub struct Writer {
        w: std::io::BufWriter<std::io::StdoutLock<'static>>,
    }
    impl Writer {
        pub fn new() -> Self {
            // stdout を一度だけ lock(書き込みごとの再ロック/二重バッファを避ける。StdoutLock は 'static)。
            Writer { w: std::io::BufWriter::new(std::io::stdout().lock()) }
        }
        pub fn print(&mut self, s: impl Display) {
            write!(self.w, "{}", s).unwrap();
        }
        pub fn println(&mut self, s: impl Display) {
            writeln!(self.w, "{}", s).unwrap();
        }
        pub fn flush(&mut self) {
            self.w.flush().unwrap();
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
    let mut sc = io::Scanner::new(); // 対話問題なら io::Interactor::new()
    let mut out = io::Writer::new();
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

    // ---- 出力(対話では書くたびに out.flush()) ----
    // out.println(ans);
    out.flush();

    // 未使用警告抑制(実装時に削除)
    let _ = (&mut sc, &mut rng, &timer, &mut out);
}
