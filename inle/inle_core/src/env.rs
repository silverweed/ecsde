use std::boxed::Box;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Env_Info {
    pub full_exe_path: Box<Path>,
    pub working_dir: Box<Path>,
    pub assets_root: Box<Path>,
    pub cfg_root: Box<Path>,
}

impl Env_Info {
    pub fn gather() -> std::io::Result<Env_Info> {
        let full_exe_path = fs::canonicalize(env::current_exe()?)?;
        let working_dir = PathBuf::from(
            full_exe_path
                .parent()
                .unwrap_or_else(|| panic!("Wierd exe path: {:?}", full_exe_path)),
        );
        let assets_root = {
            let mut assets_root_buf = working_dir.clone();
            assets_root_buf.push("assets");
            assets_root_buf.into_boxed_path()
        };
        let cfg_root = {
            let mut cfgs_root_buf = working_dir.clone();
            cfgs_root_buf.push("cfg");
            cfgs_root_buf.into_boxed_path()
        };
        Ok(Env_Info {
            full_exe_path: full_exe_path.into_boxed_path(),
            working_dir: working_dir.into_boxed_path(),
            assets_root,
            cfg_root,
        })
    }
}
