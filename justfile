default:
  just -l

[group("benchmarks"), no-cd]
bench PKG:
  cargo build -r
  pkgx hyperfine -iw3 -- './target/release/pkgx +{{PKG}}'

[group("benchmarks"), no-cd]
graph *ARGS:
  cargo flamegraph --flamecharto --root -- {{ARGS}}
  open flamegraph.svg