# `pkgx` Rust Rewrite

We like deno (a lot), but the resulting binary is 90MB. Rust will produce a
super fast binary that is much smaller while removing the current
“initialization time” that the deno runtime imposes (which is >50ms).
