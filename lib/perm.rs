// 順列(巡回路・訪問順・作業順)の近傍と**差分計算**。TSP 型は AHC 頻出で、
// **2-opt / or-opt の index がほぼ毎回バグる**ので、差分と適用をここに固定して総当たりで検証しておく。
//
// 規約:
//   ・`t` は長さ n の**閉路**。辺は (t[k], t[(k+1)%n]) の n 本(t[n-1] → t[0] を含む)。
//   ・距離は `d(a, b) -> i64` のクロージャで渡す(行列でも計算でもよい)。
//   ・🔴 **返る delta は「コストの増分」= 小さいほど良い。** 焼きなましは最大化前提なので
//     渡すときに符号を反転する ⇒ `ann.accept(-delta as f64, || rng.f64())`。
//   ・🔴 **2-opt と or-opt の rev=true は区間を反転するので、距離が対称でないと delta が合わない**
//     (区間内部の辺の向きが変わるが、この差分式は内部を数えていない)。
//     非対称距離では **or-opt の rev=false だけ**が使える。
//
// 使い方:
//   let mut t: Vec<usize> = (0..n).collect();       // 適当な初期巡回路
//   let cur = perm::tour_len(&t, |a, b| dist[a][b]);
//   // --- 2-opt: 区間 [i, j] を反転 (1 <= i <= j <= n-1) ---
//   let (i, j) = { let (a, b) = (rng.range(1, n), rng.range(1, n)); (a.min(b), a.max(b)) };
//   let delta = perm::two_opt_delta(&t, i, j, |a, b| dist[a][b]);
//   if delta < 0 { perm::apply_two_opt(&mut t, i, j); }
//   // --- or-opt: 長さ l の区間 [i, i+l) を、位置 j の直前へ移す ---
//   let l = rng.range(1, 4);                         // 1..3 要素
//   let i = rng.range(1, n - l + 1);                 // 1 <= i, i+l <= n
//   let j = loop { let j = rng.range(1, n + 1); if j < i || j > i + l { break j; } };
//   let delta = perm::or_opt_delta(&t, i, l, j, rev, |a, b| dist[a][b]);
//   if delta < 0 { perm::apply_or_opt(&mut t, i, l, j, rev); }
//
// ⚠ **t[0] は動かさない**(閉路なので回転しても同じ巡回路。始点を固定しても一般性を失わない)。
//    だから i と j は 1 以上。ここを 0 から回すと同じ解を何度も試して反復が無駄になる。
mod perm {
    /// 閉路の総コスト。**差分が合わなくなったらこれで再計算して突き合わせる**
    /// (knowledge/techniques/equivalence-by-assert)。
    pub fn tour_len(t: &[usize], mut d: impl FnMut(usize, usize) -> i64) -> i64 {
        let n = t.len();
        (0..n).map(|k| d(t[k], t[(k + 1) % n])).sum()
    }

    /// 2-opt: 区間 `[i, j]` を反転したときのコスト増分。`1 <= i <= j <= n-1`。
    /// 反転で消える辺は (t[i-1], t[i]) と (t[j], t[j+1])、生まれる辺は (t[i-1], t[j]) と (t[i], t[j+1])。
    #[inline]
    pub fn two_opt_delta(t: &[usize], i: usize, j: usize, mut d: impl FnMut(usize, usize) -> i64) -> i64 {
        debug_assert!(1 <= i && i <= j && j < t.len(), "2-opt の範囲は 1 <= i <= j <= n-1");
        let n = t.len();
        let (a, b) = (t[i - 1], t[i]); // 区間の手前 / 区間の先頭
        let (c, e) = (t[j], t[(j + 1) % n]); // 区間の末尾 / 区間の直後
        d(a, c) + d(b, e) - d(a, b) - d(c, e)
    }

    /// 2-opt を適用(区間 `[i, j]` を反転)。
    #[inline]
    pub fn apply_two_opt(t: &mut [usize], i: usize, j: usize) {
        debug_assert!(1 <= i && i <= j && j < t.len());
        t[i..=j].reverse();
    }

