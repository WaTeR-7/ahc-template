// 重み付き最短路。grid.rs の BFS は**辺重み 1 専用**なので、移動コストが場所で違うならこちら。
//   ・dijkstra : 非負重み。O((V+E) log V)
//   ・bfs01    : 重みが 0 か 1 だけ。O(V+E)(deque で log が消える。格子で効く)
// どちらも **複数始点**(距離 0 の点を複数置く / 初期距離つき)に対応する ── AHC では
// 「どの拠点からでもよい」「既に配置済みの点からの距離」を一発で作れるので使用頻度が高い。
//
// 使い方: let mut g = dijkstra::Graph::new(n);
//         g.add_undirected(u, v, w);                  // 有向なら add_edge
//         let (dist, prev) = g.dijkstra(s);           // 未到達は dijkstra::INF
//         let path = dijkstra::restore_path(&prev, s, t);   // [s, .., t](未到達なら空)
//         let (dist, _) = g.dijkstra_from(&[(a, 0), (b, 0)]);  // 複数始点
//
// ⚠ **格子を Graph に展開すると辺が 4HW 本になる**。盤面が大きく、重みが 0/1 なら bfs01 を
//    格子のまま書いた方が速い(この Graph は展開コストが割に合う中規模向け)。
mod dijkstra {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, VecDeque};

    /// 未到達を表す距離。
    pub const INF: u32 = u32::MAX;
    /// `prev` の「親なし」(始点、または未到達)。
    pub const NONE: u32 = u32::MAX;

    pub struct Graph {
        /// adj[u] = (行き先, 重み)
        pub adj: Vec<Vec<(u32, u32)>>,
    }

    impl Graph {
        pub fn new(n: usize) -> Self {
            Graph { adj: vec![Vec::new(); n] }
        }

        pub fn n(&self) -> usize {
            self.adj.len()
        }

        /// 有向辺 u → v。
        #[inline]
        pub fn add_edge(&mut self, u: usize, v: usize, w: u32) {
            self.adj[u].push((v as u32, w));
        }

        /// 無向辺(両向きに 1 本ずつ足す)。
        #[inline]
        pub fn add_undirected(&mut self, u: usize, v: usize, w: u32) {
            self.add_edge(u, v, w);
            self.add_edge(v, u, w);
        }

        /// 単一始点。戻り値は (距離, 直前の頂点)。未到達は `INF` / `NONE`。
        pub fn dijkstra(&self, src: usize) -> (Vec<u32>, Vec<u32>) {
            self.dijkstra_from(&[(src, 0)])
        }

        /// 複数始点(始点ごとに初期距離を与えられる)。
        pub fn dijkstra_from(&self, srcs: &[(usize, u32)]) -> (Vec<u32>, Vec<u32>) {
            let mut dist = vec![INF; self.n()];
            let mut prev = vec![NONE; self.n()];
            let mut heap: BinaryHeap<Reverse<(u32, u32)>> = BinaryHeap::new();
            for &(s, d0) in srcs {
                if d0 < dist[s] {
                    dist[s] = d0;
                    heap.push(Reverse((d0, s as u32)));
                }
            }
            while let Some(Reverse((d, u))) = heap.pop() {
                let u = u as usize;
                if d > dist[u] {
                    continue; // 古い(既により短い距離で確定済み)
                }
                for &(v, w) in &self.adj[u] {
                    let nd = d.saturating_add(w);
                    if nd < dist[v as usize] {
                        dist[v as usize] = nd;
                        prev[v as usize] = u as u32;
                        heap.push(Reverse((nd, v)));
                    }
                }
            }
            (dist, prev)
        }

        /// 重みが 0/1 のときの最短路(deque)。重み 0 は先頭へ、1 は末尾へ入れる。
        pub fn bfs01(&self, src: usize) -> (Vec<u32>, Vec<u32>) {
            self.bfs01_from(&[src])
        }

        /// 複数始点の 0-1 BFS。
        pub fn bfs01_from(&self, srcs: &[usize]) -> (Vec<u32>, Vec<u32>) {
            let mut dist = vec![INF; self.n()];
            let mut prev = vec![NONE; self.n()];
            let mut dq: VecDeque<u32> = VecDeque::new();
            for &s in srcs {
                if dist[s] != 0 {
                    dist[s] = 0;
                    dq.push_back(s as u32);
                }
            }
            while let Some(u) = dq.pop_front() {
                let u = u as usize;
                let d = dist[u];
                for &(v, w) in &self.adj[u] {
                    debug_assert!(w <= 1, "bfs01 は重み 0/1 専用(w={w})。dijkstra を使う");
                    let nd = d + w;
                    if nd < dist[v as usize] {
                        dist[v as usize] = nd;
                        prev[v as usize] = u as u32;
                        if w == 0 {
                            dq.push_front(v);
                        } else {
                            dq.push_back(v);
                        }
                    }
                }
            }
            (dist, prev)
        }
    }

    /// `prev` から `src` → `dst` の経路を復元する。**未到達なら空** を返す(呼び出し側で必ず判定する)。
    pub fn restore_path(prev: &[u32], src: usize, dst: usize) -> Vec<usize> {
        if src == dst {
            return vec![src];
        }
        if prev[dst] == NONE {
            return Vec::new();
        }
        let mut path = vec![dst];
        let mut cur = dst;
        while cur != src {
            let p = prev[cur];
            if p == NONE {
                return Vec::new(); // src に繋がっていない(別の始点から来た等)
            }
            cur = p as usize;
            path.push(cur);
        }
        path.reverse();
        path
    }

    #[cfg(test)]
    mod tests {
        use super::{Graph, INF, restore_path};

        fn sample() -> Graph {
            // 0 --1-- 1 --2-- 2      4 は孤立
            //  \______5______/
            let mut g = Graph::new(5);
            g.add_undirected(0, 1, 1);
            g.add_undirected(1, 2, 2);
            g.add_undirected(0, 2, 5);
            g.add_undirected(2, 3, 1);
            g
        }

        #[test]
        fn dijkstra_finds_the_shortest_and_the_path() {
            let g = sample();
            let (dist, prev) = g.dijkstra(0);
            assert_eq!(dist[0], 0);
            assert_eq!(dist[1], 1);
            assert_eq!(dist[2], 3); // 0-1-2 (3) が 0-2 直行 (5) より短い
            assert_eq!(dist[3], 4);
            assert_eq!(dist[4], INF); // 孤立
            assert_eq!(restore_path(&prev, 0, 3), vec![0, 1, 2, 3]);
            assert_eq!(restore_path(&prev, 0, 0), vec![0]);
            assert!(restore_path(&prev, 0, 4).is_empty()); // 未到達は空
        }

        #[test]
        fn multi_source_takes_the_nearest() {
            let g = sample();
            let (dist, _) = g.dijkstra_from(&[(0, 0), (3, 0)]);
            assert_eq!(dist, vec![0, 1, 1, 0, INF]);
            // 初期距離つき: 3 に下駄 10 を履かせると 2 は 0 側から来る
            let (dist, _) = g.dijkstra_from(&[(0, 0), (3, 10)]);
            assert_eq!(dist[2], 3);
        }

        #[test]
        fn bfs01_agrees_with_dijkstra() {
            // 0/1 重みのランダムグラフで両者を突き合わせる(等価な別実装との照合)
            let mut seed = 5u64;
            let mut rnd = move || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 33) as usize
            };
            for n in 2..12usize {
                let mut g = Graph::new(n);
                for _ in 0..(n * 3) {
                    let (u, v, w) = (rnd() % n, rnd() % n, (rnd() % 2) as u32);
                    g.add_edge(u, v, w);
                }
                let (a, _) = g.dijkstra(0);
                let (b, _) = g.bfs01(0);
                assert_eq!(a, b, "n={n}");
            }
        }
    }
}
