use crate::components::window_decor::WindowDecor;
use crate::scenes::settings::chrome_page::ChromePage;
use crate::scenes::settings::general_page::GeneralPage;
use crate::utils::{app_config::AppConfig, save_debouncer::SaveDebouncer};
use gpui_kit::component::group_box::GroupBoxVariant;
use gpui_kit::component::setting::Settings;
use gpui_kit::component::*;
use gpui_kit::*;
use rust_i18n::t;

pub struct SettingsPage {
    debouncer: SaveDebouncer,
    default_config: AppConfig,
}
impl SettingsPage {
    pub fn new() -> Self {
        Self {
            debouncer: SaveDebouncer::new(AppConfig::load()),
            default_config: AppConfig::default(),
        }
    }
}
impl Render for SettingsPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let debouncer = self.debouncer.clone();
        let default_config = self.default_config.clone();

        div()
            .v_flex()
            .gap_2()
            .size_full()
            .child(WindowDecor::new(t!("title.settings")))
            .child(
                Settings::new("my-settings")
                    .with_group_variant(GroupBoxVariant::Fill)
                    .page(GeneralPage::page(debouncer.clone()))
                    .page(ChromePage::page(debouncer, &default_config)),
            )
    }
}
