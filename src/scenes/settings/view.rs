use crate::scenes::app::MyApp;
use crate::scenes::home::view::Home;
use crate::utils::app_config::AppConfig;
use crate::utils::app_theme::AppTheme;
use gpui_kit::component::TitleBar;
use gpui_kit::component::button::*;
use gpui_kit::component::setting::{
    SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_kit::component::*;
use gpui_kit::*;
use rust_i18n::t;

pub struct SettingsPage {
    app: WeakEntity<MyApp>,
}
impl SettingsPage {
    pub fn new(app: WeakEntity<MyApp>) -> Self {
        Self { app }
    }
}
impl Render for SettingsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = AppConfig::load();
        let current_theme_name: SharedString = config.theme.name().into();
        let app = self.app.clone();

        div()
            .v_flex()
            .gap_2()
            .size_full()
            .child(
                TitleBar::new()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(t!("title.settings")),
                    )
                    .child(
                        div().flex().items_center().gap_2().child(
                            Button::new("back")
                                .ghost()
                                .label(t!("label.save"))
                                .on_click(move |_, window, cx| {
                                    if let Some(app) = app.upgrade() {
                                        let home_view: AnyView = cx
                                            .new(|cx| Home::new(app.downgrade(), window, cx))
                                            .into();
                                        app.update(cx, |app, cx| {
                                            app.navigate_to(home_view, cx);
                                        });
                                    }
                                }),
                        ),
                    ),
            )
            .child(
                Settings::new("my-settings").page(
                    SettingPage::new(t!("title.general")).group(
                        SettingGroup::new()
                            .title(t!("title.appearance"))
                            .item(SettingItem::new(
                                t!("label.theme"),
                                SettingField::dropdown(
                                    AppTheme::all()
                                        .iter()
                                        .map(|theme| (theme.name().into(), theme.name().into()))
                                        .collect(),
                                    move |_cx: &App| current_theme_name.clone(),
                                    move |val: SharedString, cx: &mut App| {
                                        let mut config = AppConfig::load();
                                        if let Some(theme) = AppTheme::all()
                                            .iter()
                                            .find(|t| t.name() == val.as_ref())
                                        {
                                            config.theme = *theme;
                                            config.save();
                                            theme.switch_to(cx);
                                        }
                                    },
                                )
                                .default_value(config.theme.name()),
                            )),
                    ),
                ),
            )
    }
}
