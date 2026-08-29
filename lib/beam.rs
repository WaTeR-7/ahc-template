// ビームサーチの道具 3 点セット。状態と遷移は問題ごとに違うので持たない ──
// **毎回自作していて事故るのはこの 3 つ**なので、ここに固定する。
//   ・Tree<Op>   : 手順復元の木。状態を層ごとに全部保存せず、**親リンクだけ**辿って手順を復元する。
//   ・sort_top_k_by / top_k_by : 上位 W 件の選抜。`select_nth_unstable` で O(n)(全ソートしない)。
//   ・Set64      : u64 キーの重複除去(線形探索の開番地法)。std の HashSet より速く、`clear` で使い回す。
//
// 使い方(1 層ぶんの骨格。ハッシュは zobrist.rs で作る):
//   let width = env::get("AHC_W", 100);
//   let mut tree = beam::Tree::new();
//   let mut seen = beam::Set64::new(width * 8);
//   let mut cur: Vec<(State, u32)> = vec![(init, beam::ROOT)];   // (状態, 木のノード id)
//   for _turn in 0..turns {
//       // ① 全候補を出す(評価は「大きいほど良い」に揃える)
//       let mut cand: Vec<(i64, u32, u64, Op)> = Vec::new();     // (評価, 親の cur index, ハッシュ, 手)
//       for (bi, (st, _)) in cur.iter().enumerate() {
//           for op in st.moves() {
//               cand.push((st.eval_after(&op), bi as u32, st.hash_after(&op), op));
//           }
//       }
//       // ② 多めに選抜してから重複除去する(★ 逆順にすると dedup 後に W 本埋まらない)
//       beam::sort_top_k_by(&mut cand, width * 3, |c| c.0);
//       seen.clear();
//       let mut next = Vec::with_capacity(width);
//       for (_, bi, h, op) in cand {
//           if next.len() == width { break; }
//           if !seen.insert(h) { continue; }                     // 同じ状態は 1 つだけ残す
//           let mut st = cur[bi as usize].0.clone();
//           st.apply(&op);
//           let id = tree.push(cur[bi as usize].1, op);
//           next.push((st, id));
//       }
//       cur = next;
//   }
//   let ops = tree.path(best_id);   // 根からの手順(Vec<Op>)
//
// 🔴 落とし穴:
//  ① **選抜 → 重複除去 の順にする。** 逆(除去してから選抜)は全候補を触るので遅く、
//     選抜してから除去すると W を割るので**多めに選抜する**(上の `width * 3`)。
//  ② **状態を全層ぶん持たない。** 手順は Tree の親リンクで復元する(状態 W×D 個はメモリを食う)。
//  ③ **評価の向きを揃える。** ここは全部「大きいほど良い」。コスト最小化なら符号を反転して入れる。
//  ④ **Set64 の容量は 1 層で insert する件数の 2 倍以上**。足りなければ自動で倍化するが、
//     倍化のたびに全再挿入が走るので最初から確保しておく。
mod beam {
    /// 木の根(「まだ 1 手も打っていない」印)。最初の層の親にこれを渡す。
    pub const ROOT: u32 = u32::MAX;

    /// 手順復元用の木。ノードは (親 id, その親から打った手) だけを持つ。
    pub struct Tree<Op> {
        par: Vec<u32>,
        op: Vec<Op>,
    }

    impl<Op: Clone> Tree<Op> {
        pub fn new() -> Self {
            Tree { par: Vec::new(), op: Vec::new() }
        }

        pub fn with_capacity(n: usize) -> Self {
            Tree { par: Vec::with_capacity(n), op: Vec::with_capacity(n) }
        }

        /// 親 `par`(最初の層は `ROOT`)に手 `op` をぶら下げ、新しいノード id を返す。
        #[inline]
        pub fn push(&mut self, par: u32, op: Op) -> u32 {
            self.par.push(par);
            self.op.push(op);
            (self.op.len() - 1) as u32
        }

        /// 根から `id` までの手順(打った順)。`ROOT` を渡すと空。
        pub fn path(&self, id: u32) -> Vec<Op> {
            let mut v = Vec::new();
            let mut i = id;
            while i != ROOT {
                v.push(self.op[i as usize].clone());
                i = self.par[i as usize];
            }
            v.reverse();
            v
        }

        /// 根からの手数(`path` を作らずに深さだけ知りたいとき)。
        pub fn depth(&self, id: u32) -> usize {
            let mut d = 0;
            let mut i = id;
            while i != ROOT {
                d += 1;
                i = self.par[i as usize];
            }
            d
        }

        pub fn len(&self) -> usize {
            self.op.len()
        }

        pub fn is_empty(&self) -> bool {
            self.op.is_empty()
        }

        pub fn clear(&mut self) {
            self.par.clear();
            self.op.clear();
        }
    }

    impl<Op: Clone> Default for Tree<Op> {
        fn default() -> Self {
            Self::new()
        }
    }

    /// 評価(大きいほど良い)の上位 k 件を先頭に集めて切り詰める。**順不同**・O(n)。
    pub fn top_k_by<T>(v: &mut Vec<T>, k: usize, mut key: impl FnMut(&T) -> i64) {
        if v.len() > k {
            v.select_nth_unstable_by(k, |a, b| key(b).cmp(&key(a)));
            v.truncate(k);
        }
    }

