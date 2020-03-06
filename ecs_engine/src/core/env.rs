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

    #[cfg(test)]
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
        #[cfg(test)]
        let test_cfg = Self::build_test_cfg(&working_dir);
        Ok(Env_Info {
            full_exe_path: full_exe_path.into_boxed_path(),
            working_dir: working_dir.into_boxed_path(),
            assets_root,
            cfg_root,
            #[cfg(test)]
            test_paths: Test_Paths::new(test_cfg),
        })
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

#[cfg(test)]
impl Test_Paths {
    pub fn new(cfg_root: Option<Box<Path>>) -> Test_Paths {
        Test_Paths {
            cfg_root: cfg_root.unwrap(),
        }
    }
}
