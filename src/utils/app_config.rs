use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::utils::app_theme::AppTheme;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub theme: AppTheme,
}

impl AppConfig {
    const CONFIG_FILE: &'static str = "app_config.json";
    const CONFIG_DIR: &'static str = "hazyio_dockeep";
    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .expect("Could not find a platform config directory; using defaults")
            .join(Self::CONFIG_DIR)
    }
    fn config_path() -> PathBuf {
        Self::config_dir().join(Self::CONFIG_FILE)
    }
    pub fn load() -> Self {
        let config_path = Self::config_path();

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
                    Self::save(&config);
                    config
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let config = Self::default();
                Self::save(&config);
                config
            }
            Err(error) => {
                tracing::warn!(
                    ?error,
                    path = %config_path.display(),
                    "Could not read app config; using defaults"
                );
                Self::default()
            }
        }
    }

    fn save(config: &Self) {
        let app_dir = Self::config_dir();
        let config_path = Self::config_path();
        if let Err(error) = fs::create_dir_all(app_dir)
            .and_then(|_| serde_json::to_string_pretty(config).map_err(std::io::Error::other))
            .and_then(|contents| fs::write(config_path.clone(), contents))
        {
            tracing::warn!(
                ?error,
                path = %config_path.display(),
                "Could not save app config"
            );
        }
    }
}
