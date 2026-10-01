use crate::components::window_decor::WindowDecor;
use crate::config::AppConfig;
use crate::scenes::settings::capture_page::CapturePage;
use crate::scenes::settings::chrome_page::ChromePage;
use crate::scenes::settings::general_page::GeneralPage;
use crate::scenes::settings::git_page::GitPage;
use crate::utils::save_debouncer::SaveDebouncer;
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

        div()
            .v_flex()
            .gap_2()
            .size_full()
            .child(WindowDecor::new(t!("title.settings")))
            .child(
                Settings::new("my-settings")
                    .with_group_variant(GroupBoxVariant::Fill)
                    .page(GeneralPage::page(debouncer.clone(), &self.default_config))
                    .page(ChromePage::page(debouncer.clone(), &self.default_config))
                    .page(CapturePage::page(debouncer.clone(), &self.default_config))
                    .page(GitPage::page(debouncer, &self.default_config)),
            )
    }
}
