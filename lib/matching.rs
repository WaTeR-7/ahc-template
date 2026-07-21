// 二部マッチング(Kuhn の増加路法)。左 nl 頂点・右 nr 頂点の隣接リスト版。
// 計算量 O(V*E)。数百〜数千頂点の割当・辺彩色・行/列整列などに十分速い。
// 使い方: let mut m = matching::BipartiteMatching::new(nl, nr);
//         m.add_edge(u, v);                 // 左 u -- 右 v
//         let size = m.solve();             // 最大マッチングのサイズ
//         // 以後 m.match_l[u] = 左uの相手(右, 無ければ usize::MAX)、m.match_r[v] も同様。
mod matching {
    pub struct BipartiteMatching {
        nl: usize,
        nr: usize,
        adj: Vec<Vec<usize>>,
        pub match_l: Vec<usize>, // match_l[u] = u にマッチした右頂点(usize::MAX = 無し)
        pub match_r: Vec<usize>, // match_r[v] = v にマッチした左頂点(usize::MAX = 無し)
    }

    impl BipartiteMatching {
        pub fn new(nl: usize, nr: usize) -> Self {
            BipartiteMatching {
                nl,
                nr,
                adj: vec![Vec::new(); nl],
                match_l: vec![usize::MAX; nl],
                match_r: vec![usize::MAX; nr],
            }
        }

        /// 左頂点 `u` と右頂点 `v` の間に辺を張る。
        pub fn add_edge(&mut self, u: usize, v: usize) {
            self.adj[u].push(v);
        }

        // u から増加路を探す。見つかればマッチを更新して true。
        fn augment(&mut self, u: usize, seen: &mut [bool]) -> bool {
            for k in 0..self.adj[u].len() {
                let v = self.adj[u][k];
                if !seen[v] {
                    seen[v] = true;
                    // v が未マッチ、または v の現相手を別へ押し出せれば u--v を確定
                    if self.match_r[v] == usize::MAX || self.augment(self.match_r[v], seen) {
                        self.match_r[v] = u;
                        self.match_l[u] = v;
                        return true;
                    }
                }
            }
            false
        }

        /// 最大マッチングを計算し、そのサイズを返す。複数回呼んでも状態を作り直す。
        pub fn solve(&mut self) -> usize {
            self.match_l.iter_mut().for_each(|x| *x = usize::MAX);
            self.match_r.iter_mut().for_each(|x| *x = usize::MAX);
            let mut cnt = 0;
            for u in 0..self.nl {
                let mut seen = vec![false; self.nr];
                if self.augment(u, &mut seen) {
                    cnt += 1;
                }
            }
            cnt
        }
    }

    #[cfg(test)]
    mod tests {
        use super::BipartiteMatching;

        #[test]
        fn perfect_matching() {
            // 3x3 の対角。完全マッチング(サイズ 3)。
            let mut m = BipartiteMatching::new(3, 3);
            for i in 0..3 {
                m.add_edge(i, i);
            }
            assert_eq!(m.solve(), 3);
            for i in 0..3 {
                assert_eq!(m.match_l[i], i);
                assert_eq!(m.match_r[i], i);
            }
        }

        #[test]
        fn augmenting_path_needed() {
            // 左0->{0,1}, 左1->{0}。貪欲で 0-0 を取ると 1 が詰むが、増加路で 0-1,1-0 に組み替え → 2。
            let mut m = BipartiteMatching::new(2, 2);
            m.add_edge(0, 0);
            m.add_edge(0, 1);
            m.add_edge(1, 0);
            assert_eq!(m.solve(), 2);
        }

        #[test]
        fn bottleneck_limits_size() {
            // 左3頂点すべてが右0のみに接続 → 最大マッチングは 1。
            let mut m = BipartiteMatching::new(3, 2);
            m.add_edge(0, 0);
            m.add_edge(1, 0);
            m.add_edge(2, 0);
            assert_eq!(m.solve(), 1);
            assert_eq!(m.match_r[1], usize::MAX); // 右1は誰ともマッチしない
        }
    }
}
