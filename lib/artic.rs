// 無向グラフの関節点(切断点)。取り除くと連結成分が増える頂点を Tarjan の lowlink で全列挙。
// AHC では「その活性セルを消すと残りが分断されるか」の判定(仕上げ順・連結性維持)に。
// 使い方: let mut g = artic::Graph::new(n);
//         g.add_edge(u, v);              // 無向辺
//         let cut = g.articulation();    // cut[v] == true なら v は関節点
// 注意: DFS は再帰(深さ O(n))。数万頂点級の細長いグラフではスタックに余裕を(別スレッド起動など)。
mod artic {
    pub struct Graph {
        n: usize,
        adj: Vec<Vec<usize>>,
    }

    impl Graph {
        pub fn new(n: usize) -> Self {
            Graph { n, adj: vec![Vec::new(); n] }
        }

        pub fn add_edge(&mut self, u: usize, v: usize) {
            self.adj[u].push(v);
            self.adj[v].push(u);
        }

        /// 関節点判定の bool 列を返す。非連結グラフでも各成分をまとめて処理する。
        pub fn articulation(&self) -> Vec<bool> {
            let n = self.n;
            let mut is_cut = vec![false; n];
            let mut disc = vec![0u32; n]; // 発見時刻(0 = 未訪問)
            let mut low = vec![0u32; n];
            let mut timer = 0u32;
            for s in 0..n {
                if disc[s] == 0 {
                    let children = self.dfs(s, usize::MAX, &mut disc, &mut low, &mut timer, &mut is_cut);
                    // 根は「DFS 木で子が 2 個以上」のときだけ関節点。
                    is_cut[s] = children > 1;
                }
            }
            is_cut
        }

        // u を根とする部分木を DFS。戻り値は u の DFS 木上の子の数(根判定用)。
        fn dfs(
            &self,
            u: usize,
            parent: usize,
            disc: &mut [u32],
            low: &mut [u32],
            timer: &mut u32,
            is_cut: &mut [bool],
        ) -> usize {
            *timer += 1;
            disc[u] = *timer;
            low[u] = *timer;
            let mut children = 0;
            let mut skipped_parent = false;
            for k in 0..self.adj[u].len() {
                let v = self.adj[u][k];
                if v == parent && !skipped_parent {
                    skipped_parent = true; // 親への戻り辺は 1 本だけ無視(多重辺は 2 本目以降を back-edge 扱い)
                    continue;
                }
                if disc[v] == 0 {
                    children += 1;
                    self.dfs(v, u, disc, low, timer, is_cut);
                    low[u] = low[u].min(low[v]);
                    // 非根の u で、子 v 以下が u より上に戻れない ⇒ u は関節点
                    if parent != usize::MAX && low[v] >= disc[u] {
                        is_cut[u] = true;
                    }
                } else {
                    low[u] = low[u].min(disc[v]);
                }
            }
            children
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Graph;

        // 素朴な関節点判定: v を除いた誘導部分グラフの連結成分数が、元(v 抜き頂点集合)より増えるか。
        fn brute(n: usize, edges: &[(usize, usize)]) -> Vec<bool> {
            let comps = |removed: usize| -> usize {
                let mut adj = vec![Vec::new(); n];
                for &(u, v) in edges {
                    if u != removed && v != removed {
                        adj[u].push(v);
                        adj[v].push(u);
                    }
                }
                let mut seen = vec![false; n];
                let mut c = 0;
                for s in 0..n {
                    if s == removed || seen[s] {
                        continue;
                    }
                    c += 1;
                    let mut st = vec![s];
                    seen[s] = true;
                    while let Some(x) = st.pop() {
                        for &y in &adj[x] {
                            if !seen[y] {
                                seen[y] = true;
                                st.push(y);
                            }
                        }
                    }
                }
                c
            };
            let base = comps(usize::MAX); // 誰も除かない
            (0..n).map(|v| comps(v) > base).collect()
        }

        #[test]
        fn path_middle_is_cut() {
            // 0-1-2: 中央 1 だけが関節点
            let mut g = Graph::new(3);
            g.add_edge(0, 1);
            g.add_edge(1, 2);
            assert_eq!(g.articulation(), vec![false, true, false]);
        }

        #[test]
        fn cycle_has_no_cut() {
            // 三角形は関節点なし
            let mut g = Graph::new(3);
            g.add_edge(0, 1);
            g.add_edge(1, 2);
            g.add_edge(2, 0);
            assert_eq!(g.articulation(), vec![false, false, false]);
        }

        #[test]
        fn random_against_brute() {
            // 決定的な小規模ランダムグラフで素朴解と一致することを確認
            let mut state = 12345u64;
            let mut rng = || {
                state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = state;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^ (z >> 31)
            };
            for _ in 0..300 {
                let n = 2 + (rng() % 7) as usize; // 2..=8 頂点
                let mut edges = Vec::new();
                let mut g = Graph::new(n);
                for _ in 0..(rng() % 12) {
                    let u = (rng() % n as u64) as usize;
                    let v = (rng() % n as u64) as usize;
                    if u != v {
                        g.add_edge(u, v);
                        edges.push((u, v));
                    }
                }
                assert_eq!(g.articulation(), brute(n, &edges));
            }
        }
    }
}
