use gpui_kit::{
    App, SharedString,
    component::setting::{SettingField, SettingGroup, SettingItem, SettingPage},
};

use crate::{
    config::AppConfig,
    utils::{
        app_theme::AppTheme, date_format::DateFormat, lanuages::Languages,
        save_debouncer::SaveDebouncer, time_format::TimeFormat,
    },
};
pub struct GeneralPage {}

impl GeneralPage {
    pub fn page(debouncer: SaveDebouncer, default_config: &AppConfig) -> SettingPage {
        let app_config_default = default_config.clone();
        SettingPage::new(t!("title.general"))
            .resettable(true)
            .group(
                SettingGroup::new()
                    .title(t!("title.appearance"))
                    .item(SettingItem::new(
                        t!("label.language"),
                        SettingField::dropdown(
                            Languages::all()
                                .iter()
                                .map(|lang| (lang.name_short().into(), lang.name_long().into()))
                                .collect(),
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    SharedString::from(
                                        debouncer.config.borrow().language.name_short(),
                                    )
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: SharedString, cx: &mut App| {
                                    if let Some(lang) = Languages::all()
                                        .iter()
                                        .find(|t| t.name_short() == val.as_ref())
                                    {
                                        debouncer.config.borrow_mut().language = *lang;
                                        debouncer.schedule(cx);
                                        lang.set_for_app(cx);
                                    }
                                }
                            },
                        )
                        .default_value(app_config_default.language.name_short()),
                    ))
                    .item(SettingItem::new(
                        t!("label.theme"),
                        SettingField::dropdown(
                            AppTheme::all()
                                .iter()
                                .map(|theme| (theme.name().into(), theme.name().into()))
                                .collect(),
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    SharedString::from(debouncer.config.borrow().theme.name())
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: SharedString, cx: &mut App| {
                                    if let Some(theme) =
                                        AppTheme::all().iter().find(|t| t.name() == val.as_ref())
                                    {
                                        debouncer.config.borrow_mut().theme = *theme;
                                        debouncer.schedule(cx);
                                        theme.switch_to(cx);
                                    }
                                }
                            },
                        )
                        .default_value(app_config_default.theme.name()),
                    ))
                    .item(SettingItem::new(
                        t!("label.time_format"),
                        SettingField::dropdown(
                            TimeFormat::all()
                                .iter()
                                .map(|format| (format.id().into(), format.label().into()))
                                .collect(),
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    SharedString::from(debouncer.config.borrow().time_format.id())
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: SharedString, cx: &mut App| {
                                    if let Some(format) = TimeFormat::all()
                                        .iter()
                                        .find(|format| format.id() == val.as_ref())
                                    {
                                        debouncer.config.borrow_mut().time_format = *format;
                                        debouncer.schedule(cx);
                                    }
                                }
                            },
                        )
                        .default_value(app_config_default.time_format.id()),
                    ))
                    .item(SettingItem::new(
                        t!("label.date_format"),
                        SettingField::dropdown(
                            DateFormat::all()
                                .iter()
                                .map(|format| (format.id().into(), format.label().into()))
                                .collect(),
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    SharedString::from(debouncer.config.borrow().date_format.id())
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: SharedString, cx: &mut App| {
                                    if let Some(format) = DateFormat::all()
                                        .iter()
                                        .find(|format| format.id() == val.as_ref())
                                    {
                                        debouncer.config.borrow_mut().date_format = *format;
                                        debouncer.schedule(cx);
                                    }
                                }
                            },
                        )
                        .default_value(app_config_default.date_format.id()),
                    )),
            )
    }
}
