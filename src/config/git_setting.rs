use serde::{Deserialize, Serialize};

use crate::utils::lanuages::Languages;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSetting {
    pub auto_commit: bool,
    pub language: Languages,
}

impl Default for GitSetting {
    fn default() -> Self {
        Self {
            auto_commit: false,
            language: Languages::default(),
        }
    }
}
