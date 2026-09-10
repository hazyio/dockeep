use gpui_kit::App;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Languages {
    #[default]
    English,
    Deutsch,
}
impl Languages {
    /// All known themes — used to register them at startup.
    const ALL: &'static [Languages] = &[Languages::English, Languages::Deutsch];
    pub fn all() -> &'static [Languages] {
        Self::ALL
    }
    pub fn name_short(&self) -> &'static str {
        match self {
            Languages::English => "en",
            Languages::Deutsch => "de",
        }
    }
    pub fn name_long(&self) -> &'static str {
        match self {
            Languages::English => "English",
            Languages::Deutsch => "Deutsch",
        }
    }

    pub fn set(&self) {
        rust_i18n::set_locale(self.name_short());
    }
    pub fn set_for_app(&self, cx: &mut App) {
        self.set();
        tracing::info!("Changed language to {}",self.name_short());
        cx.refresh_windows();
    }
}
