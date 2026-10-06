use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSizing {
    pub width: f64,
    pub height: f64,
}
