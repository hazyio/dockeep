use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use serde::{Deserialize, Serialize};

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
   
    pub fn all_str() -> &'static [&'static str] {
        &["png", "jpeg", "webp"]
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
