// 時間計測。AHC_TL(ms) を締切に探索を打ち切る(ローカルは遅い前提で予算は上限近く)。
// 使い方: let t = timer::Timer::new(); while t.ms() < t.tl { ... }   // tl は env AHC_TL(既定1900)
//
// **壁時計は「最外の締切」だけに使う。** 内部の重い段を壁時計で打ち切ると、同一バイナリ・同一入力で
// 出力が変わる(=非決定的になる)。そうなると掃引どうしの比較も byte 一致検証(`scripts/same.sh`)も
// 成立しなくなる。内部段は下の Budget(作業量カウンタ)で打ち切る。
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

    /// 決定的な作業量予算。**内部段の打ち切りを壁時計でやると解が非決定的になる**ので、
    /// 「その段が何単位の仕事をするか」を自分で数えて打ち切る。単位は自由(訪問セル数・展開ノード数・
    /// `ops.len()` など、その段のコストに比例する量)。**1度だけ ms へ換算をキャリブレーションする**
    /// (例: 実測 ~195単位/ms なら、1.5秒分は約 300_000 単位)。
    /// 使い方:
    ///   let mut b = timer::Budget::new(env::get("AHC_WORK", 300_000));
    ///   while b.can(cost) { b.spend(cost); /* 重い段を1回 */ }
    pub struct Budget {
        limit: u64,
        spent: u64,
    }
    impl Budget {
        pub fn new(limit: u64) -> Self {
            Budget { limit, spent: 0 }
        }
        /// コスト `w` の仕事を追加しても予算内か。
        #[inline]
        pub fn can(&self, w: u64) -> bool {
            self.spent + w <= self.limit
        }
        #[inline]
        pub fn spend(&mut self, w: u64) {
            self.spent += w;
        }
        #[inline]
        pub fn spent(&self) -> u64 {
            self.spent
        }
        #[inline]
        pub fn left(&self) -> u64 {
            self.limit.saturating_sub(self.spent)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Budget;
        #[test]
        fn budget_is_deterministic_cutoff() {
            let mut b = Budget::new(10);
            assert!(b.can(10) && !b.can(11)); // 境界は「ちょうど使い切る」まで許す
            b.spend(4);
            assert_eq!(b.spent(), 4);
            assert_eq!(b.left(), 6);
            assert!(b.can(6) && !b.can(7));
            b.spend(6);
            assert_eq!(b.left(), 0);
            assert!(!b.can(1));
            b.spend(5); // 使い過ぎても left は 0 に飽和(打ち切り判定は can で行う)
            assert_eq!(b.left(), 0);
        }
    }
}
