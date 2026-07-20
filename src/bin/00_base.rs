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

    /// 共通出力(バッチ/対話)。**core::fmt を回避**して速い:
    ///   整数=手書き itoa / str・char=バイト直書き / float=put_prec or 既定 {:.12}。
    /// put/putln/sp/nl/put_iter はチェイン可。stdout を一度だけ lock し BufWriter で束ねる
    /// (drop 時に自動 flush。対話は書くたび flush() すること)。
    pub struct Writer {
        w: std::io::BufWriter<std::io::StdoutLock<'static>>,
    }
    impl Writer {
        pub fn new() -> Self {
            Writer { w: std::io::BufWriter::new(std::io::stdout().lock()) }
        }
        /// 値を1つ出力(Wr 実装型。整数/文字列は fmt を経由しない)。
        pub fn put<T: Wr>(&mut self, x: T) -> &mut Self {
            x.wr(&mut self.w);
            self
        }
        /// 値を出力して改行。
        pub fn putln<T: Wr>(&mut self, x: T) -> &mut Self {
            x.wr(&mut self.w);
            self.nl()
        }
        /// 空白。
        pub fn sp(&mut self) -> &mut Self {
            let _ = self.w.write_all(b" ");
            self
        }
        /// 改行。
        pub fn nl(&mut self) -> &mut Self {
            let _ = self.w.write_all(b"\n");
            self
        }
        /// スライス等を sep 区切りで出力(改行は付けない)。
        pub fn put_iter<T: Wr, I: IntoIterator<Item = T>>(&mut self, iter: I, sep: &str) -> &mut Self {
            let mut first = true;
            for x in iter {
                if !first {
                    let _ = self.w.write_all(sep.as_bytes());
                }
                x.wr(&mut self.w);
                first = false;
            }
            self
        }
        /// 小数を prec 桁の固定小数点で(float は fmt 経由。put の既定 float は {:.12})。
        pub fn put_prec<T: Display>(&mut self, x: T, prec: usize) -> &mut Self {
            let _ = write!(self.w, "{:.*}", prec, x);
            self
        }
        /// 任意の Display を fmt 経由で出力するエスケープハッチ(遅い。整数/文字列は put を使う)。
        pub fn put_fmt<T: Display>(&mut self, x: T) -> &mut Self {
            let _ = write!(self.w, "{}", x);
            self
        }
        /// flush(対話では毎回。バッチは drop 時にも自動)。
        pub fn flush(&mut self) -> &mut Self {
            let _ = self.w.flush();
            self
        }
    }

    /// Writer に **fmt を経由せず** 出力できる値(整数=手書き itoa、str/char=バイト直書き、float=fmt)。
    pub trait Wr {
        fn wr<W: Write>(&self, w: &mut W);
    }
    /// 参照は中身へ転送(put_iter(&vec, ..) が &T を渡すため)。
    impl<T: Wr + ?Sized> Wr for &T {
        fn wr<W: Write>(&self, w: &mut W) {
            (**self).wr(w);
        }
    }
    /// 符号なし整数: 末尾から桁をバッファへ書いて一括 write_all。
    macro_rules! wr_uint {
        ($wide:ty, $cap:expr, $($t:ty),*) => {$(
            impl Wr for $t {
                fn wr<W: Write>(&self, w: &mut W) {
                    let mut b = [0u8; $cap];
                    let mut i = b.len();
                    let mut x = *self as $wide;
                    loop { i -= 1; b[i] = b'0' + (x % 10) as u8; x /= 10; if x == 0 { break; } }
                    let _ = w.write_all(&b[i..]);
                }
            }
        )*};
    }
    /// 符号あり整数: unsigned_abs で絶対値化(MIN も安全)してから桁を書く。
    macro_rules! wr_iint {
        ($wide:ty, $cap:expr, $($t:ty),*) => {$(
            impl Wr for $t {
                fn wr<W: Write>(&self, w: &mut W) {
                    let mut b = [0u8; $cap];
                    let mut i = b.len();
                    let neg = *self < 0;
                    let mut x = self.unsigned_abs() as $wide;
                    loop { i -= 1; b[i] = b'0' + (x % 10) as u8; x /= 10; if x == 0 { break; } }
                    if neg { i -= 1; b[i] = b'-'; }
                    let _ = w.write_all(&b[i..]);
                }
            }
        )*};
    }
    wr_uint!(u64, 20, u8, u16, u32, u64, usize);
    wr_iint!(u64, 20, i8, i16, i32, i64, isize);
    wr_uint!(u128, 40, u128);
    wr_iint!(u128, 40, i128);
    impl Wr for str {
        fn wr<W: Write>(&self, w: &mut W) { let _ = w.write_all(self.as_bytes()); }
    }
    impl Wr for String {
        fn wr<W: Write>(&self, w: &mut W) { let _ = w.write_all(self.as_bytes()); }
    }
    impl Wr for char {
        fn wr<W: Write>(&self, w: &mut W) {
            let mut b = [0u8; 4];
            let _ = w.write_all(self.encode_utf8(&mut b).as_bytes());
        }
    }
    /// float は既定で {:.12} 固定小数点(桁数を変えるなら put_prec)。
    macro_rules! wr_float {
        ($($t:ty),*) => {$(
            impl Wr for $t {
                fn wr<W: Write>(&self, w: &mut W) { let _ = write!(w, "{:.12}", self); }
            }
        )*};
    }
    wr_float!(f32, f64);
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

    // ---- 出力(整数/文字列は fmt を経由しない put。対話では書くたび out.flush()) ----
    // out.putln(ans);                    // 1値+改行
    // out.put_iter(&ans_vec, " ").nl();  // 列を空白区切り+改行
    out.flush();

    // 未使用警告抑制(実装時に削除)
    let _ = (&mut sc, &mut rng, &timer, &mut out);
}
