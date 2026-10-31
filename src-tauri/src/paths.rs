//! Platform application-data paths for tools that run without a Tauri
//! `AppHandle` (the CLI and the examples). The desktop app uses Tauri's
//! `app_data_dir`, which resolves to the same directory.

use std::path::PathBuf;

/// Overridable with `CHEDUM_DATA_DIR`, otherwise the platform default that
/// matches Tauri's bundle identifier.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("CHEDUM_DATA_DIR") {
        return PathBuf::from(dir);
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library/Application Support/com.chedum.studio")
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var_os("APPDATA").unwrap_or_default();
        PathBuf::from(appdata).join("com.chedum.studio")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
            PathBuf::from(data_home).join("com.chedum.studio")
        } else {
            let home = std::env::var_os("HOME").unwrap_or_default();
            PathBuf::from(home).join(".local/share/com.chedum.studio")
        }
    }
}

pub fn database_path() -> PathBuf {
    data_dir().join("chedum.db")
}
