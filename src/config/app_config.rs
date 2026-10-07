use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::GitSetting,
    utils::{
        app_theme::AppTheme, date_format::DateFormat, lanuages::Languages, time_format::TimeFormat,
    },
};

use super::{capture_setting::CaptureSetting, chrome_config::ChromeConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub theme: AppTheme,
    pub time_format: TimeFormat,
    pub date_format: DateFormat,
    pub language: Languages,
    pub chrome_config: ChromeConfig,
    pub capture_setting: CaptureSetting,
    pub git_setting: GitSetting,
}

impl AppConfig {
    const CONFIG_FILE: &'static str = "app_config.json";
    const CONFIG_DIR: &'static str = "hazyio_dockeep";
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .expect("Could not find a platform config directory; using defaults")
            .join(Self::CONFIG_DIR)
    }
    fn config_path() -> PathBuf {
        Self::config_dir().join(Self::CONFIG_FILE)
    }
    pub fn load() -> Self {
        Self::load_from(&Self::config_path())
    }
    pub fn log_dir() -> PathBuf {
        Self::config_dir().join("logs")
    }

    pub fn save(&self) {
        self.save_to(&Self::config_path());
    }
    fn load_from(config_path: &Path) -> Self {
        tracing::info!("Loading app config");

        match fs::read_to_string(&config_path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(config) => config,
                Err(error) => {
                    tracing::warn!(
                        ?error,
                        path = %config_path.display(),
                        "Could not parse app config; using defaults"
                    );
                    let config = Self::default();
                    config.save_to(config_path);
                    config
                }
            },

            Err(error) => {
                tracing::warn!(
                    ?error,
                    path = %config_path.display(),
                    "Could not read app config; using defaults"
                );
                let config = Self::default();
                config.save_to(config_path);
                config
            }
        }
    }

    fn save_to(&self, config_path: &Path) {
        let result = (|| -> std::io::Result<()> {
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let contents = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
            fs::write(config_path, contents)
        })();

        if let Err(error) = result {
            tracing::warn!(
                ?error,
                path = %config_path.display(),
                "Could not save app config"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use tempfile::tempdir;

    // Compare via JSON so the tests don't need PartialEq on the nested types.
    fn as_json(config: &AppConfig) -> Value {
        serde_json::to_value(config).unwrap()
    }

    #[test]
    fn missing_file_returns_defaults_and_creates_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");

        let config = AppConfig::load_from(&path);

        assert_eq!(as_json(&config), as_json(&AppConfig::default()));
        assert!(path.exists(), "default config should be persisted");
    }

    #[test]
    fn save_then_load_roundtrips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");

        let original = AppConfig::default();
        original.save_to(&path);
        let loaded = AppConfig::load_from(&path);

        assert_eq!(as_json(&loaded), as_json(&original));
    }

    #[test]
    fn save_creates_missing_parent_dirs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a/b/c/app_config.json");

        AppConfig::default().save_to(&path);

        assert!(path.exists());
    }

    #[test]
    fn saved_file_is_pretty_printed_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");

        AppConfig::default().save_to(&path);
        let contents = fs::read_to_string(&path).unwrap();

        assert!(contents.contains('\n'), "expected pretty-printed output");
        assert!(serde_json::from_str::<Value>(&contents).is_ok());
    }

    #[test]
    fn invalid_json_returns_defaults_and_rewrites_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");
        fs::write(&path, "{ not valid json").unwrap();

        let config = AppConfig::load_from(&path);

        assert_eq!(as_json(&config), as_json(&AppConfig::default()));
        let rewritten = fs::read_to_string(&path).unwrap();
        assert!(serde_json::from_str::<Value>(&rewritten).is_ok());
    }

    #[test]
    fn empty_object_falls_back_to_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");
        fs::write(&path, "{}").unwrap();

        let config = AppConfig::load_from(&path);

        assert_eq!(as_json(&config), as_json(&AppConfig::default()));
    }

    #[test]
    fn missing_fields_use_defaults_via_serde_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");

        let mut value = as_json(&AppConfig::default());
        value.as_object_mut().unwrap().remove("language");
        fs::write(&path, value.to_string()).unwrap();

        let config = AppConfig::load_from(&path);

        assert_eq!(as_json(&config), as_json(&AppConfig::default()));
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");

        let mut value = as_json(&AppConfig::default());
        value
            .as_object_mut()
            .unwrap()
            .insert("removed_in_v2".into(), json!(true));
        fs::write(&path, value.to_string()).unwrap();

        let config = AppConfig::load_from(&path);

        assert_eq!(as_json(&config), as_json(&AppConfig::default()));
    }

    #[test]
    fn unreadable_path_returns_defaults_without_overwriting() {
        let dir = tempdir().unwrap();
        // A directory where the file should be: read_to_string fails with
        // an error that is not NotFound.
        let path = dir.path().join("app_config.json");
        fs::create_dir(&path).unwrap();

        let config = AppConfig::load_from(&path);

        assert_eq!(as_json(&config), as_json(&AppConfig::default()));
        assert!(
            path.is_dir(),
            "load must not clobber the path on read errors"
        );
    }

    #[test]
    fn save_failure_does_not_panic() {
        let dir = tempdir().unwrap();
        // Parent is a regular file, so create_dir_all must fail.
        let blocker = dir.path().join("blocker");
        fs::write(&blocker, "x").unwrap();
        let path = blocker.join("app_config.json");

        AppConfig::default().save_to(&path); // should just log a warning

        assert!(!path.exists());
    }

    #[test]
    fn save_overwrites_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app_config.json");
        fs::write(&path, "garbage").unwrap();

        AppConfig::default().save_to(&path);

        let contents = fs::read_to_string(&path).unwrap();
        assert!(serde_json::from_str::<Value>(&contents).is_ok());
    }
}
