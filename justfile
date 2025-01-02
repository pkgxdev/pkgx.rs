default:
  just -l

[group("benchmarks"), no-cd]
bench PKG:
  cargo build -r
  pkgx hyperfine -iw3 -- './target/release/pkgx +{{PKG}}'

[group("benchmarks"), no-cd]
graph *ARGS:
  cargo flamegraph --flamechart --root -- {{ARGS}}
  open flamegraph.svg

[group("benchmarks"), no-cd]
sample *ARGS:
  cargo build -r
  pkgx samply record ./target/release/pkgx {{ARGS}}