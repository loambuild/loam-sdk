use assert_cmd::{assert::Assert, Command};
use assert_fs::TempDir;
use fs_extra::dir::{copy, CopyOptions};
use std::path::PathBuf;

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub cwd: PathBuf,
}

pub trait AssertExt {
    fn stdout_as_str(&self) -> String;
    fn stderr_as_str(&self) -> String;
}

impl AssertExt for Assert {
    fn stdout_as_str(&self) -> String {
        String::from_utf8(self.get_output().stdout.clone())
            .expect("failed to make str")
            .trim()
            .to_owned()
    }
    fn stderr_as_str(&self) -> String {
        String::from_utf8(self.get_output().stderr.clone())
            .expect("failed to make str")
            .trim()
            .to_owned()
    }
}

impl TestEnv {
    pub fn new(template: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

        copy(template_dir.join(template), &temp_dir, &CopyOptions::new()).unwrap();

        Self {
            cwd: temp_dir.path().join(template),
            temp_dir,
        }
    }

    pub fn from<F: FnOnce(&TestEnv)>(template: &str, f: F) {
        let test_env = TestEnv::new(template);
        f(&test_env);
    }

    pub fn loam(&self, cmd: &str) -> Command {
        let mut loam = Command::cargo_bin("loam").unwrap();
        loam.current_dir(&self.cwd);
        loam.arg(cmd);
        loam
    }

    pub fn soroban(&self, cmd: &str) -> Command {
        let mut soroban = Command::new("soroban");
        soroban.env(
            "PATH",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/bin"),
        );
        soroban.current_dir(&self.cwd);
        soroban.arg(cmd);
        soroban
    }

    pub fn replace_file(&self, path1: &str, path2: &str) {
        let file_path1 = self.cwd.join(path1);
        let file_path2 = self.cwd.join(path2);
        let content = std::fs::read_to_string(&file_path1).expect("Failed to read file1");
        std::fs::write(&file_path2, content).expect("Failed to modify file");
    }

    pub fn set_environments_toml(&self, contents: impl AsRef<[u8]>) {
        std::fs::write(self.cwd.join("environments.toml"), contents).unwrap();
    }
}
