// 対話(リアクティブ)問題用 I/O。stdin を一括読みせず、必要になった時だけ次の行を読む。
//   ・io.rs の Scanner は read_to_string で全部読むため、対話では EOF を待ってデッドロックする。
//   ・★ 出力のたびに flush が必須(溜めると相手に届かず固まる)。putln() は書き込み後に自動 flush。
// 使い方: let mut io = interactive::Io::new(); let x: i32 = io.next(); io.putln(ans);
mod interactive {
    use std::collections::VecDeque;
    use std::io::{self, BufRead, Write};
    pub struct Io {
        reader: io::BufReader<io::Stdin>,
        toks: VecDeque<String>,
        writer: io::Stdout,
    }
    impl Io {
        pub fn new() -> Self {
            Io {
                reader: io::BufReader::new(io::stdin()),
                toks: VecDeque::new(),
                writer: io::stdout(),
            }
        }
        /// 次のトークンを読む(バッファが空なら次の行を stdin から読み込む)。EOF なら panic。
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
        /// 1行出力して即 flush(対話では毎回必須)。
        pub fn putln(&mut self, s: impl std::fmt::Display) {
            writeln!(self.writer, "{}", s).unwrap();
            self.writer.flush().unwrap();
        }
        /// flush せずに書き込む(まとめて出す時。最後に flush() する)。
        pub fn put(&mut self, s: impl std::fmt::Display) {
            write!(self.writer, "{}", s).unwrap();
        }
        pub fn flush(&mut self) {
            self.writer.flush().unwrap();
        }
    }
}
