use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct ProjectSettingsData {
    pub save_format: crate::config::ImageFormat,
    pub save_with_tab_title: bool,
    pub auto_commit: bool,
    /// relative path to project root
    pub save_to_dir: PathBuf,
}
impl Default for ProjectSettingsData {
    fn default() -> Self {
        let app_config = crate::config::AppConfig::default();
        Self {
            save_format: app_config.capture_setting.image_format,
            auto_commit: app_config.git_setting.auto_commit,
            save_with_tab_title: app_config.capture_setting.save_with_tab_title,
            save_to_dir: PathBuf::from(""),
        }
    }
}
impl ProjectSettingsData {
    pub fn save(&self, project_path: &PathBuf) {
        let config_path = project_path.join(".dockeep");
        let contents = serde_json::to_string(self).unwrap();
        fs::write(&config_path, contents).unwrap();
    }
    pub fn load(project_path: &PathBuf) -> Self {
        let config_path = project_path.join(".dockeep");
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(config) => {
                        return config;
                    }
                    Err(error) => {
                        tracing::warn!(
                            ?error,
                            path = %config_path.display(),
                            "Could not parse app config; using defaults"
                        );
                    }
                },

                Err(error) => {
                    tracing::warn!(
                        ?error,
                        path = %config_path.display(),
                        "Could not read app config; using defaults"
                    );
                }
            }
        }
        tracing::info!("Loaded default project settings ",);
        // any error, or config not found, return default
        Self::default()
    }
}
