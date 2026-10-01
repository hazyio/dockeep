use serde::{Deserialize, Serialize};

use super::{capture_sizing::CaptureSizing, image_format::ImageFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSetting {
    pub quality: u8,
    pub capture_from_surface: bool,
    pub image_format: ImageFormat,
    pub mobile_capture_sizing: CaptureSizing,
    pub desktop_capture_sizing: CaptureSizing,
    pub crop_timeout: u64,
}

impl Default for CaptureSetting {
    fn default() -> Self {
        Self {
            quality: 100,
            capture_from_surface: true,
            image_format: ImageFormat::Png,
            desktop_capture_sizing: CaptureSizing {
                width: 1920.0,
                height: 1080.0,
            },
            mobile_capture_sizing: CaptureSizing {
                width: 375.0,
                height: 667.0,
            },
            crop_timeout: 15,
        }
    }
}
