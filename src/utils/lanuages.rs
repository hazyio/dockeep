use gpui_kit::App;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Languages {
    #[default]
    English,
    Deutsch,
    Spanish,
    Korean,
    Russian,
    ChineseSimplified,
}
impl Languages {
    const ALL: &'static [Languages] = &[
        Languages::English,
        Languages::Deutsch,
        Languages::Spanish,
        Languages::Korean,
        Languages::Russian,
        Languages::ChineseSimplified,
    ];
    pub fn all() -> &'static [Languages] {
        Self::ALL
    }
    pub fn name_short(&self) -> &'static str {
        match self {
            Languages::English => "en",
            Languages::Deutsch => "de",
            Languages::Spanish => "es",
            Languages::Korean => "kr",
            Languages::Russian => "ru",
            Languages::ChineseSimplified => "zh-CN",
        }
    }
    pub fn name_long(&self) -> &'static str {
        match self {
            Languages::English => "English",
            Languages::Deutsch => "Deutsch",
            Languages::Spanish => "Español",
            Languages::Korean => "한국어",
            Languages::Russian => "Русский",
            Languages::ChineseSimplified => "简体中文",
        }
    }

    pub fn set(&self) {
        rust_i18n::set_locale(self.name_short());
    }
    pub fn set_for_app(&self, cx: &mut App) {
        self.set();
        tracing::info!("Changed language to {}", self.name_short());
        cx.refresh_windows();
    }
}
