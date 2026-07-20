// 時間計測。AHC_TL(ms) を締切に探索を打ち切る(ローカルは遅い前提で予算は上限近く)。
// 使い方: let t = timer::Timer::new(); while t.ms() < t.tl { ... }   // tl は env AHC_TL(既定1900)
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
