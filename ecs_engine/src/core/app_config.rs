use std::iter::Iterator;

#[cfg(debug_assertions)]
use std::path;

pub struct App_Config {
    pub title: String,
    pub target_win_size: (u32, u32),
    #[cfg(debug_assertions)]
    pub in_replay_file: Option<Box<path::Path>>,
}

pub fn maybe_override_with_cmdline_args<'a>(
    cfg: &mut App_Config,
    mut args: impl Iterator<Item = &'a String>,
) {
    // Consume program name
    args.next();

    while let Some(arg) = args.next() {
        match arg as &str {
            "--title" => {
                if let Some(title) = args.next() {
                    cfg.title = title.to_string();
                } else {
                    eprintln!("Expected an argument after --title flag.");
                }
            }
            #[cfg(debug_assertions)]
            "--replay" => {
                if let Some(path) = args.next() {
                    let mut pathbuf = path::PathBuf::new();
                    pathbuf.push(path);
                    cfg.in_replay_file = Some(pathbuf.into_boxed_path());
                } else {
                    eprintln!("Expected an argument after --replay flag.");
                }
            }
            _ => eprintln!("Unknown argument {}", arg),
        }
    }
}
