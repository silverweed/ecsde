use std::boxed::Box;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Env_Info {
    full_exe_path: Box<Path>,
    working_dir: Box<Path>,
    assets_root: Box<Path>,
    cfg_root: Box<Path>,

    #[allow(dead_code)]
    test_paths: Test_Paths,
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
        let test_cfg = Self::build_test_cfg(&working_dir);
        Ok(Env_Info {
            full_exe_path: full_exe_path.into_boxed_path(),
            working_dir: working_dir.into_boxed_path(),
            assets_root,
            cfg_root,
            test_paths: Test_Paths::new(test_cfg),
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

    pub fn get_cfg_root(&self) -> &Path {
        &self.cfg_root
    }

    #[cfg(test)]
    pub fn get_test_cfg_root(&self) -> &Path {
        &self.test_paths.cfg_root
    }

    #[cfg(not(test))]
    fn build_test_cfg(_working_dir: &PathBuf) -> Option<Box<Path>> {
        None
    }

    #[cfg(test)]
    fn build_test_cfg(working_dir: &PathBuf) -> Option<Box<Path>> {
        let mut tests_root_buf = working_dir.clone();
        tests_root_buf.push("test_resources");
        tests_root_buf.push("cfg");
        Some(tests_root_buf.into_boxed_path())
    }
}

#[cfg(test)]
#[derive(Clone)]
struct Test_Paths {
    pub cfg_root: Box<Path>,
}

#[cfg(not(test))]
#[allow(dead_code)]
#[derive(Clone)]
struct Test_Paths {}

impl Test_Paths {
    #[cfg(test)]
    pub fn new(cfg_root: Option<Box<Path>>) -> Test_Paths {
        Test_Paths {
            cfg_root: cfg_root.unwrap(),
        }
    }

    #[cfg(not(test))]
    #[allow(dead_code)]
    pub fn new(_cfg_root: Option<Box<Path>>) -> Test_Paths {
        Test_Paths {}
    }
}
