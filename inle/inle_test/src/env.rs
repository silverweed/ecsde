use inle_core::env::Env_Info;
use std::path::{Path, PathBuf};

pub fn get_test_cfg_root(env: &Env_Info) -> Box<Path> {
    let mut tests_root_buf = PathBuf::from(env.working_dir.clone());
    tests_root_buf.push("test_resources");
    tests_root_buf.push("cfg");
    tests_root_buf.into_boxed_path()
}
