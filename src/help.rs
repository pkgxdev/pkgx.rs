use regex::Regex;

fn dim(input: &str) -> String {
    // Placeholder function for "dim" styling
    format!("\x1b[2m{}\x1b[0m", input)
}

pub fn usage(verbosity: i8) -> String {
    if verbosity <= 0 {
        let usage = r##"
usage:
  pkgx [+pkg@x.y…] <program|path> [--] [arg…]

examples:
  $ pkgx gum format "# hello world" "sup?"
  $ pkgx node@18 --eval 'console.log("hello world")'
  $ pkgx +openssl cargo build

more:
  $ pkgx --help --verbose
  $ open https://docs.pkgx.sh
"##;

        usage
            .replace('[', &dim("["))
            .replace(']', &dim("]"))
            .replace('<', &dim("<"))
            .replace('>', &dim(">"))
            .replace('$', &dim("$"))
            .replace('|', &dim("|"))
    } else {
        let usage = r#"
usage:
  pkgx [+pkg@x.y…] <program|path> [--] [arg…]

  • assembles the requested environment, installing packages as necessary
  • automatically determines additional packages based on the args
  • executes program and args in that environment

flags:
  -S, --sync         # synchronize pantry †
  -u, --update       # attempt to update pkgs
  -v, --verbose[=n]  # set verbosity ‡

  # • repetitions override previous values

  # † typically not needed, we automatically synchronize when appropriate
  # ‡ see VERBOSE

aliases:
  --silent   # no chat, no errors, no output; just exit code (--verbose=-2) §
  --quiet    # minimal chat, errors, & output (--verbose=-1)

  # § silences pkgx, *not the executed program*

alt. modes:
  --help     # hi mom!
  --version  # prints pkgx’s version

environments variables:
  PKGX_DIR   # cache pkgs here, defaults to ~/.pkgx
  VERBOSE    # {-2: silent, -1: quietish, 0: default, 1: verbose, 2: debug}
  DEBUG      # alias for `VERBOSE=2`

  # • explicit flags override any environment variables

environmental influencers:
  CI         # defaults verbosity to -1 (--quiet)
  CLICOLOR   # see https://bixense.com/clicolors
"#;

        // not bothering to wrap this in a lazy_static! block
        // because it's only used once
        let re = Regex::new("(?m)#.*$").unwrap();

        re.replace_all(&usage, |caps: &regex::Captures| {
            dim(caps.get(0).unwrap().as_str())
        })
        .to_string()
    }
}
