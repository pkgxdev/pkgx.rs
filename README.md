![pkgx.dev](https://pkgx.dev/banner.png)

`pkgx` is a 4.2MB, *standalone* binary that can *run anything*.
&nbsp;&nbsp;[![coverage][]][coveralls] [![teaRank][]](https://tea.xyz)

&nbsp;


### Quickstart

```sh
brew install pkgxdev/made/pkgx || sh <(curl https://pkgx.sh)
```

> [docs.pkgx.sh/installing-w/out-brew]

&nbsp;


# Run Anything

```sh
$ deno
command not found: deno

$ pkgx deno
Deno 1.36.3
> ^D

$ deno
command not found: deno
# ^^ nothing was installed; your system remains untouched
```


## Run *Any Version* of Anything

```sh
$ pkgx node@14 --version
Node.js v14.21.3

$ pkgx python@2 --version
Python 2.7.18
```


## Run Anywhere

* <details><summary>macOS</summary><br>

  * macOS >= 11
  * x86-64 & Apple Silicon

  </details>
* <details><summary>Linux</summary><br>

  * glibc >=2.28 [repology](https://repology.org/project/glibc/versions)
  * `x86_64` & `arm64`

  </details>
* <details><summary>Windows</summary><br>

  WSL2; x86-64. *Native windows is planned.*

  </details>
* <details><summary>Docker</summary><br>

  ```sh
  $ pkgx docker run -it pkgxdev/pkgx

  (docker) $ pkgx node@16
  Welcome to Node.js v16.20.1.
  Type ".help" for more information.
  >
  ```

  Or in a `Dockerfile`:

  ```Dockerfile
  FROM pkgxdev/pkgx
  RUN pkgx deno@1.35 task start
  ```

  Or in any image:

  ```Dockerfile
  FROM ubuntu
  RUN curl https://pkgx.sh | sh
  RUN pkgx python@3.10 -m http.server 8000
  ```

  > [docs.pkgx.sh/docker]

  </details>
* <details><summary>CI/CD</summary><br>

  ```yaml
  - uses: pkgxdev/setup@v1
  - run: pkgx shellcheck
  ```

  Or in other CI/CD providers:

  ```sh
  $ curl https://pkgx.sh | sh
  $ pkgx shellcheck
  ```

  > [docs.pkgx.sh/ci-cd]

  </details>
* <details><summary>Scripts</summary><br>

  ```sh
  #!/usr/bin/env -S pkgx +git python@3.12

  # python 3.12 runs the script and `git` is available during its execution
  ```

  > [docs.pkgx.sh/scripts]

  </details>
* <details><summary>Editors</summary><br>

  ```sh
  $ cd myproj

  myproj $ env +cargo
  (+cargo) myproj $ code .
  ```

  Or use [`dev`][dev]; a separate tool that uses the pkgx primitives to
  automatically determine and utilize your dependencies based on your
  project’s keyfiles.

  ```sh
  $ cd myproj

  myproj $ dev
  env +cargo +rust

  (+cargo+rust) my-rust-project $ code .
  ```

  > [docs.pkgx.sh/editors]

  </details>

&nbsp;


## `dev`

`dev` is a separate tool that leverages pkgx's core
features to auto-detect and install project dependencies, seamlessly
integrating them into your shell and editor.

```sh
my-rust-proj $ dev    # do `pkgx integrate --dry-run` first
dev: found Cargo.toml; env +cargo +rust

(+cargo+rust) my-rust-proj $ cargo build
Compiling my-rust-proj v0.1.0
#…
```

The `dev` tool requires shell integration to work.

> [docs.pkgx.sh/dev][dev]

## `pkgm`

`pkgm` installs `pkgx` packages to `/usr/local`. It installs alongside `pkgx`.

> [github.com/pkgxdev/pkgm](https://github.com/pkgxdev/pkgm)

## Scripting

One of the most powerful uses of `pkgx` is scripting, eg. here’s a script
to release new versions to GitHub:

```sh
#!/usr/bin/env -S pkgx +gum +gh +npx +git bash -eo pipefail

gum format "# Welcome to our Release Script"

versions="$(git tag | grep '^v[0-9]\+\.[0-9]\+\.[0-9]\+')"
v_latest="$(npx -- semver --include-prerelease $versions | tail -n1)"
v_new=$(npx -- semver bump $v_latest --increment $1)

gh release create \
  $v_new \
  --title "$v_new Released 🎉" \
  --generate-notes \
  --notes-start-tag=v$v_latest
```

Above you can see how we “loaded” the shebang with `+pkg` syntax to bring in
all the tools we needed.

> We have pretty advanced versions of the above script, eg
> [teaBASE][teaBASE-release-script]

There’s tools for just about every language ecosystem so you can import
dependencies, eg. using `uv` to import python packages (and the specific
python you want too):

```sh
#!/usr/bin/env -S pkgx +python@3.11 uv run --script

# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "requests<3",
#   "rich",
# ]
# ///

import requests
from rich.pretty import pprint

resp = requests.get("https://peps.python.org/api/peps.json")
data = resp.json()
pprint([(k, v["title"]) for k, v in data.items()][:10])
```

## Magic

It can be fun to add magic to your shell:

```sh
# add to ~/.zshrc
command_not_found_handler() {
  pkgx -- "$@"
}
```

Thus if you type `gh` and not in your `PATH` pkgx will install it and run it
as though it was installed all along.

> [!NOTE]
> Bash is the same function but drop the `r` from the end of the name.

&nbsp;


## Further Reading

[docs.pkgx.sh][docs] is a comprehensive manual and user guide for `pkgx`.

&nbsp;



# Contributing

* To add packages see the [pantry README]
* To hack on `pkgx` itself; clone it and we’re built in rust

If you have questions or feedback:

* [github.com/orgs/pkgxdev/discussions][discussions]
* [x.com/pkgxdev](https://x.com/pkgxdev) (DMs are open)


[docs]: https://docs.pkgx.sh
[pantry README]: ../../../pantry#contributing
[discussions]: ../../discussions
[docs.pkgx.sh/pkgx-install]: https://docs.pkgx.sh/pkgx-install
[docs.pkgx.sh/ci-cd]: https://docs.pkgx.sh/ci-cd
[docs.pkgx.sh/scripts]: https://docs.pkgx.sh/scripts
[docs.pkgx.sh/editors]: https://docs.pkgx.sh/editors
[docs.pkgx.sh/docker]: https://docs.pkgx.sh/docker
[docs.pkgx.sh/installing-w/out-brew]: https://docs.pkgx.sh/installing-w/out-brew
[docs.pkgx.sh/shell-integration]: https://docs.pkgx.sh/shell-integration
[dev]: https://docs.pkgx.sh/dev
[teaBASE-release-script]: https://github.com/teaxyz/teaBASE/blob/main/Scripts/publish-release.sh
[Scriptisto]: https://github.com/igor-petruk/scriptisto

[coverage]: https://coveralls.io/repos/github/pkgxdev/pkgx/badge.svg?branch=main
[coveralls]: https://coveralls.io/github/pkgxdev/pkgx?branch=main
[teaRank]: https://img.shields.io/endpoint?url=https%3A%2F%2Fchai.tea.xyz%2Fv1%2FgetTeaRankBadge%3FprojectId%3D79e9363b-862c-43e0-841d-4d4eaad1fc95
