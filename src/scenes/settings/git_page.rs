use gpui_kit::{
    App, SharedString,
    component::setting::{SettingField, SettingGroup, SettingItem, SettingPage},
};

use crate::{
    config::AppConfig,
    utils::{lanuages::Languages, save_debouncer::SaveDebouncer},
};
pub struct GitPage {}

impl GitPage {
    pub fn page(debouncer: SaveDebouncer, default_config: &AppConfig) -> SettingPage {
        let app_config_default = default_config.clone();

        SettingPage::new(t!("title.git")).resettable(true).group(
            SettingGroup::new()
                .title(t!("title.commit"))
                .item(SettingItem::new(
                    t!("label.auto_commit"),
                    SettingField::switch(
                        {
                            let debouncer = debouncer.clone();
                            move |_: &App| debouncer.config.borrow().git_setting.auto_commit
                        },
                        {
                            let debouncer = debouncer.clone();
                            move |val: bool, cx: &mut App| {
                                debouncer.config.borrow_mut().git_setting.auto_commit = val;
                                debouncer.schedule(cx);
                            }
                        },
                    )
                    .default_value(app_config_default.git_setting.auto_commit),
                ))
                .item(
                    SettingItem::new(
                        t!("label.commit_lanuage"),
                        SettingField::dropdown(
                            Languages::all()
                                .iter()
                                .map(|lang| (lang.name_short().into(), lang.name_long().into()))
                                .collect(),
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    SharedString::from(
                                        debouncer.config.borrow().git_setting.language.name_short(),
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
                                        debouncer.config.borrow_mut().git_setting.language = *lang;
                                        debouncer.schedule(cx);
                                        lang.set_for_app(cx);
                                    }
                                }
                            },
                        )
                        .default_value(app_config_default.git_setting.language.name_short()),
                    )
                    .disabled(!debouncer.config.borrow().git_setting.auto_commit),
                ),
        )
    }
}
