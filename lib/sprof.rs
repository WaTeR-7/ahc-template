// 命令の番地を一定間隔で採るプロファイラ(SIGPROF。外部クレート・外部の道具なし)。
//
// なぜ必要か: `prof.rs` のように**細かい区間ごとにタイマーを挟む計測は、軽い段を水増しする**。
// ahc072 では、出来事ごとに rdtsc を挟むと「何もしない出来事の通過」が DP の 3〜4 割に見えたが、
// この方式で採ると実際は 1 割前後で、本命は別の段(手の列の再生が全体の半分)だった。
// ⇒ **段の大きさを知るのは `prof.rs`(回数 × 1 回あたり)、内側ループの中の割合を知るのはこちら。**
//
// 使い方:
//   1) この `mod sprof` を解答に貼り、測りたい区間を挟む(出力は変わらないので、元の版と byte 一致を確認できる):
//        let prof_out = std::env::var("PROF_OUT").ok();
//        if prof_out.is_some() { sprof::start(200); }      // 200 µs ごとを要求(実際の間隔は下の落とし穴)
//        heavy();
//        if let Some(p) = prof_out { sprof::dump(&p); }     // 番地を 1 行 1 個の 16 進で書く
//   2) **debug 情報つき**でビルドする(最適化はそのまま。掃引中の target を汚さないよう別の場所へ):
//        CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release --bin <bin> --target-dir <別の場所>
//   3) PROF_OUT=<file> <bin> < 入力 > /dev/null を何件か走らせ、
//        python3 scripts/sprof.py <bin> src/bin/<bin>.rs <file>...
//      で関数ごと・行ごとの割合を出す(addr2line が要る。binutils に入っている)。
//
// 落とし穴:
//   ・**x86_64 Linux 専用**(ucontext の中の RIP の位置と構造体の並びを決め打ちしている)。ほかの環境では何もしない。
//   ・標本は CPU 時間に対して採る(ITIMER_PROF)。🔴 **実際の間隔はカーネルの刻みで決まり、要求より粗い**
//     (WSL2 では 200 µs を要求しても 1 秒に約 100 個)。⇒ **1 件では足りない。何十件か走らせて、番地のファイルをまとめて渡す**
//     (1% の段を 50 個で読むには標本 5000 個 ＝ 1 秒の区間なら 50 件)。
//   ・共有ライブラリ(libm の sin など)の中の標本は `??` と出る(実行ファイルの外なので行に直せない)。
//   ・インライン展開された関数は addr2line の `-i` で内側から順に出る。`sprof.py` は「このソースの中で一番内側」を数える。
//   ・提出する版には貼らない(シグナルハンドラと 8 MB の静的バッファを持つ)。計測用の版(`NNs_samp.rs`)を別に作る。
mod sprof {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    mod imp {
        use std::sync::atomic::{AtomicUsize, Ordering};
        const CAP: usize = 1 << 20;
        static mut BUF: [u64; CAP] = [0; CAP];
        static N: AtomicUsize = AtomicUsize::new(0);
        #[repr(C)]
        struct SigAction { handler: usize, mask: [u64; 16], flags: i32, restorer: usize }
        #[repr(C)]
        struct TimeVal { sec: i64, usec: i64 }
        #[repr(C)]
        struct ITimerVal { interval: TimeVal, value: TimeVal }
        unsafe extern "C" {
            fn sigaction(sig: i32, act: *const SigAction, old: *mut SigAction) -> i32;
            fn setitimer(which: i32, new: *const ITimerVal, old: *mut ITimerVal) -> i32;
        }
        const SIGPROF: i32 = 27;
        const ITIMER_PROF: i32 = 2;
        const SA_SIGINFO: i32 = 4;
        const SA_RESTART: i32 = 0x1000_0000;
        extern "C" fn on_prof(_sig: i32, _info: *mut u8, uc: *mut u8) {
            // ucontext の gregs[REG_RIP] は先頭から 168 バイト目
            unsafe {
                let rip = *(uc.add(168) as *const u64);
                let n = N.fetch_add(1, Ordering::Relaxed);
                if n < CAP { (*std::ptr::addr_of_mut!(BUF))[n] = rip; }
            }
        }
        fn timer(us: i64) {
            let it = ITimerVal { interval: TimeVal { sec: 0, usec: us }, value: TimeVal { sec: 0, usec: us } };
            unsafe { setitimer(ITIMER_PROF, &it, std::ptr::null_mut()); }
        }
        pub fn start(us: i64) {
            let act = SigAction { handler: on_prof as *const () as usize, mask: [0; 16], flags: SA_SIGINFO | SA_RESTART, restorer: 0 };
            unsafe { sigaction(SIGPROF, &act, std::ptr::null_mut()); }
            timer(us);
        }
        pub fn dump(path: &str) {
            timer(0);
            // 位置独立な実行ファイルなので、読み込まれた先頭の番地を引いて addr2line に渡せる形にする
            let maps = std::fs::read_to_string("/proc/self/maps").unwrap_or_default();
            let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            let base = maps.lines().find(|l| l.ends_with(&exe))
                .and_then(|l| u64::from_str_radix(l.split('-').next().unwrap_or(""), 16).ok()).unwrap_or(0);
            let n = N.load(Ordering::Relaxed).min(CAP);
            let mut s = String::with_capacity(n * 8);
            for i in 0..n {
                let a = unsafe { (*std::ptr::addr_of!(BUF))[i] };
                s += &format!("{:x}\n", a.wrapping_sub(base));
            }
            let _ = std::fs::write(path, s);
        }
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    mod imp {
        pub fn start(_us: i64) {}
        pub fn dump(_path: &str) {}
    }
    /// `us` マイクロ秒(CPU 時間)ごとに命令の番地を 1 つ採り始める。
    pub fn start(us: i64) { imp::start(us) }
    /// 採るのを止め、番地(実行ファイルの先頭からの相対、16 進)を 1 行 1 個で `path` に書く。
    pub fn dump(path: &str) { imp::dump(path) }
}
