use std::path;

pub struct App_Config {
    pub title: String,
    pub target_win_size: (u32, u32),
    pub replay_file: Option<Box<path::Path>>,
}

impl App_Config {
    pub fn new(mut args: std::env::Args) -> App_Config {
        let mut cfg = App_Config {
            title: String::from("Unnamed app"),
            target_win_size: (800, 600),
            replay_file: None,
        };

        // Consume program name
        args.next();

        while let Some(arg) = args.next() {
            match arg.as_ref() {
                "--title" => {
                    if let Some(title) = args.next() {
                        cfg.title = title;
                    } else {
                        eprintln!("Expected an argument after --title flag.");
                    }
                }
                "--replay" => {
                    if let Some(path) = args.next() {
                        let mut pathbuf = path::PathBuf::new();
                        pathbuf.push(path);
                        cfg.replay_file = Some(pathbuf.into_boxed_path());
                    } else {
                        eprintln!("Expected an argument after --replay flag.");
                    }
                }
                _ => eprintln!("Unknown argument {}", arg),
            }
        }

        cfg
    }
}
