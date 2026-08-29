// 焼きなまし(SA)の運転席。**温度 / 進行度 / 時間チェックの間引き / 受理判定 / 統計**だけを持つ。
// 状態と近傍は問題ごとに違うので持たない(持たせると結局毎回書き換えることになる)。
//
// 規約: **スコアは大きいほど良い(最大化)。`delta` = 適用後 − 現在。**
//       最小化問題なら符号を反転して渡す(`ann.accept(-cost_delta, ..)`)。
//
// 使い方(この形をそのままコピペして埋める。`rng.rs` も一緒に貼る):
//   let mut ann = anneal::Annealer::new(2000.0, 10.0);  // t0=開始温度, t1=終了温度(env で上書き可)
//   let mut rng = rng::Rng::new(0x1234_5678);
//   let mut cur = score(&sol);
//   let (mut best, mut best_sol) = (cur, sol.clone());
//   while ann.running() {
//       let mv = /* 近傍を1つ選ぶ: rng.below(..) など */;
//       let delta = /* 適用したらスコアがどれだけ増えるか(★差分で出す) */;
//       if ann.accept(delta, || rng.f64()) {
//           apply(&mut sol, mv);
//           cur += delta;
//           if cur > best { best = cur; best_sol = sol.clone(); }   // ★ best は必ず別に持つ
//       }
//   }
//   ann.report();   // iters / 受理率 / 最終温度 を stderr へ
//   // 提出するのは best_sol
//
// 🔴 落とし穴(毎回ここで落ちる):
//  ① **温度はスコアのスケールで決める。** t0 は「1 手の delta の大きい方」、t1 は「意味のある最小の差」。
//     桁が合っていないと、全部受理(= ただのランダムウォーク)か 全部棄却(= ただの山登り)になる。
//     **見るのは `report()` の `worse`(悪化手の受理率)** ── 温度が効いているかはこの数字だけが答える
//     (全体の受理率は改善手を含むので、山登りでも高く出る)。
//     目安: **序盤 3〜5 割 / 終盤ほぼ 0**。区間ごとに読むには
//     `if ann.progress() > 0.1 && !done { ann.report(); ann.reset_stats(); done = true; }` のように挟む。
//     **まず温度を合わせてから中身を疑う** ── ここがズレたまま近傍を作り込んでも何も分からない。
//  ② **best を別に持つ。** SA の現在解は終了時点でベストとは限らない
//     (受理率が高い区間では現在解は best から遠い ── knowledge/techniques/mechanism-failure-triage)。
//  ③ **delta は差分計算で出す。** 毎回フルスコアを計算すると反復数が 2〜3 桁落ちて、SA でなくなる。
//  ④ **`running()` は時間を毎回は見ない**(既定 256 回に 1 回)。1 反復が重い探索では
//     `check_every(16)` などに縮める ── 縮めないと TL を突き抜ける。
//
// 決定性: 既定は壁時計で進行度を決めるので**実行のたびに結果が変わる**。`AHC_ITERS=N` を与えると
// 進行度を**反復数**で決める決定モードになり、`scripts/same.sh` の byte 一致検証に使える
// (CLAUDE.md「速度だけを変える最適化の byte 一致検証は壁時計ゲートを無効化して行う」の受け皿)。
mod anneal {
    use std::time::Instant;

    pub struct Annealer {
        start: Instant,
        tl: u128,       // ms(AHC_TL)
        t0: f64,        // 開始温度
        t1: f64,        // 終了温度
        max_iters: u64, // AHC_ITERS。0 なら時間で進行(既定)
        mask: u64,      // 進行度を見直す間引き(2^k - 1)
        iters: u64,
        accepted: u64,
        worse: u64,          // 悪化手を試した回数
        worse_accepted: u64, // そのうち受理した回数(= 温度が効いている量)
        progress: f64,
        temp: f64,
        inv_temp: f64,
        done: bool,
    }

    impl Annealer {
        /// env から `AHC_TL`(既定 1900ms) / `AHC_T0` / `AHC_T1` / `AHC_ITERS`(既定 0 = 時間で進行)を読む。
        /// 引数の t0/t1 は env が無いときの既定値。**掃引は env でやる**(再コンパイル不要)。
        pub fn new(t0: f64, t1: f64) -> Self {
            fn env<T: std::str::FromStr>(key: &str, default: T) -> T {
                std::env::var(key).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
            }
            Self::with_params(env("AHC_T0", t0), env("AHC_T1", t1), env("AHC_TL", 1900u128), env("AHC_ITERS", 0u64))
        }

