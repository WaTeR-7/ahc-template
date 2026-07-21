// env チューナブル読取り。パラメータ(TL/ビーム幅/温度…)を再コンパイル無しに掃引するため、
// env で外出しする(CLAUDE.md 方針)。未設定/パース失敗なら default にフォールバック。
// 使い方: let w: usize = env::get("AHC_W", 20); let tl: u128 = env::get("AHC_TL", 1900);
mod env {
    /// 環境変数 `key` を `T` にパースして返す。未設定・空・パース失敗はすべて `default`。
    pub fn get<T: std::str::FromStr>(key: &str, default: T) -> T {
        std::env::var(key).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
    }

    #[cfg(test)]
    mod tests {
        use super::get;
        // set_var は edition 2024 で unsafe(並行 setenv はプロセス全体で UB)。テストは並列実行される
        // ので、env を触る検証は 1 関数にまとめて逐次化する(他テストは env を参照しない)。
        #[test]
        fn get_reads_env_or_falls_back() {
            // 未設定 → default
            let v: u64 = get("AHC_UNSET_ZZZ_9137", 42);
            assert_eq!(v, 42);
            // パース失敗 → default
            unsafe {
                std::env::set_var("AHC_TEST_BAD_9137", "not_a_number");
            }
            let v: u64 = get("AHC_TEST_BAD_9137", 7);
            assert_eq!(v, 7);
            // 正常にパース(同じ文字列を別の型へも)
            unsafe {
                std::env::set_var("AHC_TEST_OK_9137", "123");
            }
            assert_eq!(get::<u64>("AHC_TEST_OK_9137", 0), 123);
            assert_eq!(get::<f64>("AHC_TEST_OK_9137", 0.0), 123.0);
        }
    }
}
