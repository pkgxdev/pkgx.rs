# `pkgx` Rust Rewrite

We like deno (a lot), but the resulting binary is 90MB. With Rust:

* Potentially super fast
* Much smaller binary
* Lose the deno runtime initialization overhead (>50ms)
* No need to depend on external tools like `tar` and `git`

Using `deno` to figure out what `pkgx` is was the right call. Rewriting in
Rust now it is mature is also the right call.