    /// or-opt: 区間 `[i, i+l)` を抜き出し、**元の位置 `j` の直前**へ挿入したときのコスト増分。
    /// `rev` が真なら区間を反転して挿入する。
    /// 制約: `1 <= i`, `l >= 1`, `i + l <= n`, `1 <= j <= n`, かつ **`j < i` または `j > i+l`**
    ///       (`j == i` と `j == i+l` は「動かさない」ので除く。`j == n` は末尾＝t[0] の直前)。
    #[inline]
    pub fn or_opt_delta(
        t: &[usize],
        i: usize,
        l: usize,
        j: usize,
        rev: bool,
        mut d: impl FnMut(usize, usize) -> i64,
    ) -> i64 {
        let n = t.len();
        debug_assert!(1 <= i && l >= 1 && i + l <= n, "or-opt の区間は 1 <= i, i+l <= n");
        debug_assert!(1 <= j && j <= n && (j < i || j > i + l), "or-opt の挿入位置が区間に重なっている");
        let p = t[i - 1]; // 区間の手前
        let q = t[(i + l) % n]; // 区間の直後
        let u = t[j - 1]; // 挿入先の手前
        let v = t[j % n]; // 挿入先の直後
        let (head, tail) = (t[i], t[i + l - 1]); // 区間の先頭 / 末尾(元の向き)
        let (s, e) = if rev { (tail, head) } else { (head, tail) }; // 挿入後の先頭 / 末尾
        (d(p, q) + d(u, s) + d(e, v)) - (d(p, head) + d(tail, q) + d(u, v))
    }

    /// or-opt を適用。区間の移動は `rotate` 1 回で済む(要素の付け替えを手書きしない)。
    #[inline]
    pub fn apply_or_opt(t: &mut [usize], i: usize, l: usize, j: usize, rev: bool) {
        debug_assert!(1 <= i && l >= 1 && i + l <= t.len());
        debug_assert!(1 <= j && j <= t.len() && (j < i || j > i + l));
        if rev {
            t[i..i + l].reverse();
        }
        if j > i + l {
            t[i..j].rotate_left(l); // [区間, 中間] → [中間, 区間]
        } else {
            t[j..i + l].rotate_right(l); // [中間, 区間] → [区間, 中間]
        }
    }

    /// 位置 `i` と `j` の要素を入れ替えたときのコスト増分。
    /// **隣接(j == i+1)と端の折り返しも正しく扱う**(影響する辺を重複なく数える ── ここが手書きだと落ちる)。
    #[inline]
    pub fn swap_delta(t: &[usize], i: usize, j: usize, mut d: impl FnMut(usize, usize) -> i64) -> i64 {
        let n = t.len();
        debug_assert!(i < n && j < n);
        if i == j {
            return 0;
        }
        // 入れ替え後の並びを「読むだけ」で表す
        let at = |k: usize| -> usize {
            let k = k % n;
            if k == i {
                t[j]
            } else if k == j {
                t[i]
            } else {
                t[k]
            }
        };
        // 影響する辺は最大 4 本(i と j それぞれの前後)。重複を潰してから足す。
        let mut es = [(i + n - 1) % n, i, (j + n - 1) % n, j];
        es.sort_unstable();
        let mut delta = 0;
        let mut prev = usize::MAX;
        for &k in es.iter() {
            if k == prev {
                continue;
            }
            prev = k;
            delta += d(at(k), at(k + 1)) - d(t[k], t[(k + 1) % n]);
        }
        delta
    }

