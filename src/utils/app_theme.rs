use gpui_kit::{
    App,
    component::{Theme, ThemeRegistry},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum AppTheme {
    AuroraLight,
    AyuLight,
    AyuDark,
    TokyoNight,
    TokyoStorm,
    TokyoMoon,
    ShadcnLight,
    #[default]
    ShadcnDark,
}

impl AppTheme {
    /// All known themes — used to register them at startup.
    const ALL: &'static [AppTheme] = &[
        AppTheme::AuroraLight,
        AppTheme::AyuLight,
        AppTheme::AyuDark,
        AppTheme::TokyoNight,
        AppTheme::TokyoStorm,
        AppTheme::TokyoMoon,
        AppTheme::ShadcnLight,
        AppTheme::ShadcnDark,
    ];
    pub fn all() -> &'static [AppTheme] {
        Self::ALL
    }

    /// Must match the `name` field inside the theme's JSON.
    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::AuroraLight => "Aurora Light",
            AppTheme::AyuLight => "Ayu Light",
            AppTheme::AyuDark => "Ayu Dark",
            AppTheme::TokyoNight => "Tokyo Night",
            AppTheme::TokyoStorm => "Tokyo Storm",
            AppTheme::TokyoMoon => "Tokyo Moon",
            AppTheme::ShadcnLight => "Shadcn Light",
            AppTheme::ShadcnDark => "Shadcn Dark",
        }
    }
    fn mode(&self) -> gpui_kit::component::ThemeMode {
        match self {
            AppTheme::AuroraLight => gpui_kit::component::ThemeMode::Light,
            AppTheme::AyuLight => gpui_kit::component::ThemeMode::Light,
            AppTheme::AyuDark => gpui_kit::component::ThemeMode::Dark,
            AppTheme::TokyoNight => gpui_kit::component::ThemeMode::Dark,
            AppTheme::TokyoStorm => gpui_kit::component::ThemeMode::Dark,
            AppTheme::TokyoMoon => gpui_kit::component::ThemeMode::Dark,
            AppTheme::ShadcnLight => gpui_kit::component::ThemeMode::Light,
            AppTheme::ShadcnDark => gpui_kit::component::ThemeMode::Dark,
        }
    }

    fn contents(&self) -> &'static str {
        match self {
            AppTheme::AuroraLight => include_str!("../../themes/aurora.json"),
            AppTheme::AyuLight => include_str!("../../themes/ayu-light.json"),
            AppTheme::AyuDark => include_str!("../../themes/ayu-dark.json"),
            AppTheme::TokyoNight => include_str!("../../themes/tokyo-night.json"),
            AppTheme::TokyoStorm => include_str!("../../themes/tokyo-storm.json"),
            AppTheme::TokyoMoon => include_str!("../../themes/tokyo-moon.json"),
            AppTheme::ShadcnLight => include_str!("../../themes/shadcn-light.json"),
            AppTheme::ShadcnDark => include_str!("../../themes/shadcn-dark.json"),
        }
    }

    /// Call once at startup. Parses every embedded theme JSON and registers
    /// it in the ThemeRegistry. Cheap to call once; do NOT call this on
    /// every theme switch — see `switch_to` for that.
    pub fn load_all(cx: &mut App) {
        let registry = ThemeRegistry::global_mut(cx);
        for theme in Self::ALL {
            if let Err(e) = registry.load_themes_from_str(theme.contents()) {
                tracing::error!("Failed to load theme '{}': {:?}", theme.name(), e);
            }
        }
        tracing::info!("Loaded {} themes", registry.sorted_themes().len());
    }

    /// Call this to switch themes at runtime. No parsing — just looks up
    /// the already-registered config and applies it.
    pub fn switch_to(&self, cx: &mut App) {
        let name: &str = self.name();
        tracing::info!("Switching to theme: {}", name);
        let theme_config = ThemeRegistry::global(cx).themes().get(name).cloned();

        match theme_config {
            Some(config) => {
                tracing::info!("Switching to theme: {}", config.name);
                Theme::global_mut(cx).apply_config(&config);
                Theme::global_mut(cx).mode = self.mode();
                cx.refresh_windows();
            }
            None => {
                tracing::error!(
                    "Theme '{}' not found — did you forget to call AppTheme::load_all(cx) at startup?",
                    name
                );
            }
        }
    }
}
