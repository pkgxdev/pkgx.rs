pub enum Mode {
    X,
    Help,
    Version,
}

pub struct Flags {
    pub verbosity: i8,
}

pub struct Args {
    pub plus: Vec<String>,
    pub args: Vec<String>,
    pub find_program: bool,
    pub mode: Mode,
    pub flags: Flags,
}

pub fn parse() -> Args {
    let mut mode = Mode::X;
    let mut plus = Vec::new();
    let mut args = Vec::new();
    let mut verbosity: i8 = 0;
    let mut find_program = false;
    let mut collecting_args = false;

    for arg in std::env::args().skip(1) {
        if collecting_args {
            args.push(arg);
        } else if arg.starts_with('+') {
            plus.push(arg.trim_start_matches('+').to_string());
        } else if arg == "--" {
            find_program = false;
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
                    _ => panic!("unknown argument: -{}", c),
                }
            }
        } else {
            find_program = !arg.contains('/');
            collecting_args = true;
            args.push(arg);
        }
    }

    Args {
        plus,
        args,
        find_program,
        mode,
        flags: Flags { verbosity },
    }
}
