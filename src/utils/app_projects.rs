use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};

use crate::utils::app_config::AppConfig;
use crate::utils::prelude::now_timestamp;

#[derive(Debug)]
pub enum AppProjectError {
    AlreadyExists(String),
    ProjectNotFound,
    Other(anyhow::Error),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AppProjectInfo {
    pub name: String,
    pub path: String,
    /// Unix timestamp (seconds since epoch) when the project was added.
    /// Defaults to 0 (the Unix epoch) when not present in stored data.
    #[serde(default)]
    pub last_accessed_datetime: u64,
}

impl AppProjectInfo {
    /// Updates `last_accessed_datetime` for this project in persistent storage.
    /// Loads all projects, patches the matching entry by path, and saves.
    /// Errors are logged as warnings and never propagate to the caller.
    pub fn update_lastaccess(&self) {
        let now = now_timestamp();
        let (mut projects, error) = AppProjects::load();
        if let Some(e) = error {
            tracing::warn!("update_lastaccess: failed to load projects: {e}");
            return;
        }
        if let Some(p) = projects.iter_mut().find(|p| p.path == self.path) {
            p.last_accessed_datetime = now;
        }
        if let Err(e) = (AppProjects { projects }).save() {
            tracing::warn!("update_lastaccess: failed to save projects: {e}");
        }
    }
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
    pub fn remove_project(path: &PathBuf) -> Result<(), AppProjectError> {
        let (mut projects, error) = Self::load();
        if let Some(error) = error {
            return Err(AppProjectError::Other(error));
        }
        let before = projects.len();
        projects.retain(|p| p.path != path.to_string_lossy().as_ref());
        if projects.len() == before {
            return Err(AppProjectError::ProjectNotFound);
        }
        AppProjects { projects }
            .save()
            .map_err(AppProjectError::Other)
    }

    pub fn add_project(project: AppProjectInfo) -> Result<(), AppProjectError> {
        let (mut projects, error) = Self::load();
        if let Some(error) = error {
            return Err(AppProjectError::Other(error));
        }
        if let Some(existing) = projects.iter().find(|p| p.path == project.path) {
            return Err(AppProjectError::AlreadyExists(existing.name.clone()));
        }
        projects.push(project);
        AppProjects { projects }
            .save()
            .map_err(AppProjectError::Other)
    }
}
