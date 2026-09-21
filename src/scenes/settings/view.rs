use crate::scenes::app::MyApp;
use crate::scenes::home::view::HomePage;
use crate::utils::app_config::AppConfig;
use crate::utils::app_icons::AppIcons;
use crate::utils::app_theme::AppTheme;
use crate::utils::lanuages::Languages;
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
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let config = AppConfig::load();
        let current_theme_name: SharedString = config.theme.name().into();
        let current_language: SharedString = config.language.name_short().into();
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
                            Button::new("save")
                                .ghost()
                                .child(AppIcons::Check)
                                .label(t!("label.save"))
                                .on_click(move |_, window, cx| {
                                    if let Some(app) = app.upgrade() {
                                        let home_view: AnyView = cx
                                            .new(|cx| HomePage::new(app.downgrade(), window, cx))
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
                                t!("label.language"),
                                SettingField::dropdown(
                                    Languages::all()
                                        .iter()
                                        .map(|lang| {
                                            (lang.name_short().into(), lang.name_long().into())
                                        })
                                        .collect(),
                                    move |_cx: &App| current_language.clone(),
                                    move |val: SharedString, cx: &mut App| {
                                        if let Some(lang) = Languages::all()
                                            .iter()
                                            .find(|t| t.name_short() == val.as_ref())
                                        {
                                            let mut config = AppConfig::load();
                                            config.language = *lang;
                                            config.save();
                                            lang.set_for_app(cx);
                                        }
                                    },
                                )
                                .default_value(config.language.name_short()),
                            ))
                            .item(SettingItem::new(
                                t!("label.theme"),
                                SettingField::dropdown(
                                    AppTheme::all()
                                        .iter()
                                        .map(|theme| (theme.name().into(), theme.name().into()))
                                        .collect(),
                                    move |_cx: &App| current_theme_name.clone(),
                                    move |val: SharedString, cx: &mut App| {
                                        if let Some(theme) = AppTheme::all()
                                            .iter()
                                            .find(|t| t.name() == val.as_ref())
                                        {
                                            let mut config = AppConfig::load();
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
