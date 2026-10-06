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
        let custom_language_key = "use_custom_language";
        let mut commit_language_option = vec![(
            SharedString::from(custom_language_key),
            SharedString::from(t!(
                "label.use_selected",
                lang = default_config.language.name_long()
            )),
        )];
        commit_language_option.extend(Languages::all().iter().map(|lang| {
            (
                SharedString::from(lang.name_short()),
                SharedString::from(lang.name_long()),
            )
        }));
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
                            commit_language_option,
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    if debouncer.config.borrow().git_setting.use_custom_language {
                                        SharedString::from(custom_language_key)
                                    } else {
                                        SharedString::from(
                                            debouncer
                                                .config
                                                .borrow()
                                                .git_setting
                                                .language
                                                .name_short(),
                                        )
                                    }
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: SharedString, cx: &mut App| {
                                    if val.as_ref() == custom_language_key {
                                        debouncer
                                            .config
                                            .borrow_mut()
                                            .git_setting
                                            .use_custom_language = true;
                                    } else {
                                        if let Some(lang) = Languages::all()
                                            .iter()
                                            .find(|t| t.name_short() == val.as_ref())
                                        {
                                            debouncer.config.borrow_mut().git_setting.language =
                                                *lang;
                                            debouncer.schedule(cx);
                                            lang.set_for_app(cx);
                                        }
                                    }
                                    debouncer.schedule(cx);
                                }
                            },
                        )
                        .default_value(custom_language_key),
                    )
                    .disabled(!debouncer.config.borrow().git_setting.auto_commit),
                ),
        )
    }
}
