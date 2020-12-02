#[macro_use]
extern crate lazy_static;

use ckb_tool::ckb_types::bytes::Bytes;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(test)]
mod tests;

lazy_static! {
    static ref LOADER: Loader = Loader::default();
    static ref TX_FOLDER: PathBuf = {
        let path = LOADER.path("dumped_tests");
        if Path::new(&path).exists() {
            fs::remove_dir_all(&path).expect("remove old dir");
        }
        fs::create_dir_all(&path).expect("create test dir");
        path
    };
}

const TEST_ENV_VAR: &str = "CAPSULE_TEST_ENV";

pub enum TestEnv {
    Debug,
    Release,
}

impl FromStr for TestEnv {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(TestEnv::Debug),
            "release" => Ok(TestEnv::Release),
            _ => Err("no match"),
        }
    }
}

pub struct Loader(PathBuf);

impl Default for Loader {
    fn default() -> Self {
        let test_env = match env::var(TEST_ENV_VAR) {
            Ok(val) => val.parse().expect("test env"),
            Err(_) => TestEnv::Debug,
        };
        Self::with_test_env(test_env)
    }
}

impl Loader {
    fn with_test_env(env: TestEnv) -> Self {
        let load_prefix = match env {
            TestEnv::Debug => "debug",
            TestEnv::Release => "release",
        };
        let dir = env::current_dir().unwrap();
        let mut base_path = PathBuf::new();
        base_path.push(dir);
        base_path.push("..");
        base_path.push("build");
        base_path.push(load_prefix);
        Loader(base_path)
    }

    pub fn path(&self, name: &str) -> PathBuf {
        let mut path = self.0.clone();
        path.push(name);
        path
    }

    pub fn load_binary(&self, name: &str) -> Bytes {
        fs::read(self.path(name)).expect("binary").into()
    }
}
