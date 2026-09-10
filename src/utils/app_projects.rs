use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};

use crate::utils::app_config::AppConfig;

#[derive(Deserialize, Serialize, Clone)]
pub struct AppProjectInfo {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppProjects {
    pub projects: Vec<AppProjectInfo>,
}

impl AppProjects {
    pub fn new() -> Self {
        Self { projects: vec![] }
    }
    pub fn load() -> (Vec<AppProjectInfo>, Option<Error>) {
        let config_path = Self::config_path();
        match fs::read_to_string(&config_path) {
            Ok(contents) => match serde_json::from_str::<AppProjects>(&contents) {
                Ok(projects) => (projects.projects, None),
                Err(error) => (vec![], Some(error.into())),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (vec![], None),
            Err(error) => (vec![], Some(error.into())),
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        let backup_path = config_path.with_extension("json.bak");
        let contents =
            serde_json::to_string_pretty(self).context("Could not serialize projects")?;

        fs::create_dir_all(AppConfig::config_dir()).context("Could not create config directory")?;

        let has_backup = if config_path.exists() {
            if backup_path.exists() {
                fs::remove_file(&backup_path).with_context(|| {
                    format!("Could not remove old backup at {}", backup_path.display())
                })?;
            }
            fs::rename(&config_path, &backup_path).with_context(|| {
                format!(
                    "Could not back up projects from {} to {}",
                    config_path.display(),
                    backup_path.display()
                )
            })?;
            true
        } else {
            false
        };

        match fs::write(&config_path, contents) {
            Ok(()) => {
                if has_backup {
                    fs::remove_file(&backup_path).with_context(|| {
                        format!("Could not remove backup at {}", backup_path.display())
                    })?;
                }
                Ok(())
            }
            Err(error) => {
                if has_backup {
                    let _ = fs::remove_file(&config_path);
                    fs::rename(&backup_path, &config_path).with_context(|| {
                        format!(
                            "Could not restore projects backup to {}",
                            config_path.display()
                        )
                    })?;
                }
                Err(error).with_context(|| {
                    format!("Could not save projects to {}", config_path.display())
                })
            }
        }
    }

    fn config_path() -> PathBuf {
        AppConfig::config_dir().join("projects.json")
    }
    pub fn set_projects(&mut self, projects: Vec<AppProjectInfo>) {
        self.projects = projects;
    }
    pub fn add_project(project: AppProjectInfo) -> Result<()> {
        let (mut projects, error) = Self::load();
        if let Some(error) = error {
            return Err(error);
        }
        projects.push(project);
        AppProjects { projects }.save()
    }
}
