// 壁付き格子(grid)。AHC 頻出の「セル間に壁がある H×W 盤面」を 1 次元 id で扱い、
// 壁を越えない 4 近傍走査と BFS 最短距離(辺重み 1)を提供する。
// 壁モデル(セル間に立つ):
//   vwall[i*w + j] = セル(i,j) と (i,j+1) の間の縦壁(右側)。j < w-1 のみ意味を持つ。
//   hwall[i*w + j] = セル(i,j) と (i+1,j) の間の横壁(下側)。i < h-1 のみ意味を持つ。
// 使い方: let mut g = grid::Grid::new(h, w);
//         let c = g.id(i, j); g.vwall[c] = true; // (i,j)-(i,j+1) を遮断(id を先に束縛して借用衝突回避)
//         let dist = g.bfs(g.id(0, 0));           // 単一始点。未到達は u16::MAX
//         let ap   = g.all_pairs();               // 全点間。ap[s*n + t](n = h*w)
mod grid {
    use std::collections::VecDeque;

    pub struct Grid {
        pub h: usize,
        pub w: usize,
        pub vwall: Vec<bool>, // 縦壁(右隣との間)。index = i*w + j
        pub hwall: Vec<bool>, // 横壁(下隣との間)。index = i*w + j
    }

    impl Grid {
        pub fn new(h: usize, w: usize) -> Self {
            Grid { h, w, vwall: vec![false; h * w], hwall: vec![false; h * w] }
        }

        #[inline]
        pub fn id(&self, i: usize, j: usize) -> usize {
            i * self.w + j
        }

        /// 壁を越えずに到達できる 4 近傍セル id を `f` に渡す(上下左右)。
        #[inline]
        pub fn for_each_neighbor(&self, cell: usize, mut f: impl FnMut(usize)) {
            let j = cell % self.w;
            if cell >= self.w && !self.hwall[cell - self.w] {
                f(cell - self.w); // 上
            }
            if cell + self.w < self.h * self.w && !self.hwall[cell] {
                f(cell + self.w); // 下
            }
            if j > 0 && !self.vwall[cell - 1] {
                f(cell - 1); // 左
            }
            if j + 1 < self.w && !self.vwall[cell] {
                f(cell + 1); // 右
            }
        }

        /// `start` からの最短距離(辺重み 1・壁越え不可)。未到達は u16::MAX。
        pub fn bfs(&self, start: usize) -> Vec<u16> {
            let n = self.h * self.w;
            let mut dist = vec![u16::MAX; n];
            let mut q = VecDeque::new();
            dist[start] = 0;
            q.push_back(start);
            while let Some(cur) = q.pop_front() {
                let nd = dist[cur] + 1;
                self.for_each_neighbor(cur, |nc| {
                    if dist[nc] == u16::MAX {
                        dist[nc] = nd;
                        q.push_back(nc);
                    }
                });
            }
            dist
        }

        /// 全点間最短距離。戻り値 d に対し d[s*n + t] が s→t 距離(n = h*w)。
        /// メモリは n^2 * 2 bytes。大きな盤面では注意。
        pub fn all_pairs(&self) -> Vec<u16> {
            let n = self.h * self.w;
            let mut d = vec![u16::MAX; n * n];
            for s in 0..n {
                let ds = self.bfs(s);
                d[s * n..(s + 1) * n].copy_from_slice(&ds);
            }
            d
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Grid;

        #[test]
        fn line_with_wall() {
            // 1x3: 0 - 1 - 2。(1,2) 間に縦壁を立てると 2 は 0 から到達不能。
            let mut g = Grid::new(1, 3);
            let d = g.bfs(g.id(0, 0));
            assert_eq!(d, vec![0, 1, 2]);
            let c = g.id(0, 1);
            g.vwall[c] = true; // (0,1)-(0,2) を遮断
            let d = g.bfs(g.id(0, 0));
            assert_eq!(d, vec![0, 1, u16::MAX]);
        }

        #[test]
        fn detour_around_wall() {
            // 2x2 全開放: 対角 (0,0)->(1,1) は距離 2。
            let mut g = Grid::new(2, 2);
            let n = g.h * g.w;
            let ap = g.all_pairs();
            assert_eq!(ap[g.id(0, 0) * n + g.id(1, 1)], 2);
            // 全点間は対称
            for s in 0..n {
                for t in 0..n {
                    assert_eq!(ap[s * n + t], ap[t * n + s]);
                }
            }
            // (0,0)-(0,1) と (0,0)-(1,0) を塞ぐと (0,0) は孤立
            let c = g.id(0, 0);
            g.vwall[c] = true; // 右を遮断
            g.hwall[c] = true; // 下を遮断
            let d = g.bfs(g.id(0, 0));
            assert_eq!(d[g.id(0, 0)], 0);
            assert_eq!(d[g.id(0, 1)], u16::MAX);
            assert_eq!(d[g.id(1, 0)], u16::MAX);
        }
    }
}
