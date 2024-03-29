#[cfg(debug_assertions)]
use std::path;

#[derive(Default)]
pub(super) struct Cmdline_Args {
    #[cfg(debug_assertions)]
    pub in_replay_file: Option<Box<path::Path>>,

    pub n_entities_to_spawn: Option<usize>,
    pub start_from_menu: bool,
    pub verbose: bool,
}

macro_rules! opt_with_arg {
    ($opt: expr, $args: ident, $target: expr, $conv_fn: expr) => {{
        if let Some(opt) = $args.next() {
            $target = $conv_fn(opt);
            linfo!("Cmdline {}: {:?}", $opt, $target);
        } else {
            println!("\n\tExpected an argument after {} flag.\n", $opt);
            std::process::exit(1);
        }
    }};
}

pub(super) fn parse_cmdline_args<'a>(mut args: impl Iterator<Item = &'a String>) -> Cmdline_Args {
    // Consume program name
    args.next();

    let mut cmdline_args = Cmdline_Args::default();

    while let Some(arg) = args.next() {
        match arg as &str {
            #[cfg(debug_assertions)]
            "--replay" => opt_with_arg!("--replay", args, cmdline_args.in_replay_file, |path| {
                let mut pathbuf = path::PathBuf::new();
                pathbuf.push(path);
                Some(pathbuf.into_boxed_path())
            }),

            "--nentities" => opt_with_arg!(
                "--nentities",
                args,
                cmdline_args.n_entities_to_spawn,
                |n: &str| { n.parse::<usize>().ok() }
            ),

            "--from-menu" => {
                cmdline_args.start_from_menu = true;
            }

            "--verbose" => {
                cmdline_args.verbose = true;
            }

            _ => eprintln!("Unknown argument {}", arg),
        }
    }

    cmdline_args
}
