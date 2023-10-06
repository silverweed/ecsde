use std::boxed::Box;
use std::env;
use std::ffi::OsStr;
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
        let mut working_dir = PathBuf::from(
            full_exe_path
                .parent()
                .unwrap_or_else(|| panic!("Wierd exe path: {:?}", full_exe_path)),
        );

        // Find out if we're in a dev environment and, if so, set the working dir to the repository
        // root (so we don't have to symlink/copy assets, cfg etc).
        // @Cleanup: this should be a dev-only thing, maybe turn it on with a feature flag?
        let cur_dir = working_dir.as_path().file_name().and_then(OsStr::to_str);
        let parent_dir = working_dir
            .as_path()
            .parent()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str);
        if matches!(cur_dir, Some("debug" | "release" | "profile" | "shipping"))
            && matches!(parent_dir, Some("target"))
        {
            working_dir.pop();
            working_dir.pop();
        } else if matches!(cur_dir, Some("deps"))
            && matches!(parent_dir, Some("debug" | "release" | "profile" | "shipping"))
        {
            working_dir.pop();
            working_dir.pop();
            working_dir.pop();
        }

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

// @Speed: when we have a frame temp allocator, this should probably allocate there.
#[inline]
pub fn asset_path(env: &Env_Info, dir: &str, file: &str) -> Box<Path> {
    let mut s = PathBuf::from(env.assets_root.as_ref());
    s.push(dir);
    s.push(file);
    s.into_boxed_path()
}

// @Speed: when we have a frame temp allocator, this should probably allocate there.
#[inline]
pub fn asset_dir_path(env: &Env_Info, dir: &str) -> Box<Path> {
    let mut s = PathBuf::from(env.assets_root.as_ref());
    s.push(dir);
    s.into_boxed_path()
}
