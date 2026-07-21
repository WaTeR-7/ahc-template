// Zobrist ハッシュ。`slots` 個の位置 × `vals` 個の値それぞれにランダムな u64 を割当て、
// 状態のハッシュ = 全位置の (位置,値) 乱数の XOR。ビーム/焼きなましの状態 dedup に。
// 差分更新が O(1): 位置 slot の値を a→b に変えるとき h ^= at(slot,a) ^ at(slot,b)。
// 自己完結(rng モジュール不要。種生成に splitmix64 を内蔵)。
// 使い方: let z = zobrist::Zobrist::new(n_cells, n_vals, 0x1234);
//         let mut h = z.hash(&state);              // state[slot] = 値
//         h ^= z.at(slot, old) ^ z.at(slot, new);  // state[slot] を old→new に更新
mod zobrist {
    pub struct Zobrist {
        vals: usize,
        table: Vec<u64>, // table[slot*vals + val]
    }

    impl Zobrist {
        pub fn new(slots: usize, vals: usize, seed: u64) -> Self {
            let mut s = seed ^ 0xD1B5_4A32_D192_ED03;
            let mut table = vec![0u64; slots * vals];
            for x in table.iter_mut() {
                // splitmix64
                s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = s;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                *x = z ^ (z >> 31);
            }
            Zobrist { vals, table }
        }

        /// 位置 `slot` に値 `val` が置かれた寄与(乱数)。差分更新に使う。
        #[inline]
        pub fn at(&self, slot: usize, val: usize) -> u64 {
            self.table[slot * self.vals + val]
        }

        /// `state[slot] = 値` の状態全体のハッシュ(全 slot の XOR)。
        pub fn hash(&self, state: &[usize]) -> u64 {
            let mut h = 0u64;
            for (slot, &v) in state.iter().enumerate() {
                h ^= self.at(slot, v);
            }
            h
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Zobrist;

        #[test]
        fn deterministic_for_same_seed() {
            let a = Zobrist::new(16, 8, 42);
            let b = Zobrist::new(16, 8, 42);
            let st = [0usize, 3, 7, 1, 5, 2, 6, 4];
            assert_eq!(a.hash(&st), b.hash(&st));
            // 種が違えば(ほぼ確実に)ハッシュも違う
            let c = Zobrist::new(16, 8, 43);
            assert_ne!(a.hash(&st), c.hash(&st));
        }

        #[test]
        fn incremental_matches_full() {
            let z = Zobrist::new(10, 10, 7);
            let mut st = vec![0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9];
            let mut h = z.hash(&st);
            // slot 3 の値を 3 -> 9 に変える差分更新が、再計算と一致
            let (slot, old, new) = (3usize, st[3], 9usize);
            h ^= z.at(slot, old) ^ z.at(slot, new);
            st[slot] = new;
            assert_eq!(h, z.hash(&st));
        }

        #[test]
        fn empty_state_is_zero() {
            let z = Zobrist::new(4, 4, 1);
            assert_eq!(z.hash(&[]), 0);
        }
    }
}