        /// env を読まない版(テスト・多段 SA で温度を明示したいとき)。
        pub fn with_params(t0: f64, t1: f64, tl: u128, max_iters: u64) -> Self {
            assert!(t0 > 0.0 && t1 > 0.0, "温度は正でなければならない (t0={t0}, t1={t1})");
            assert!(t1 <= t0, "終了温度は開始温度以下 (t0={t0}, t1={t1})");
            Annealer {
                start: Instant::now(),
                tl,
                t0,
                t1,
                max_iters,
                mask: 255,
                iters: 0,
                accepted: 0,
                worse: 0,
                worse_accepted: 0,
                progress: 0.0,
                temp: t0,
                inv_temp: 1.0 / t0,
                done: false,
            }
        }

        /// 進行度を見直す間隔(2 冪)。1 反復が重いときは 16 などに縮める。
        pub fn check_every(mut self, n: u64) -> Self {
            assert!(n.is_power_of_two(), "間隔は 2 冪 (n={n})");
            self.mask = n - 1;
            self
        }

        /// ループ条件。**呼ぶたびに 1 反復を消費する**(`while ann.running() { .. }` の形で使う)。
        #[inline]
        pub fn running(&mut self) -> bool {
            if self.iters & self.mask == 0 || (self.max_iters > 0 && self.iters >= self.max_iters) {
                self.refresh();
            }
            if self.done {
                return false;
            }
            self.iters += 1;
            true
        }

        /// 進行度と温度を計算し直す。冷却は幾何(指数): `T = t0 * (t1/t0)^progress`。
        fn refresh(&mut self) {
            let p = if self.max_iters > 0 {
                self.iters as f64 / self.max_iters as f64
            } else if self.tl == 0 {
                1.0 // TL=0 は「即終了」。0 除算で NaN を作らない(NaN は >= 1.0 が偽 = 止まらない)
            } else {
                self.start.elapsed().as_secs_f64() * 1000.0 / self.tl as f64
            };
            if p >= 1.0 {
                self.done = true;
                self.progress = 1.0;
            } else {
                self.progress = p;
            }
            self.temp = self.t0 * (self.t1 / self.t0).powf(self.progress);
            self.inv_temp = 1.0 / self.temp;
        }

        /// 受理判定。`delta >= 0`(改善・横ばい)は常に受理、悪化は `exp(delta / T)` の確率で受理。
        /// `rand01` は **悪化したときにだけ**呼ばれる([0,1) の一様乱数を返すこと)。
        /// 呼び方: `if ann.accept(delta, || rng.f64()) { .. }`
        #[inline]
        pub fn accept(&mut self, delta: f64, rand01: impl FnOnce() -> f64) -> bool {
            if delta >= 0.0 {
                self.accepted += 1;
                return true;
            }
            self.worse += 1;
            if rand01() < (delta * self.inv_temp).exp() {
                self.accepted += 1;
                self.worse_accepted += 1;
                return true;
            }
            false
        }

        /// 整数スコア版(`delta` をそのまま渡せる糖衣)。
        #[inline]
        pub fn accept_i64(&mut self, delta: i64, rand01: impl FnOnce() -> f64) -> bool {
            self.accept(delta as f64, rand01)
        }

        #[inline]
        pub fn temp(&self) -> f64 {
            self.temp
        }
        /// 0.0(開始) → 1.0(終了)。近傍の種類を進行度で切り替えるときに使う。
        #[inline]
        pub fn progress(&self) -> f64 {
            self.progress
        }
        #[inline]
        pub fn iters(&self) -> u64 {
            self.iters
        }
        #[inline]
        pub fn accepted(&self) -> u64 {
            self.accepted
        }
        pub fn elapsed_ms(&self) -> u128 {
            self.start.elapsed().as_millis()
        }
        /// 全体の受理率(改善手を含む)。山登りでも高く出るので、温度の判定には使わない。
        pub fn accept_rate(&self) -> f64 {
            self.accepted as f64 / self.iters.max(1) as f64
        }

        /// **悪化手の受理率** ── 温度が効いているかを答えるのはこちら。
        /// 目安は序盤 3〜5 割 / 終盤ほぼ 0(`reset_stats` で区間ごとに読む)。
        pub fn worse_accept_rate(&self) -> f64 {
            self.worse_accepted as f64 / self.worse.max(1) as f64
        }

        /// 統計だけを 0 に戻す(温度・進行度・反復数はそのまま)。
        /// 進行度で区切って `report(); reset_stats();` すると**区間ごとの受理率**が読める。
        pub fn reset_stats(&mut self) {
            self.accepted = 0;
            self.worse = 0;
            self.worse_accepted = 0;
        }

