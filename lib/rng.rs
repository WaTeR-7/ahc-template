// 乱数 splitmix64(高速・種から決定的)。焼きなまし/ビームの近傍選択・受理判定に。
// 使い方: let mut rng = rng::Rng::new(seed); rng.below(n); rng.f64();  // f64: [0,1)
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

    // 部品の単体テスト(`cargo test` で走る)。#[cfg(test)] なので提出ファイルに貼っても出力に出ない。
    #[cfg(test)]
    mod tests {
        use super::Rng;
        #[test]
        fn deterministic_and_in_range() {
            let (mut a, mut b) = (Rng::new(42), Rng::new(42));
            for _ in 0..100 {
                assert_eq!(a.u64(), b.u64()); // 同じ種は同じ列(決定的)
            }
            let mut r = Rng::new(1);
            for _ in 0..1000 {
                assert!(r.below(7) < 7);
                let f = r.f64();
                assert!((0.0..1.0).contains(&f));
            }
        }
    }
}
