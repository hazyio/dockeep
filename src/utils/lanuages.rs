use gpui_kit::App;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Lanuages {
    #[default]
    English,
}
impl Lanuages {
    /// All known themes — used to register them at startup.
    const ALL: &'static [Lanuages] = &[Lanuages::English];
    pub fn all() -> &'static [Lanuages] {
        Self::ALL
    }
    pub fn name_short(&self) -> &'static str {
        match self {
            Lanuages::English => "en",
        }
    }
    pub fn name_long(&self) -> &'static str {
        match self {
            Lanuages::English => "English",
        }
    }

    pub fn set(&self, cx: &mut App) {
        match self {
            Lanuages::English => {
                rust_i18n::set_locale(self.name_short());
            }
        }
        cx.refresh_windows();
    }
}