    /// 上位 k 件を **降順に整列** して切り詰める。O(n + k log k)。
    /// ビームの選抜はこちら ── 重複除去を「良い順」に処理する必要があるため。
    pub fn sort_top_k_by<T>(v: &mut Vec<T>, k: usize, mut key: impl FnMut(&T) -> i64) {
        top_k_by(v, k, &mut key);
        v.sort_unstable_by(|a, b| key(b).cmp(&key(a)));
    }

    /// u64 キーの集合(開番地法・線形探索)。ビーム 1 層ぶんの重複除去に使い、層ごとに `clear()`。
    /// キー 0 は内部で 1 に潰す(0 を空きの番兵に使うため)── ハッシュ値なので実害はない。
    pub struct Set64 {
        keys: Vec<u64>,
        mask: usize,
        len: usize,
    }

    impl Set64 {
        /// `cap` 件を入れても詰まらない表(2 冪・容量は cap の 2 倍以上)を作る。
        pub fn new(cap: usize) -> Self {
            let mut n = 16usize;
            while n < cap.max(1) * 2 {
                n <<= 1;
            }
            Set64 { keys: vec![0; n], mask: n - 1, len: 0 }
        }

        /// 全消去(容量は保つ)。**層ごとにこれを呼んで使い回す。**
        pub fn clear(&mut self) {
            self.keys.fill(0);
            self.len = 0;
        }

        /// 新規なら true、既にあれば false。
        #[inline]
        pub fn insert(&mut self, key: u64) -> bool {
            if (self.len + 1) * 2 > self.keys.len() {
                self.grow();
            }
            self.insert_raw(if key == 0 { 1 } else { key })
        }

        pub fn contains(&self, key: u64) -> bool {
            let k = if key == 0 { 1 } else { key };
            let mut i = Self::mix(k) as usize & self.mask;
            loop {
                let cur = self.keys[i];
                if cur == 0 {
                    return false;
                }
                if cur == k {
                    return true;
                }
                i = (i + 1) & self.mask;
            }
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        #[inline]
        fn insert_raw(&mut self, k: u64) -> bool {
            let mut i = Self::mix(k) as usize & self.mask;
            loop {
                let cur = self.keys[i];
                if cur == 0 {
                    self.keys[i] = k;
                    self.len += 1;
                    return true;
                }
                if cur == k {
                    return false;
                }
                i = (i + 1) & self.mask;
            }
        }

        fn grow(&mut self) {
            let n = self.keys.len() * 2;
            let old = std::mem::replace(&mut self.keys, vec![0; n]);
            self.mask = n - 1;
            self.len = 0;
            for k in old {
                if k != 0 {
                    self.insert_raw(k);
                }
            }
        }

        /// ハッシュ値をさらに撹拌(zobrist の下位ビットに偏りがあっても詰まらないように)。
        #[inline]
        fn mix(mut z: u64) -> u64 {
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{ROOT, Set64, Tree, sort_top_k_by, top_k_by};

        #[test]
        fn tree_reconstructs_the_path() {
            let mut t: Tree<char> = Tree::new();
            let a = t.push(ROOT, 'L');
            let b = t.push(a, 'U');
            let c = t.push(b, 'R');
            let d = t.push(a, 'D'); // 枝分かれ
            assert_eq!(t.path(c), vec!['L', 'U', 'R']);
            assert_eq!(t.path(d), vec!['L', 'D']);
            assert_eq!(t.path(ROOT), Vec::<char>::new());
            assert_eq!(t.depth(c), 3);
            assert_eq!(t.depth(ROOT), 0);
            assert_eq!(t.len(), 4);
            t.clear();
            assert!(t.is_empty());
        }

        #[test]
        fn top_k_keeps_the_best_k() {
            // 全ソートした結果の上位 k と、集合として一致すること
            let mut seed = 999u64;
            let mut rnd = move || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 33) as i64 % 1000
            };
            for n in [0usize, 1, 2, 5, 50] {
                for k in [0usize, 1, 3, 50, 100] {
                    let v: Vec<i64> = (0..n).map(|_| rnd()).collect();
                    let mut expect = v.clone();
                    expect.sort_unstable_by(|a, b| b.cmp(a));
                    expect.truncate(k);

                    let mut got = v.clone();
                    top_k_by(&mut got, k, |x| *x);
                    got.sort_unstable_by(|a, b| b.cmp(a));
                    assert_eq!(got, expect, "top_k_by(n={n}, k={k})");

                    let mut got = v.clone();
                    sort_top_k_by(&mut got, k, |x| *x);
                    assert_eq!(got, expect, "sort_top_k_by は降順で返る(n={n}, k={k})");
                }
            }
        }

        #[test]
        fn set64_dedups_and_grows() {
            let mut s = Set64::new(4); // わざと小さく作って倍化を踏ませる
            for i in 1..=1000u64 {
                assert!(s.insert(i * 0x9E37_79B9), "初回は新規のはず");
            }
            for i in 1..=1000u64 {
                assert!(!s.insert(i * 0x9E37_79B9), "2 回目は重複のはず");
                assert!(s.contains(i * 0x9E37_79B9));
            }
            assert_eq!(s.len(), 1000);
            assert!(!s.contains(12345));
            // キー 0 も 1 件として扱える(番兵と衝突しない)
            assert!(s.insert(0));
            assert!(!s.insert(0));
            s.clear();
            assert!(s.is_empty());
            assert!(!s.contains(0));
            assert!(s.insert(0));
        }
    }
}
