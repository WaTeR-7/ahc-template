// 壁付き格子(grid)。AHC 頻出の「セル間に壁がある H×W 盤面」を 1 次元 id で扱い、
// 壁を越えない 4 近傍走査と BFS 最短距離(辺重み 1)を提供する。
// 壁モデル(セル間に立つ):
//   vwall[i*w + j] = セル(i,j) と (i,j+1) の間の縦壁(右側)。j < w-1 のみ意味を持つ。
//   hwall[i*w + j] = セル(i,j) と (i+1,j) の間の横壁(下側)。i < h-1 のみ意味を持つ。
// 使い方: let mut g = grid::Grid::new(h, w);
//         let c = g.id(i, j); g.vwall[c] = true; // (i,j)-(i,j+1) を遮断(id を先に束縛して借用衝突回避)
//         let dist = g.bfs(g.id(0, 0));           // 単一始点。未到達は u16::MAX
//         let ap   = g.all_pairs();               // 全点間。ap[s*n + t](n = h*w)
//         let (i, j) = g.rc(cell);                // id → 座標(id の逆)
//         g.for_each_neighbor_dir(c, |nc, d| ..); // 方向つき(d は DIR / DIR_CHAR の添字)
// **壁を1つも立てなければ素の格子**として使える(Grid::new だけで 4 近傍・BFS が揃う)。
mod grid {
    use std::collections::VecDeque;

    /// 4 近傍の (di, dj) と、出力に使う方向文字。**順序は 上 下 左 右**。
    /// 🔴 **問題が使う文字と順序は毎回違う。条文を見て必ず差し替える**
    ///    (ここを確かめずに書くと、全部の解が同じ向きに間違ったまま最後まで気づかない)。
    pub const DIR: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    pub const DIR_CHAR: [u8; 4] = *b"UDLR";

    /// マンハッタン距離(壁を無視した下界としてよく使う)。
    #[inline]
    pub fn manhattan(a: (usize, usize), b: (usize, usize)) -> usize {
        a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
    }

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

        /// id → (行, 列)。`id` の逆。
        #[inline]
        pub fn rc(&self, cell: usize) -> (usize, usize) {
            (cell / self.w, cell % self.w)
        }

        /// 壁を越えずに到達できる 4 近傍セル id を `f` に渡す(上下左右)。
        #[inline]
        pub fn for_each_neighbor(&self, cell: usize, mut f: impl FnMut(usize)) {
            self.for_each_neighbor_dir(cell, |nc, _| f(nc));
        }

        /// 方向つきの 4 近傍。`f(隣の id, 方向の添字)` ── 添字は `DIR` / `DIR_CHAR` と対応する
        /// (0=上 1=下 2=左 3=右)。移動列をそのまま出力に組み立てるときはこちら。
        #[inline]
        pub fn for_each_neighbor_dir(&self, cell: usize, mut f: impl FnMut(usize, usize)) {
            let j = cell % self.w;
            if cell >= self.w && !self.hwall[cell - self.w] {
                f(cell - self.w, 0); // 上
            }
            if cell + self.w < self.h * self.w && !self.hwall[cell] {
                f(cell + self.w, 1); // 下
            }
            if j > 0 && !self.vwall[cell - 1] {
                f(cell - 1, 2); // 左
            }
            if j + 1 < self.w && !self.vwall[cell] {
                f(cell + 1, 3); // 右
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
        fn rc_is_the_inverse_of_id() {
            let g = Grid::new(4, 5);
            for i in 0..g.h {
                for j in 0..g.w {
                    assert_eq!(g.rc(g.id(i, j)), (i, j));
                }
            }
        }

        #[test]
        fn neighbor_dir_matches_the_dir_table() {
            // 方向の添字が DIR の (di, dj) と一致していること(ここがズレると出力が全部逆を向く)
            let g = Grid::new(3, 3);
            let c = g.id(1, 1);
            let mut seen = vec![];
            g.for_each_neighbor_dir(c, |nc, d| seen.push((nc, d)));
            assert_eq!(seen.len(), 4);
            for (nc, d) in seen {
                let (i, j) = g.rc(c);
                let (di, dj) = super::DIR[d];
                let want = g.id((i as i32 + di) as usize, (j as i32 + dj) as usize);
                assert_eq!(nc, want, "方向 {} ({}) がテーブルと違う", d, super::DIR_CHAR[d] as char);
            }
            // for_each_neighbor は方向つき版と同じ集合を返す(委譲しても壊れていない)
            let mut plain = vec![];
            g.for_each_neighbor(c, |nc| plain.push(nc));
            assert_eq!(plain, vec![g.id(0, 1), g.id(2, 1), g.id(1, 0), g.id(1, 2)]);
            assert_eq!(super::manhattan((0, 0), (2, 3)), 5);
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
