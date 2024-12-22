# `pkgx` Rust Rewrite

We like deno, but the resulting binary `pkgx` is 90MB. Rust will produce an
super fast binary that is much smaller removing the current “initialization
time” that the deno runtime imposes (which is >50ms).
