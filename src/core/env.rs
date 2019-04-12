use std::boxed::Box;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Env_Info {
    full_exe_path: Box<Path>,
    working_dir: Box<Path>,
    assets_root: Box<Path>,
}

impl Env_Info {
    pub fn gather() -> std::io::Result<Env_Info> {
        let full_exe_path = fs::canonicalize(env::current_exe()?)?;
        let working_dir = PathBuf::from(full_exe_path.parent()
		.expect(&format!("Wierd exe path: {:?}", full_exe_path)));
	let mut assets_root_buf = PathBuf::from(working_dir.clone());
	assets_root_buf.push("assets");
        let assets_root = assets_root_buf.into_boxed_path();
        Ok(Env_Info {
            full_exe_path: full_exe_path.into_boxed_path(),
            working_dir: working_dir.into_boxed_path(),
            assets_root,
        })
    }

    pub fn get_cwd(&self) -> &Path {
        &self.working_dir
    }

    pub fn get_exe(&self) -> &Path {
        &self.full_exe_path
    }

    pub fn get_assets_root(&self) -> &Path {
        &self.assets_root
    }
}
