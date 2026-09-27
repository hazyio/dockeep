use gpui_kit::{
    App, SharedString,
    component::setting::{
        NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage,
    },
};

use crate::utils::{app_config::AppConfig, save_debouncer::SaveDebouncer};
pub struct ChromePage {}

impl ChromePage {
    pub fn page(debouncer: SaveDebouncer, default_config: &AppConfig) -> SettingPage {
        let app_chrome_config_default = default_config.chrome_config.clone();
        let read_use_attach = debouncer.config.borrow().chrome_config.use_attach;

        SettingPage::new(t!("title.chrome")).resettable(true).group(
            SettingGroup::new()
                .title(t!("label.chrome_path"))
                .item(
                    SettingItem::new(
                        t!("label.path"),
                        SettingField::input(
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    SharedString::from(
                                        debouncer.config.borrow().chrome_config.path.clone(),
                                    )
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: SharedString, cx: &mut App| {
                                    debouncer.config.borrow_mut().chrome_config.path =
                                        val.to_string();
                                    debouncer.schedule(cx);
                                }
                            },
                        )
                        .default_value(app_chrome_config_default.path.clone()),
                    )
                    .disabled(read_use_attach),
                )
                .item(
                    SettingItem::new(
                        t!("label.use_attach"),
                        SettingField::switch(move |_| read_use_attach, {
                            let debouncer = debouncer.clone();
                            move |val: bool, cx: &mut App| {
                                debouncer.config.borrow_mut().chrome_config.use_attach = val;
                                debouncer.schedule(cx);
                            }
                        })
                        .default_value(app_chrome_config_default.use_attach),
                    )
                    .description(t!("description.use_attach").to_string()),
                )
                .item(
                    SettingItem::new(
                        t!("label.port"),
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 0.0,
                                max: 65535.0,
                                ..Default::default()
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    debouncer.config.borrow().chrome_config.attach_port as f64
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: f64, cx: &mut App| {
                                    debouncer.config.borrow_mut().chrome_config.attach_port =
                                        val as u16;
                                    debouncer.schedule(cx);
                                }
                            },
                        )
                        .default_value(app_chrome_config_default.attach_port),
                    )
                    .description(t!("description.min_port", min = 1024).to_string())
                    .disabled(!read_use_attach),
                ),
        )
    }
}
