use std::env;
use std::fs;

pub struct Env_Info {
    full_exe_path: String,
    working_dir: String,
    assets_root: String,
}

impl Env_Info {
    pub fn gather() -> std::io::Result<Env_Info> {
        let exe = fs::canonicalize(env::current_exe()?)?;
        let full_exe_path = String::from(exe.to_str().unwrap());
        let slash_idx = full_exe_path.rfind('/').unwrap();
        let working_dir = String::from(&full_exe_path[..slash_idx]);
        let assets_root = format!("{}/{}", working_dir, "assets");
        Ok(Env_Info {
            full_exe_path,
            working_dir,
            assets_root,
        })
    }

    pub fn get_cwd(&self) -> &str {
        &self.working_dir
    }

    pub fn get_exe(&self) -> &str {
        &self.full_exe_path
    }

    pub fn get_assets_root(&self) -> &str {
        &self.assets_root
    }
}
