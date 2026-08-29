// Union-Find(素集合)。「今つないだら連結か」「グループはいくつ残っているか」を実質 O(1) で答える。
// AHC では**連結性の制約**(分断してはいけない / 1 つに繋げたい)と、貪欲の順序決定に頻出。
// 使い方: let mut d = dsu::Dsu::new(n);
//         if d.unite(a, b) { /* 新しく繋がった(既に同じ組なら false) */ }
//         d.same(a, b); d.size(a); d.groups();
//
// 🔴 **焼きなましで「元に戻す」必要があるなら RollbackDsu を使う。** 経路圧縮した Dsu は巻き戻せない
//    (親ポインタを潰してしまうので、unite を undo しても元の形に復元できない)。RollbackDsu は
//    経路圧縮を捨てて union by size だけにし、`snapshot()` を取った時点まで `rollback_to()` で戻す
//    (代わりに find が O(log n))。**「近傍を試して悪ければ戻す」型の探索ではこちらが正解。**
mod dsu {
    /// 経路圧縮 + union by size。`p[x]` は 根なら `-(要素数)`、そうでなければ親の index。
    pub struct Dsu {
        p: Vec<i32>,
        groups: usize,
    }

    impl Dsu {
        pub fn new(n: usize) -> Self {
            Dsu { p: vec![-1; n], groups: n }
        }

        /// 根を返す(path halving・非再帰なのでスタックを食わない)。
        #[inline]
        pub fn find(&mut self, mut x: usize) -> usize {
            while self.p[x] >= 0 {
                let par = self.p[x] as usize;
                if self.p[par] < 0 {
                    return par;
                }
                self.p[x] = self.p[par]; // 祖父へ付け替え
                x = self.p[par] as usize;
            }
            x
        }

        /// 併合。**新しく繋がったときだけ true**(既に同じ組なら false)。
        pub fn unite(&mut self, a: usize, b: usize) -> bool {
            let (mut a, mut b) = (self.find(a), self.find(b));
            if a == b {
                return false;
            }
            if self.p[a] > self.p[b] {
                std::mem::swap(&mut a, &mut b); // 大きい方(= p が小さい方)を根にする
            }
            self.p[a] += self.p[b];
            self.p[b] = a as i32;
            self.groups -= 1;
            true
        }

        pub fn same(&mut self, a: usize, b: usize) -> bool {
            self.find(a) == self.find(b)
        }

        /// `x` が属する組の要素数。
        pub fn size(&mut self, x: usize) -> usize {
            let r = self.find(x);
            (-self.p[r]) as usize
        }

        /// 残っている組の個数(初期値 n。unite が成功するたびに 1 減る)。
        pub fn groups(&self) -> usize {
            self.groups
        }

        pub fn len(&self) -> usize {
            self.p.len()
        }

        pub fn is_empty(&self) -> bool {
            self.p.is_empty()
        }
    }

    /// 巻き戻せる Union-Find。**経路圧縮しない**(union by size のみ)ので、unite の履歴を
    /// そのまま逆再生できる。`find` は O(log n)。
    /// 使い方: let t = d.snapshot(); ...試す...; d.rollback_to(t);   // 試行前へ完全に戻る
    pub struct RollbackDsu {
        p: Vec<i32>,
        hist: Vec<(usize, i32)>, // (書き換えた index, 書き換える前の値)
    }

    impl RollbackDsu {
        pub fn new(n: usize) -> Self {
            RollbackDsu { p: vec![-1; n], hist: Vec::new() }
        }

        /// 根を返す(`&self` で済む ── 圧縮しないため)。
        #[inline]
        pub fn find(&self, mut x: usize) -> usize {
            while self.p[x] >= 0 {
                x = self.p[x] as usize;
            }
            x
        }

