use assert_cmd::{assert::Assert, Command};
use assert_fs::TempDir;
use fs_extra::dir::{copy, CopyOptions};
use std::future::Future;
use std::path::PathBuf;
use tokio::process::Command as ProcessCommand;

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

    pub async fn from_async<F, Fut>(template: &str, f: F)
    where
        F: FnOnce(TestEnv) -> Fut,
        Fut: Future<Output = ()>,
    {
        let test_env = TestEnv::new(template);
        f(test_env).await;
    }

    pub fn modify_file(&self, path: &str, content: &str) {
        let file_path = self.cwd.join(path);
        std::fs::write(file_path, content).expect("Failed to modify file");
    }

    pub fn delete_file(&self, path: &str) {
        let file_path = self.cwd.join(path);
        std::fs::remove_file(file_path).expect("Failed to delete file");
    }

    pub fn loam(&self, cmd: &str) -> Command {
        let mut loam = Command::cargo_bin("loam").unwrap();
        loam.current_dir(&self.cwd);
        loam.arg(cmd);
        loam
    }

    fn cargo_bin_loam(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_BIN_EXE_loam"))
    }

    pub fn loam_process(&self, cmd: &str) -> ProcessCommand {
        println!("{}", self.cargo_bin_loam().display());
        let mut loam = ProcessCommand::new(self.cargo_bin_loam());
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

    pub fn set_environments_toml(&self, contents: impl AsRef<[u8]>) {
        std::fs::write(self.cwd.join("environments.toml"), contents).unwrap();
    }
}
