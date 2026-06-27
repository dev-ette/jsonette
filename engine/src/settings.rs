/*
 * Copyright (c) 2026 DevEtte.
 *
 * This project is dual-licensed under both the MIT License and the
 * Apache License, Version 2.0 (the "License"). You may not use this
 * file except in compliance with one of these licenses.
 *
 * You may obtain a copy of the Licenses at:
 * - MIT: https://opensource.org
 * - Apache 2.0: http://apache.org
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Thread-safe global settings singleton and configuration cache manager.

use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use crate::types::AppSettings;

/// Global static container holding the active application preferences in-memory.
/// Wrapped in `OnceLock` for safe initialization and `RwLock` for thread-safe read/write access.
static SETTINGS: OnceLock<RwLock<AppSettings>> = OnceLock::new();

/// Singleton manager for system settings matching the StarUML architecture model.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// The active configuration preferences.
    pub settings: AppSettings,
}

impl Settings {
    /// Returns the global singleton instance of the `Settings` struct.
    /// Resolves preferences lazily from disk on the first invocation.
    ///
    /// # Returns
    ///
    /// A `Settings` struct containing a copy of the active application settings.
    pub fn get_settings() -> Self {
        let lock = Self::get_settings_lock();
        let read_guard = lock.read().expect("Failed to acquire settings read lock");
        Self {
            settings: *read_guard,
        }
    }

    /// Returns a reference to the global thread-safe RwLock container.
    /// Initializes lazily on the first request by trying to load settings from disk.
    /// Falls back to defaults if parsing or loading fails.
    ///
    /// # Returns
    ///
    /// A reference to the static RwLock containing the configuration.
    pub fn get_settings_lock() -> &'static RwLock<AppSettings> {
        SETTINGS.get_or_init(|| {
            let settings = Self::load_settings_from_disk().unwrap_or_else(|_| {
                let defaults = AppSettings::default();
                let _ = Self::save_settings_to_disk(&defaults);
                defaults
            });
            RwLock::new(settings)
        })
    }

    /// Retrieves the home directory path in a platform-independent way.
    /// On Unix/macOS, reads the `HOME` environment variable.
    /// On Windows, reads `USERPROFILE`, falling back to `HOMEDRIVE` + `HOMEPATH`.
    ///
    /// # Returns
    ///
    /// An optional `PathBuf` pointing to the user's home folder.
    #[allow(dead_code)]
    fn get_home_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("USERPROFILE")
                .ok()
                .or_else(|| {
                    let drive = std::env::var("HOMEDRIVE").ok()?;
                    let path = std::env::var("HOMEPATH").ok()?;
                    Some(format!("{}{}", drive, path))
                })
                .map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
    }

    /// Returns the absolute path to the settings file on disk.
    /// Resolves to the user's home folder in normal execution, or to a process-isolated file
    /// in the system temporary directory during tests to avoid concurrent test collision.
    ///
    /// # Returns
    ///
    /// An optional `PathBuf` pointing to the settings file.
    fn get_settings_path() -> Option<PathBuf> {
        #[cfg(test)]
        {
            let mut path = std::env::temp_dir();
            path.push(format!("jsonette_test_settings_{}.json", std::process::id()));
            Some(path)
        }
        #[cfg(not(test))]
        {
            let mut path = Self::get_home_dir()?;
            path.push(".jsonette_settings.json");
            Some(path)
        }
    }

    /// Reads the configuration file from disk and parses it.
    /// If the file does not exist, writes the default options to disk first and returns them.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `AppSettings` or an error string.
    fn load_settings_from_disk() -> Result<AppSettings, String> {
        let path = Self::get_settings_path().ok_or_else(|| "Unable to locate home directory".to_string())?;
        if !path.exists() {
            let defaults = AppSettings::default();
            Self::save_settings_to_disk(&defaults)?;
            return Ok(defaults);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        let settings: AppSettings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;
        Ok(settings)
    }

    /// Serializes and writes the provided configuration options to disk.
    ///
    /// # Arguments
    ///
    /// * `settings` - A reference to the `AppSettings` structure to write to disk.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error string.
    fn save_settings_to_disk(settings: &AppSettings) -> Result<(), String> {
        let path = Self::get_settings_path().ok_or_else(|| "Unable to locate home directory".to_string())?;
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        Ok(())
    }
}

/// Retrieves a copy of the current global configuration settings.
/// Resolves preferences lazily from disk on the first invocation.
///
/// # Returns
///
/// A copied `AppSettings` struct reflecting the active preferences.
pub fn get_settings() -> AppSettings {
    Settings::get_settings().settings
}

/// Updates the in-memory preferences and persists the changes to disk.
///
/// # Arguments
///
/// * `settings` - The new `AppSettings` configuration to apply globally.
///
/// # Returns
///
/// A `Result` indicating success or containing an error string.
pub fn update_settings(settings: AppSettings) -> Result<(), String> {
    let lock = Settings::get_settings_lock();
    {
        let mut write_guard = lock.write().expect("Failed to acquire settings write lock");
        *write_guard = settings;
    }
    Settings::save_settings_to_disk(&settings)
}
