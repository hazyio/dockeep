use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::utils::{app_theme::AppTheme, lanuages::Languages};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
}
impl ImageFormat {
    pub fn to_cdp_format(&self) -> CaptureScreenshotFormatOption {
        match self {
            ImageFormat::Png => CaptureScreenshotFormatOption::Png,
            ImageFormat::Jpeg => CaptureScreenshotFormatOption::Jpeg,
            ImageFormat::Webp => CaptureScreenshotFormatOption::Webp,
        }
    }
    pub fn all() -> &'static [ImageFormat] {
        &[ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Webp]
    }
    pub fn to_value(&self) -> &str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Webp => "webp",
        }
    }
    pub fn from_value(value: &str) -> ImageFormat {
        match value {
            "png" => ImageFormat::Png,
            "jpeg" => ImageFormat::Jpeg,
            "webp" => ImageFormat::Webp,
            _ => ImageFormat::Png,
        }
    }
    pub fn to_string(&self) -> String {
        self.to_value().to_uppercase()
    }
}
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSizing {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeConfig {
    pub path: String,
    pub use_attach: bool,
    pub attach_port: u16,
}

fn find_chrome() -> String {
    #[cfg(target_os = "windows")]
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];

    #[cfg(target_os = "macos")]
    let candidates = ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"];

    #[cfg(target_os = "linux")]
    let candidates = [
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
    ];

    candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|p| p.to_string())
        .unwrap_or_default()
}

impl Default for ChromeConfig {
    fn default() -> Self {
        Self {
            path: find_chrome(),
            use_attach: false,
            attach_port: 9222,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub theme: AppTheme,
    pub language: Languages,
    pub chrome_config: ChromeConfig,
    pub capture_setting: CaptureSetting,
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
        tracing::info!("Loading app config");
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
                    config.save();
                    config
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let config = Self::default();
                config.save();
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

    pub fn save(&self) {
        let app_dir = Self::config_dir();
        let config_path = Self::config_path();
        if let Err(error) = fs::create_dir_all(app_dir)
            .and_then(|_| serde_json::to_string_pretty(self).map_err(std::io::Error::other))
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
