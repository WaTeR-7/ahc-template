// 入出力(io)。入力2種と共通出力を1モジュールにまとめる:
//   ・Scanner    : stdin を read_to_string で一括読み(**バッチ問題用**・高速)。対話では EOF 待ちで固まる。
//   ・Interactor : 必要時に1行ずつ読む(**対話/リアクティブ問題用**・入力のみ)。
//   ・Writer     : **共通出力**。整数は手書き itoa、str/char はバイト直書きで core::fmt("{}"解析+Display)を回避。
// 使い方(バッチ): let mut sc=io::Scanner::new(); let mut o=io::Writer::new(); let n:usize=sc.next(); o.putln(ans);
// 使い方(対話)  : let mut it=io::Interactor::new(); let mut o=io::Writer::new(); let x:i32=it.next(); o.putln(q); o.flush();
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
