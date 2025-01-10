# What is `pkgx`?

`pkgx` is a 4MB, *standalone binary* that can *run anything*.

```sh
$ pkgx python@3.10 --version
Python 3.10.16
```

# Quick Start

```sh
$ brew install pkgxdev/made/pkgx || curl https://pkgx.sh | sh
```

[Getting Started](getting-started.md)


# Using `pkgx`

| <p>​<a href="running-anything.md">Run Anything</a><br>Run anything with `pkgx`</p> | <p><a href="scripting.md">Scripting</a><br>Write scripts in any language with all the tools you need available from line 1</p> |
| ----- | ----- |


# Using `dev`

`dev` uses shellcode and `pkgx` to create “virtual environments” for any
project and any toolset.

> https://github.com/pkgxdev/dev


# Using `pkgm`

`pkgm` installs `pkgx` packages to `/usr/local`.

> https://github.com/pkgxdev/pkgm


# Using `mash`

`mash` is a package manager for scripts that use `pkgx` to make the whole
open source ecosystem available to them.

> https://github.com/pkgxdev/mash
