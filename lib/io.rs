// 一括入力 Scanner(**バッチ問題用**)。stdin を read_to_string で全部読むので高速。
// ※ 対話(リアクティブ)問題では EOF を待ってデッドロックする → lib/interactive.rs を使う。
// 使い方: let mut sc = io::Scanner::new(); let n: usize = sc.next(); let a: Vec<i64> = sc.vec(n);
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