        pub fn unite(&mut self, a: usize, b: usize) -> bool {
            let (mut a, mut b) = (self.find(a), self.find(b));
            if a == b {
                return false;
            }
            if self.p[a] > self.p[b] {
                std::mem::swap(&mut a, &mut b);
            }
            self.hist.push((a, self.p[a]));
            self.hist.push((b, self.p[b]));
            self.p[a] += self.p[b];
            self.p[b] = a as i32;
            true
        }

        pub fn same(&self, a: usize, b: usize) -> bool {
            self.find(a) == self.find(b)
        }

        pub fn size(&self, x: usize) -> usize {
            (-self.p[self.find(x)]) as usize
        }

        /// 組の個数。unite の成功 1 回につき履歴が 2 本積まれるので履歴長から出せる。
        pub fn groups(&self) -> usize {
            self.p.len() - self.hist.len() / 2
        }

        /// 現時点の履歴位置。`rollback_to` に渡す。
        pub fn snapshot(&self) -> usize {
            self.hist.len()
        }

        /// `t = snapshot()` を取った時点まで巻き戻す。
        pub fn rollback_to(&mut self, t: usize) {
            while self.hist.len() > t {
                let (i, v) = self.hist.pop().unwrap();
                self.p[i] = v;
            }
        }

        pub fn len(&self) -> usize {
            self.p.len()
        }

        pub fn is_empty(&self) -> bool {
            self.p.is_empty()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{Dsu, RollbackDsu};

        // 素朴な実装(色配列を全走査で塗る)と全操作で突き合わせる。
        // 「実装が正しいか」は等価な別実装との照合で決める(knowledge/techniques/equivalence-by-assert)。
        fn naive_unite(color: &mut [usize], a: usize, b: usize) -> bool {
            if color[a] == color[b] {
                return false;
            }
            let (from, to) = (color[b], color[a]);
            for c in color.iter_mut() {
                if *c == from {
                    *c = to;
                }
            }
            true
        }

        #[test]
        fn matches_naive() {
            let n = 30;
            let mut d = Dsu::new(n);
            let mut color: Vec<usize> = (0..n).collect();
            let mut seed = 12345u64;
            let mut rnd = move || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (seed >> 33) as usize
            };
            for _ in 0..500 {
                let (a, b) = (rnd() % n, rnd() % n);
                assert_eq!(d.unite(a, b), naive_unite(&mut color, a, b));
                for x in 0..n {
                    for y in 0..n {
                        assert_eq!(d.same(x, y), color[x] == color[y]);
                    }
                }
                let distinct: std::collections::HashSet<usize> = color.iter().copied().collect();
                assert_eq!(d.groups(), distinct.len());
                let cnt = color.iter().filter(|&&c| c == color[0]).count();
                assert_eq!(d.size(0), cnt);
            }
        }

        #[test]
        fn rollback_restores_exactly() {
            let n = 12;
            let mut d = RollbackDsu::new(n);
            d.unite(0, 1);
            d.unite(2, 3);
            let t = d.snapshot();
            let before: Vec<bool> = (0..n).flat_map(|x| (0..n).map(move |y| (x, y))).map(|(x, y)| d.same(x, y)).collect();
            let g = d.groups();

            // 試しに大量に繋いでから戻す
            for x in 0..n {
                d.unite(x, (x + 1) % n);
            }
            assert_eq!(d.groups(), 1);
            d.rollback_to(t);

            let after: Vec<bool> = (0..n).flat_map(|x| (0..n).map(move |y| (x, y))).map(|(x, y)| d.same(x, y)).collect();
            assert_eq!(before, after); // 連結関係が完全に一致
            assert_eq!(d.groups(), g);
            assert_eq!(d.snapshot(), t);
        }

        #[test]
        fn unite_reports_new_connection_only() {
            let mut d = Dsu::new(4);
            assert!(d.unite(0, 1));
            assert!(!d.unite(1, 0)); // 既に同じ
            assert_eq!(d.groups(), 3);
            assert!(d.unite(2, 3));
            assert!(d.unite(0, 3));
            assert_eq!(d.groups(), 1);
            assert_eq!(d.size(2), 4);
        }
    }
}