        /// stderr へ 1 行。掃引のログに必ず残す(スコアだけ見ても温度の当たり外れは分からない)。
        pub fn report(&self) {
            eprintln!(
                "[anneal] iters={} accept={} ({:.1}%) worse={}/{} ({:.1}%) t0={:.4} t1={:.4} temp={:.4} ms={} mode={}",
                self.iters,
                self.accepted,
                100.0 * self.accept_rate(),
                self.worse_accepted,
                self.worse,
                100.0 * self.worse_accept_rate(),
                self.t0,
                self.t1,
                self.temp,
                self.elapsed_ms(),
                if self.max_iters > 0 { "iters(決定的)" } else { "time" }
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Annealer;

        #[test]
        fn iter_mode_runs_exactly_max_iters() {
            for &m in &[1u64, 7, 100, 256, 1000, 1024] {
                let mut a = Annealer::with_params(100.0, 1.0, 10_000, m);
                let mut n = 0u64;
                while a.running() {
                    n += 1;
                    assert!(n <= m + 1, "止まらない(m={m})");
                }
                assert_eq!(n, m, "反復数が max_iters と一致しない(m={m})");
                assert_eq!(a.iters(), m);
            }
        }

        #[test]
        fn temperature_decays_from_t0_to_t1() {
            let mut a = Annealer::with_params(100.0, 1.0, 10_000, 1024).check_every(1);
            assert!((a.temp() - 100.0).abs() < 1e-9);
            let mut prev = a.temp();
            let mut last = prev;
            while a.running() {
                let t = a.temp();
                assert!(t <= prev + 1e-12, "温度が上がった");
                assert!((1.0..=100.0).contains(&t), "温度が範囲外: {t}");
                prev = t;
                last = t;
            }
            // 最後の反復の温度は t1 に十分近い(進行度 (m-1)/m での値)
            assert!(last < 1.01, "終端温度が t1 まで落ちていない: {last}");
            assert!((a.progress() - 1.0).abs() < 1e-12);
        }

        #[test]
        fn accept_rule() {
            let mut a = Annealer::with_params(10.0, 10.0, 10_000, 1_000_000);
            a.running();
            // 改善・横ばいは乱数を消費せずに必ず受理(消費したら panic するクロージャで確認)
            assert!(a.accept(5.0, || panic!("改善時に乱数を引いてはいけない")));
            assert!(a.accept(0.0, || panic!("横ばいでも乱数を引いてはいけない")));
            // 悪化: exp(-10/10) = 0.3679. r がそれ未満なら受理、超えれば棄却
            assert!(a.accept(-10.0, || 0.30));
            assert!(!a.accept(-10.0, || 0.40));
            // 十分大きな悪化は棄却(確率がほぼ 0)
            assert!(!a.accept(-1000.0, || 0.999_999));
            assert_eq!(a.accepted(), 3);
        }

        #[test]
        fn worse_accept_rate_tracks_only_worsening_moves() {
            let mut a = Annealer::with_params(10.0, 10.0, 10_000, 1_000_000);
            a.running();
            // 改善手は worse に数えない(全体の受理率だけが上がる)
            for _ in 0..10 {
                assert!(a.accept(1.0, || panic!("改善で乱数を引いてはいけない")));
            }
            assert_eq!(a.worse_accept_rate(), 0.0, "悪化手が 0 件なら 0 を返す(0 除算しない)");
            // 悪化手 4 件のうち 2 件受理 → 悪化受理率 50%、全体は 12/14
            assert!(a.accept(-10.0, || 0.1));
            assert!(!a.accept(-10.0, || 0.9));
            assert!(a.accept(-10.0, || 0.2));
            assert!(!a.accept(-10.0, || 0.8));
            assert!((a.worse_accept_rate() - 0.5).abs() < 1e-12);
            assert_eq!(a.accepted(), 12);
            a.reset_stats();
            assert_eq!(a.accepted(), 0);
            assert_eq!(a.worse_accept_rate(), 0.0);
            assert!(a.iters() > 0, "reset_stats は反復数を消さない");
        }

        #[test]
        fn time_mode_stops_at_tl() {
            // TL を極小にすると、間引き間隔ぶんだけ回して即座に止まる(無限ループしない)
            let mut a = Annealer::with_params(10.0, 1.0, 0, 0).check_every(1);
            let mut n = 0u64;
            while a.running() {
                n += 1;
                assert!(n < 1_000_000, "TL=0 なのに止まらない");
            }
            assert_eq!(n, 0);
        }
    }
}
