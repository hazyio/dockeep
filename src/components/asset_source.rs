use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString};

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.len() < 1 {
            return Ok(None);
        }
        let bytes: Option<&'static [u8]> = match path {
            "trash.svg" => Some(include_bytes!("../../assets/svg/trash.svg")),
            "icon.svg" => Some(include_bytes!("../../assets/icon.svg")),
            "icons/loader.svg" | "loader.svg" => {
                Some(include_bytes!("../../assets/svg/loader.svg"))
            }
            "icons/settings.svg" | "settings.svg" => {
                Some(include_bytes!("../../assets/svg/settings.svg"))
            }
            "icons/search.svg" | "search.svg" => {
                Some(include_bytes!("../../assets/svg/search.svg"))
            }
            "icons/chevron-down.svg" => Some(include_bytes!("../../assets/svg/chevron-down.svg")),
            "icons/check.svg" | "check.svg" => Some(include_bytes!("../../assets/svg/check.svg")),
            "icons/close.svg" | "close.svg" => Some(include_bytes!("../../assets/svg/close.svg")),
            
            _ => {
                tracing::error!("could not find asset at path \"{}\"", path);
                None
            }
        };
        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let all = ["trash.svg", "icon.svg"];
        Ok(all
            .into_iter()
            .filter(|p| p.starts_with(path))
            .map(SharedString::from)
            .collect())
    }
}
