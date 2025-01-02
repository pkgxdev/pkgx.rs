# `pkgx` Rust Rewrite

We like deno (a lot), but the resulting binary is 90MB. With Rust:

* Potentially super fast
* Much smaller binary
* Lose the deno runtime initialization overhead (>50ms)
* No need to depend on external tools like `tar` and `git`

Using `deno` to figure out what `pkgx` is was the right call. Rewriting in
Rust now it is mature is also the right call.


## Migration Guide

We have omitted some features because we don’t use them and aren’t sure if
anyone else does.

Thus if a feature is missing you have come to depend on *let us know* and
we’ll add it back.

## `dev`

`dev` is now a dedicated separate tool that uses `pkgx` and is written in
`deno`.

## `pkgx --shellcode`

We found the shellcode to in general be a misfeature and have removed it.

## Installing Packages

Use Homebrew. We intend to write a separate tool called `pkgm` that will
install `pkgx` packages properly.

Writing your own stubs is still easy and viable.
