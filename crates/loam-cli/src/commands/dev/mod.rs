use clap::Parser;
use notify::{self, RecursiveMode, Watcher};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time;

use crate::commands::build;

use super::build::clients::LoamEnv;

enum Message {
    FileChanged,
}

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    #[command(flatten)]
    pub build_cmd: build::Cmd,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Watcher(#[from] notify::Error),
    #[error(transparent)]
    Build(#[from] build::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

fn canonicalize_path(path: &Path) -> PathBuf {
    if path.as_os_str().is_empty() {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else if path.components().count() == 1 {
        // Path is a single component, assuming it's a filename
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    } else {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
}

fn is_parent_in_watched_dirs(parent: &Path, watched_dirs: &[Arc<PathBuf>]) -> bool {
    watched_dirs.iter().any(|p| canonicalize_path(p) == parent)
}

fn is_temporary_file(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    // Vim temporary files
    if file_name.starts_with('.') {
        return true;
    }
    if file_name.ends_with('~') {
        return true;
    }

    // Emacs temporary files
    if file_name.starts_with('#') && file_name.ends_with('#') {
        return true;
    }
    // VSCode temporary files
    if std::path::Path::new(file_name)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("tmp"))
    {
        return true;
    }

    // Add more patterns for other editors as needed

    false
}

impl Cmd {
    pub async fn run(&mut self) -> Result<(), Error> {
        let (tx, mut rx) = mpsc::channel::<Message>(100);
        let rebuild_state = Arc::new(Mutex::new(false));
        let workspace_root: &Path = self
            .build_cmd
            .manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let env_toml_path = Arc::new(workspace_root.join("environments.toml"));
        let env_toml_parent = Arc::new(
            env_toml_path
                .parent()
                .unwrap_or(Path::new(""))
                .to_path_buf(),
        );

        let mut watched_dirs = Vec::new();
        let packages = self
            .build_cmd
            .list_packages()?
            .into_iter()
            .map(|package| {
                Arc::new(PathBuf::from(
                    package.manifest_path.parent().unwrap().as_str(),
                ))
            })
            .collect::<Vec<_>>();

        for package_path in &packages {
            watched_dirs.push(package_path.clone());
            eprintln!("Watching {}", package_path.as_path().display());
        }
        let watched_dirs_clone = watched_dirs.clone();
        let env_toml_path_clone = Arc::clone(&env_toml_path);
        let env_toml_parent_clone = Arc::clone(&env_toml_parent);
        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(
                        event.kind,
                        notify::EventKind::Create(_)
                            | notify::EventKind::Modify(_)
                            | notify::EventKind::Remove(_)
                    ) {
                        if let Some(path) = event.paths.first() {
                            // Ignore temporary files
                            if is_temporary_file(path) {
                                return;
                            }
                            let env_toml_parent_abs = canonicalize_path(&env_toml_parent_clone);
                            let env_toml_path_abs = canonicalize_path(&env_toml_path_clone);
                            let parent_is_env_toml_parent =
                                path.parent() == Some(env_toml_parent_abs.as_path());
                            let path_is_env_toml = path == env_toml_path_abs.as_path();
                            let parent_is_in_watched_dirs = is_parent_in_watched_dirs(
                                &env_toml_parent_abs,
                                &watched_dirs_clone,
                            );
                            // skip if the file is in the parent directory of environments.toml and it is not environments.toml
                            if !(parent_is_env_toml_parent
                                && !path_is_env_toml
                                && !parent_is_in_watched_dirs)
                            {
                                eprintln!("File changed: {path:?}");
                                if let Err(e) = tx.blocking_send(Message::FileChanged) {
                                    eprintln!("Error sending through channel: {e}");
                                }
                            }
                        }
                    }
                }
            })
            .unwrap();
        // Watch the parent directory of environments.toml
        watcher.watch(env_toml_parent.as_path(), RecursiveMode::NonRecursive)?;
        for package_path in watched_dirs {
            watcher.watch(package_path.as_path(), RecursiveMode::Recursive)?;
        }

        let build_command = self.cloned_build_command();
        let cmd = build_command.lock().await;
        if let Err(e) = cmd.run().await {
            eprintln!("Build error: {e}");
        }
        eprintln!("Watching for changes. Press Ctrl+C to stop.");

        let rebuild_state_clone = rebuild_state.clone();
        loop {
            tokio::select! {
                _ = rx.recv() => {
                    let mut state = rebuild_state_clone.lock().await;
                    let build_command_inner = self.cloned_build_command();
                    if !*state {
                        *state= true;
                        tokio::spawn(Self::debounced_rebuild(build_command_inner, Arc::clone(&rebuild_state_clone)));
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    eprintln!("Stopping dev mode.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn debounced_rebuild(
        build_command: Arc<Mutex<build::Cmd>>,
        rebuild_state: Arc<Mutex<bool>>,
    ) {
        // Debounce to avoid multiple rapid rebuilds
        time::sleep(std::time::Duration::from_secs(1)).await;

        eprintln!("Changes detected. Rebuilding...");
        let cmd = build_command.lock().await;
        if let Err(e) = cmd.run().await {
            eprintln!("Build error: {e}");
        }
        eprintln!("Watching for changes. Press Ctrl+C to stop.");

        let mut state = rebuild_state.lock().await;
        *state = false;
    }

    fn cloned_build_command(&mut self) -> Arc<Mutex<build::Cmd>> {
        self.build_cmd
            .build_clients
            .env
            .get_or_insert(LoamEnv::Development);
        self.build_cmd
            .profile
            .get_or_insert_with(|| "debug".to_string());
        Arc::new(Mutex::new(self.build_cmd.clone()))
    }
}
