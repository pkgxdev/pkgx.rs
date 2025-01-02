pub enum Mode {
    X,
    Help,
    Version,
}

pub struct Flags {
    pub verbosity: i8,
}

pub fn parse() -> (Vec<String>, Vec<String>, Mode, Flags) {
    let mut mode = Mode::X;
    let mut plus = Vec::new();
    let mut args = Vec::new();
    let mut collecting_args = false;
    let mut verbosity: i8 = 0;

    for arg in std::env::args().skip(1) {
        if collecting_args {
            args.push(arg);
        } else if arg.starts_with('+') {
            plus.push(arg.trim_start_matches('+').to_string());
        } else if arg == "--" {
            collecting_args = true;
        } else if arg.starts_with("--") {
            match arg.as_str() {
                "--sync" => (),
                "--update" => (),
                "--help" => mode = Mode::Help,
                "--version" => mode = Mode::Version,
                "--verbose" => verbosity = 1,
                _ => panic!("unknown argument {}", arg),
            }
        } else if arg.starts_with('-') {
            // spit arg into characters
            for c in arg.chars().skip(1) {
                match c {
                    'S' => (),
                    'u' => (),
                    'h' => mode = Mode::Help,
                    'v' => mode = Mode::Version,
                    _ => panic!("unknown argument -{}", c),
                }
            }
        } else {
            collecting_args = true;
            args.push(arg);
        }
    }

    (plus, args, mode, Flags { verbosity })
}
