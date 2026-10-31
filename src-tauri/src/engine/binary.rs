//! Locates the Stockfish executable on macOS, Linux and Windows.
//!
//! Release bundles ship Stockfish as a Tauri sidecar (`externalBin`): the
//! bundler copies `binaries/chedum-stockfish-<target>` next to the application
//! executable and strips the target triple. During development the binary
//! stays under `src-tauri/binaries` with the triple in its name.
//!
//! The sidecar is called `chedum-stockfish` rather than `stockfish` so a Linux
//! package never conflicts with a system Stockfish installation.

use std::path::{Path, PathBuf};

use tauri::Manager;

/// File name produced by `npm run fetch:stockfish` for the current target,
/// for example `chedum-stockfish-aarch64-apple-darwin` or
/// `chedum-stockfish-x86_64-pc-windows-msvc.exe`.
pub fn bundled_file_name() -> String {
    let target = env!("CHEDUM_TARGET");
    if target.contains("windows") {
        format!("chedum-stockfish-{target}.exe")
    } else {
        format!("chedum-stockfish-{target}")
    }
}

/// Development location: `src-tauri/binaries/<bundled file name>`.
pub fn bundled_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(bundled_file_name())
}

/// Name of the sidecar once Tauri has bundled it next to the app executable.
fn sidecar_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "chedum-stockfish.exe"
    } else {
        "chedum-stockfish"
    }
}

fn path_lookup(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Finds a usable Stockfish binary. `CHEDUM_STOCKFISH` always wins, which is
/// also how the CLI tests and the accuracy scripts point at a custom build.
pub fn discover(app: &tauri::AppHandle) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(path) = std::env::var_os("CHEDUM_STOCKFISH") {
        candidates.push(PathBuf::from(path));
    }

    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        candidates.push(dir.join(sidecar_name()));
        candidates.push(dir.join(bundled_file_name()));
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join(bundled_file_name()));
        candidates.push(resource_dir.join("binaries").join(bundled_file_name()));
        candidates.push(resource_dir.join(sidecar_name()));
        candidates.push(resource_dir.join("binaries").join(sidecar_name()));
    }

    candidates.push(bundled_path());

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("binaries").join(bundled_file_name()));
        candidates.push(
            cwd.join("src-tauri")
                .join("binaries")
                .join(bundled_file_name()),
        );
        candidates.push(cwd.join("binaries").join(sidecar_name()));
    }

    if let Some(path) = path_lookup(sidecar_name()) {
        candidates.push(path);
    }

    #[cfg(target_os = "macos")]
    {
        candidates.push(PathBuf::from("/opt/homebrew/bin/stockfish"));
        candidates.push(PathBuf::from("/usr/local/bin/stockfish"));
    }

    candidates.into_iter().find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_name_matches_the_host_target() {
        let name = bundled_file_name();
        assert!(name.starts_with("chedum-stockfish-"));
        assert_eq!(
            name.ends_with(".exe"),
            cfg!(target_os = "windows"),
            "only Windows sidecars keep the .exe suffix"
        );
    }
}