    /// swap を適用。
    #[inline]
    pub fn apply_swap(t: &mut [usize], i: usize, j: usize) {
        t.swap(i, j);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct R(u64);
        impl R {
            fn next(&mut self) -> u64 {
                self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                self.0 >> 33
            }
            fn below(&mut self, n: usize) -> usize {
                (self.next() % n as u64) as usize
            }
        }

        /// 対称な距離行列(2-opt / rev=true が成立する前提)。
        fn sym_matrix(n: usize, r: &mut R) -> Vec<Vec<i64>> {
            let mut m = vec![vec![0i64; n]; n];
            for a in 0..n {
                for b in (a + 1)..n {
                    let w = (r.below(100) + 1) as i64;
                    m[a][b] = w;
                    m[b][a] = w;
                }
            }
            m
        }

        /// 非対称な距離行列(or-opt の rev=false だけが成立する)。
        fn asym_matrix(n: usize, r: &mut R) -> Vec<Vec<i64>> {
            let mut m = vec![vec![0i64; n]; n];
            for a in 0..n {
                for b in 0..n {
                    if a != b {
                        m[a][b] = (r.below(100) + 1) as i64;
                    }
                }
            }
            m
        }

        #[test]
        fn tour_len_counts_the_closing_edge() {
            // 3 点の閉路は 3 辺(0→1→2→0)。開いた経路の 2 辺と混同しない。
            let m = vec![vec![0, 1, 10], vec![1, 0, 100], vec![10, 100, 0]];
            assert_eq!(tour_len(&[0, 1, 2], |a, b| m[a][b]), 1 + 100 + 10);
        }

        #[test]
        fn two_opt_delta_matches_recomputation() {
            let mut r = R(2024);
            for n in 4..9usize {
                for _ in 0..20 {
                    let m = sym_matrix(n, &mut r);
                    let mut t: Vec<usize> = (0..n).collect();
                    for k in (1..n).rev() {
                        t.swap(k, r.below(k + 1));
                    }
                    let before = tour_len(&t, |a, b| m[a][b]);
                    // 全ての合法な (i, j) を総当たり
                    for i in 1..n {
                        for j in i..n {
                            let delta = two_opt_delta(&t, i, j, |a, b| m[a][b]);
                            let mut u = t.clone();
                            apply_two_opt(&mut u, i, j);
                            let after = tour_len(&u, |a, b| m[a][b]);
                            assert_eq!(after - before, delta, "2-opt n={n} i={i} j={j} t={t:?}");
                        }
                    }
                }
            }
        }

        #[test]
        fn or_opt_delta_matches_recomputation() {
            let mut r = R(777);
            for n in 4..9usize {
                for _ in 0..20 {
                    let m = sym_matrix(n, &mut r);
                    let mut t: Vec<usize> = (0..n).collect();
                    for k in (1..n).rev() {
                        t.swap(k, r.below(k + 1));
                    }
                    let before = tour_len(&t, |a, b| m[a][b]);
                    for i in 1..n {
                        for l in 1..=(n - i) {
                            for j in 1..=n {
                                if !(j < i || j > i + l) {
                                    continue;
                                }
                                for &rev in &[false, true] {
                                    let delta = or_opt_delta(&t, i, l, j, rev, |a, b| m[a][b]);
                                    let mut u = t.clone();
                                    apply_or_opt(&mut u, i, l, j, rev);
                                    let after = tour_len(&u, |a, b| m[a][b]);
                                    assert_eq!(
                                        after - before,
                                        delta,
                                        "or-opt n={n} i={i} l={l} j={j} rev={rev} t={t:?}"
                                    );
                                    // 要素は増減しない(rotate の範囲を間違えていない)
                                    let mut s = u.clone();
                                    s.sort_unstable();
                                    assert_eq!(s, (0..n).collect::<Vec<_>>());
                                }
                            }
                        }
                    }
                }
            }
        }

        #[test]
        fn or_opt_without_reverse_works_on_asymmetric_distance() {
            // rev=false は区間の内部の辺の向きを変えないので、非対称距離でも成立する。
            let mut r = R(31337);
            for n in 4..9usize {
                for _ in 0..20 {
                    let m = asym_matrix(n, &mut r);
                    let t: Vec<usize> = (0..n).collect();
                    let before = tour_len(&t, |a, b| m[a][b]);
                    for i in 1..n {
                        for l in 1..=(n - i) {
                            for j in 1..=n {
                                if !(j < i || j > i + l) {
                                    continue;
                                }
                                let delta = or_opt_delta(&t, i, l, j, false, |a, b| m[a][b]);
                                let mut u = t.clone();
                                apply_or_opt(&mut u, i, l, j, false);
                                assert_eq!(tour_len(&u, |a, b| m[a][b]) - before, delta);
                            }
                        }
                    }
                }
            }
        }

        #[test]
        fn swap_delta_matches_recomputation_including_adjacent_and_wrap() {
            let mut r = R(4242);
            for n in 3..9usize {
                for _ in 0..20 {
                    let m = asym_matrix(n, &mut r); // swap は反転しないので非対称でも良い
                    let mut t: Vec<usize> = (0..n).collect();
                    for k in (1..n).rev() {
                        t.swap(k, r.below(k + 1));
                    }
                    let before = tour_len(&t, |a, b| m[a][b]);
                    for i in 0..n {
                        for j in 0..n {
                            let delta = swap_delta(&t, i, j, |a, b| m[a][b]);
                            let mut u = t.clone();
                            apply_swap(&mut u, i, j);
                            let after = tour_len(&u, |a, b| m[a][b]);
                            assert_eq!(after - before, delta, "swap n={n} i={i} j={j} t={t:?}");
                        }
                    }
                }
            }
        }
    }
}
