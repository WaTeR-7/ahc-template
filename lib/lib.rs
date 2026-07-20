// cplib: 再利用部品を束ねる cargo の lib ターゲット(検証専用の“正本”)。
//   ・各部品は lib/<name>.rs に「そのまま貼れる `mod` ブロック」で置く。
//   ・ここで include! して 1 crate にまとめ、`cargo check` / `cargo test` で通常どおり型検査・テストする。
//   ・解答は この lib を `use` せず、必要な `mod` ブロックを lib/ から**コピペ**する
//     (自己完結でそのまま提出でき、その場で改造もできる)。部品を足したら include! を1行追加。
#![allow(dead_code, unused)]

include!("io.rs");
include!("rng.rs");
include!("timer.rs");
