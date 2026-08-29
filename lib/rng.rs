// 乱数 splitmix64(高速・種から決定的)。焼きなまし/ビームの近傍選択・受理判定に。
// 使い方: let mut rng = rng::Rng::new(seed);
//         rng.below(n);        // [0, n)
//         rng.range(lo, hi);   // [lo, hi)
//         rng.f64();           // [0, 1)
//         rng.shuffle(&mut v); // その場でシャッフル
// ⚠ below/range は剰余なので厳密には一様でない(n が 2^64 に比べて十分小さければ無視できる偏り)。
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
        /// [lo, hi) の一様整数。`lo < hi` であること。
        #[inline]
        pub fn range(&mut self, lo: usize, hi: usize) -> usize {
            lo + self.below(hi - lo)
        }
        #[inline]
        pub fn f64(&mut self) -> f64 {
            (self.u64() >> 11) as f64 / (1u64 << 53) as f64
        }
        /// Fisher-Yates シャッフル(その場)。初期解の多スタートや、候補の走査順をばらすのに。
        pub fn shuffle<T>(&mut self, v: &mut [T]) {
            for i in (1..v.len()).rev() {
                let j = self.below(i + 1);
                v.swap(i, j);
            }
        }
        /// スライスから 1 つ選ぶ(空なら None)。
        #[inline]
        pub fn pick<'a, T>(&mut self, v: &'a [T]) -> Option<&'a T> {
            if v.is_empty() { None } else { Some(&v[self.below(v.len())]) }
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
                let v = r.range(3, 9);
                assert!((3..9).contains(&v));
                let f = r.f64();
                assert!((0.0..1.0).contains(&f));
            }
        }

        #[test]
        fn shuffle_is_a_permutation() {
            let mut r = Rng::new(7);
            for n in 0..30 {
                let mut v: Vec<usize> = (0..n).collect();
                r.shuffle(&mut v);
                let mut sorted = v.clone();
                sorted.sort_unstable();
                assert_eq!(sorted, (0..n).collect::<Vec<_>>()); // 要素の集合は不変
            }
            // 決定的: 同じ種なら同じ並びになる
            let (mut a, mut b) = (Rng::new(9), Rng::new(9));
            let (mut x, mut y): (Vec<usize>, Vec<usize>) = ((0..50).collect(), (0..50).collect());
            a.shuffle(&mut x);
            b.shuffle(&mut y);
            assert_eq!(x, y);
            assert_ne!(x, (0..50).collect::<Vec<_>>()); // 恒等ではない(n=50 なら実質確実)
        }

        #[test]
        fn pick_returns_element_or_none() {
            let mut r = Rng::new(3);
            let empty: [u8; 0] = [];
            assert!(r.pick(&empty).is_none());
            let v = [10, 20, 30];
            for _ in 0..100 {
                assert!(v.contains(r.pick(&v).unwrap()));
            }
        }
    }
}
