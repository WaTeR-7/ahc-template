// 呼び出し回数と段別経過時間の計測(自前プロファイラ)。
//
// なぜ必要か: 「機構は正しいが遅い」と分かった時点で**推測で削ると外す**。ahc068 では1つのホットループの
// 中に O(N) の呼び出し 19.7M 回 と O(1) の呼び出し 25.3M 回 が同居していて、**どちらを間引くべきかは
// 計測しないと見えなかった**(回数が多い方は O(1) の側だった)。間引く側を特定できた結果、1行の変更で
// 実行時間が半分になり、空いた予算を閾値に回して +18.7M。**削る前に必ず「回数 × 1回のコスト」を測る。**
//
// 使い方:
//   1) 下の ON を true にしてビルドする(**false なら定数畳み込みで全て消える**ので、20M 回呼ばれる関数の
//      先頭に置いたままでも提出ビルドのコストはゼロ。測り終わったら false に戻す)。
//   2) NAMES を自分の段/関数名に書き換える。添字が計測スロット。
//   3) 数える:            prof::hit(C_MOVE);                    // 呼び出し1回
//                        prof::add(C_INNER, cand.len() as u64); // 内側ループの反復数をまとめて
//      時間を測る:        { let _g = prof::span(T_SEARCH); heavy(); }   // スコープを抜けた時に加算
//   4) main の末尾で       prof::report(timer.ms());            // stderr に表を出す
//
// 読み方: **合計の 90% 以上を占める段が1〜2個に絞れるまで細分する**。そこ以外を速くしても意味がない。
// 「回数」と「1回あたり」を分けて見る(回数が多いのに安い段は間引いても効かない)。
//
// スロットの約束:
//   ・**1スロット = 1つの事象**。同じスロットに `hit` と `span` を付けると「回数・時間・1回あたり」が
//     揃って出る(これが一番役に立つ)。別々の事象を同じスロットに混ぜると ns/call が無意味になる。
//   ・**span を入れ子にすると時間が二重計上される**(合計が 100% を超える)。割合で読みたい段は
//     互いに素にする。
mod prof {
    use std::cell::Cell;
    use std::time::Instant;

    /// プロファイル時だけ true。false なら `hit`/`add`/`span`/`report` は**完全に消える**(定数畳み込み)。
    pub const ON: bool = false;

    /// 計測スロット数(足りなければ増やす)。
    pub const N: usize = 8;
    /// スロット名。自分の段/関数名に書き換える(添字が計測スロット)。
    pub const NAMES: [&str; N] = ["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7"];

    thread_local! {
        static CNT: [Cell<u64>; N] = const { [const { Cell::new(0) }; N] };
        static NANOS: [Cell<u128>; N] = const { [const { Cell::new(0) }; N] };
    }

    /// 呼び出し回数を1つ数える(ホットな関数の先頭に置く)。
    #[inline(always)]
    pub fn hit(i: usize) {
        add(i, 1);
    }

    /// 回数を `w` だけ増やす(内側ループの反復数などをまとめて数える)。
    #[inline(always)]
    pub fn add(i: usize, w: u64) {
        if ON {
            CNT.with(|c| c[i].set(c[i].get() + w));
        }
    }

    /// 段の経過時間を測る。**戻り値を生かしたスコープ**を抜けた時に加算される。
    ///   { let _g = prof::span(T_SEARCH); heavy(); }   // `let _ =` にすると即 drop されるので注意
    #[inline(always)]
    pub fn span(i: usize) -> Span {
        Span { i, t: if ON { Some(Instant::now()) } else { None } }
    }

    pub struct Span {
        i: usize,
        t: Option<Instant>,
    }
    impl Drop for Span {
        #[inline(always)]
        fn drop(&mut self) {
            if let Some(t) = self.t {
                let ns = t.elapsed().as_nanos();
                NANOS.with(|v| v[self.i].set(v[self.i].get() + ns));
            }
        }
    }

    /// 現在値の読み出し(テスト・自作レポート用)。
    pub fn count(i: usize) -> u64 {
        CNT.with(|c| c[i].get())
    }
    pub fn nanos(i: usize) -> u128 {
        NANOS.with(|v| v[i].get())
    }
    pub fn reset() {
        CNT.with(|c| c.iter().for_each(|x| x.set(0)));
        NANOS.with(|v| v.iter().for_each(|x| x.set(0)));
    }

    /// stderr へ表を出す(`total_ms` は全体の経過時間。段の合計が全体の何%かを見る)。
    pub fn report(total_ms: u128) {
        if !ON {
            return;
        }
        eprintln!("--- prof (total {} ms) ---", total_ms);
        eprintln!("{:<12} {:>14} {:>10} {:>7} {:>12}", "slot", "count", "ms", "%", "ns/call");
        let mut sum_ns = 0u128;
        for (i, name) in NAMES.iter().enumerate() {
            let (c, ns) = (count(i), nanos(i));
            if c == 0 && ns == 0 {
                continue;
            }
            sum_ns += ns;
            let ms = ns as f64 / 1e6; // 1ms 未満の段を 0 に丸めない(細分の途中で消えると追えない)
            let pct = if total_ms > 0 { ms * 100.0 / total_ms as f64 } else { 0.0 };
            let per = if c > 0 { ns / c as u128 } else { 0 };
            eprintln!("{:<12} {:>14} {:>10.2} {:>6.1}% {:>12}", name, c, ms, pct, per);
        }
        eprintln!(
            "計測できた段の合計: {:.2} ms / 全体 {} ms（入れ子の span は二重計上）",
            sum_ns as f64 / 1e6,
            total_ms
        );
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        // ON=false(既定)でも API が壊れていないこと、および ON=true 相当の加算ロジックを検証する。
        // ON は const なので、ここでは加算そのものを直接叩いて確認する。
        #[test]
        fn counters_accumulate_and_reset() {
            reset();
            CNT.with(|c| c[1].set(c[1].get() + 3)); // add(1,3) の中身(ON に依らず検証したいので直接)
            NANOS.with(|v| v[1].set(v[1].get() + 1_500_000));
            assert_eq!(count(1), 3);
            assert_eq!(nanos(1), 1_500_000);
            reset();
            assert_eq!(count(1), 0);
            assert_eq!(nanos(1), 0);
        }

        // ON が定数 false であること自体を検査するテストなので、定数 assert の lint は意図的に許可する。
        #[allow(clippy::assertions_on_constants)]
        #[test]
        fn disabled_by_default_costs_nothing() {
            reset();
            hit(0);
            add(0, 100);
            {
                let _g = span(0);
            }
            // ON=false のときは何も記録されない(提出ビルドで副作用が無いことの確認)
            assert!(!ON);
            assert_eq!(count(0), 0);
            assert_eq!(nanos(0), 0);
            report(1); // 何も出力しない
        }
    }
}
