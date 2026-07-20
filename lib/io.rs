// 入出力(io)。入力2種と共通出力を1モジュールにまとめる:
//   ・Scanner    : stdin を read_to_string で一括読み(**バッチ問題用**・高速)。対話では EOF 待ちで固まる。
//   ・Interactor : 必要時に1行ずつ読む(**対話/リアクティブ問題用**・入力のみ)。
//   ・Writer        : **共通出力**(バッチも対話も使える)。対話では書くたびに flush() する。
// 使い方(バッチ): let mut sc=io::Scanner::new(); let mut out=io::Writer::new(); let n:usize=sc.next(); out.println(ans);
// 使い方(対話)  : let mut it=io::Interactor::new(); let mut out=io::Writer::new(); let x:i32=it.next(); out.println(q); out.flush();
mod io {
    use std::collections::VecDeque;
    use std::fmt::Display;
    use std::io::{BufRead, Read, Write}; // トレイト(read_to_string/read_line/write!・flush)用

    /// バッチ入力: stdin を全部読んでトークン列にする(高速)。対話問題では使わない。
    pub struct Scanner {
        it: std::str::SplitAsciiWhitespace<'static>,
    }
    impl Scanner {
        pub fn new() -> Self {
            // プロセス即終了前提で 'static リーク(競プロ定石)で自己参照を回避。
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
        /// 次のトークンを読む(バッファが空なら次行を読み込む)。EOF なら panic。
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

    /// 共通出力: バッチも対話も使える。既定は BufWriter(高速, drop 時にも flush)。
    /// 対話では応答を書くたびに flush() すること(溜めると相手に届かずデッドロック)。
    pub struct Writer {
        w: std::io::BufWriter<std::io::Stdout>,
    }
    impl Writer {
        pub fn new() -> Self {
            Writer { w: std::io::BufWriter::new(std::io::stdout()) }
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
