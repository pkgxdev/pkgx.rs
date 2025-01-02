# Benchmarks

## Original

```sh
% just bench facebook.com/watchman
cargo build -r
   Compiling pkgx v2.0.0-rc (/Users/jacob/pkgx/pkgx.rs)
    Finished `release` profile [optimized] target(s) in 4.35s
pkgx hyperfine -iw3 -- './target/release/pkgx +facebook.com/watchman'
Benchmark 1: ./target/release/pkgx +facebook.com/watchman
  Time (mean ± σ):      10.6 ms ±   4.1 ms    [User: 4.1 ms, System: 4.7 ms]
  Range (min … max):     6.9 ms …  36.3 ms    184 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
```

## DashMap

insignificant gains (0.5ms on average)

## libsqlite

cargo flamegraph shows a fair amount of time in loading the sqlite3 library. However, statically linking the
library doesn't seem to improve performance.

the majority of the flamegraph does seem to be invovled with sqlite3. that's _good_ in general, since it means that we're
bounded by disk access. i'm not sure we can improve on this, since without data we're not a program.

leaving the static linking in `Cargo.toml` for now, because this is Rust.
